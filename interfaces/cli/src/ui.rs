use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use koi::models::{
    account::{balances::AccountBalances, metadata::WalletType},
    tx::{Tx, decode::Decoded},
};

use super::{
    app::{AccountFocus, AccountListRow, AccountPanel, App, ResourceState, Tab},
    config::Theme,
    defi::DefiResult,
    form::{AccountKind, ActiveForm, AssetType, DiscoveryState, TextForm},
    format::{DisplayAmount, format_quote, format_token, percent_change},
    icon::IconRenderer,
    layout::{AccountSidebarLayout, ListTableLayout, table_body_height},
    scroll::visible_window,
    settings::SettingsSection,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    app.layout.clear();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    app.layout.body = vertical[1];
    let viewport_rows = if app.selected_account.is_none() && app.tab == Tab::Accounts {
        if account_icons_enabled(app) {
            super::layout::table_visible_rows(vertical[1], IconRenderer::list_row_height())
        } else {
            table_body_height(vertical[1])
        }
    } else {
        table_body_height(vertical[1])
    };
    app.reconcile_scroll(viewport_rows);
    render_top_bar(frame, app, vertical[0]);
    render_body(frame, app, vertical[1]);
    render_footer(frame, app, vertical[2]);

    if app.tx_detail && app.account_panel == AccountPanel::Transactions {
        render_tx_detail(frame, app, frame.area());
    }

    if let Some(form) = &app.form {
        render_form(frame, form, frame.area(), app.theme);
    }

    if app.show_help {
        render_help(frame, app, frame.area());
    }

    if app.command_palette.is_some() {
        render_command_palette(frame, app, frame.area());
    }
}

#[derive(Clone, Copy)]
struct Palette {
    chrome: Color,
    chrome_fg: Color,
    panel: Color,
    panel_fg: Color,
    selected: Color,
    selected_fg: Color,
    muted: Color,
    accent: Color,
    overlay: Color,
}

fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Terminal => Palette {
            chrome: Color::Reset,
            chrome_fg: Color::Reset,
            panel: Color::Reset,
            panel_fg: Color::Reset,
            overlay: Color::Black,
            muted: Color::DarkGray,
            accent: Color::Cyan,
            selected: Color::DarkGray,
            selected_fg: Color::Reset,
        },
        Theme::Dark | Theme::Midnight => Palette {
            chrome: Color::Rgb(18, 24, 35),
            chrome_fg: Color::Rgb(225, 232, 240),
            panel: Color::Rgb(29, 38, 52),
            panel_fg: Color::Rgb(225, 232, 240),
            selected: Color::Rgb(50, 82, 105),
            selected_fg: Color::White,
            muted: Color::Rgb(145, 160, 175),
            accent: Color::Cyan,
            overlay: Color::Rgb(10, 14, 20),
        },
        Theme::Light | Theme::Paper => Palette {
            chrome: Color::Rgb(220, 229, 238),
            chrome_fg: Color::Rgb(24, 35, 48),
            panel: Color::Rgb(246, 249, 252),
            panel_fg: Color::Rgb(24, 35, 48),
            selected: Color::Rgb(183, 213, 235),
            selected_fg: Color::Rgb(14, 39, 59),
            muted: Color::Rgb(83, 99, 116),
            accent: Color::Blue,
            overlay: Color::Rgb(190, 200, 210),
        },
    }
}

fn render_command_palette(frame: &mut Frame, app: &App, area: Rect) {
    let Some(command) = &app.command_palette else {
        return;
    };
    let colors = palette(app.theme);
    let choices = app.command_choices();
    let height = (choices.len().min(8) + 4) as u16;
    let popup = centered_modal(area, 56, height);
    paint_area(frame, popup, colors.panel);
    let mut lines = vec![Line::from(Span::styled(
        format!(":{}", command.query),
        Style::default()
            .fg(colors.accent)
            .bg(colors.panel)
            .add_modifier(Modifier::BOLD),
    ))];
    if choices.is_empty() {
        lines.push(Line::from(Span::styled(
            " No matching commands",
            Style::default().fg(colors.muted).bg(colors.panel),
        )));
    } else {
        lines.extend(choices.iter().take(8).enumerate().map(|(index, choice)| {
            let selected = index == command.selected;
            Line::from(Span::styled(
                format!("{} {}", if selected { "›" } else { " " }, choice.label(app)),
                if selected {
                    Style::default().fg(colors.selected_fg).bg(colors.selected)
                } else {
                    Style::default().fg(colors.panel_fg).bg(colors.panel)
                },
            ))
        }));
    }
    lines.push(Line::from(Span::styled(
        " Enter select · Esc close",
        Style::default().fg(colors.muted).bg(colors.panel),
    )));
    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(colors.panel).fg(colors.panel_fg))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.accent).bg(colors.panel))
                    .style(Style::default().bg(colors.panel))
                    .title(" Command "),
            ),
        popup,
    );
}

fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let bindings: Vec<(&str, &str)> = if app.move_mode.is_some() {
        vec![
            ("j/k", "navigate · move grabbed item"),
            ("Space/Enter", "grab or drop the row"),
            ("s", "save the new layout"),
            ("Esc", "drop grab, then cancel"),
            ("g / e / x", "create · rename · delete group"),
        ]
    } else if app.selected_account.is_some() {
        vec![
            ("o/a/d/t", "overview · assets · defi · transactions"),
            ("h/l", "focus sidebar · content"),
            ("j/k", "switch panel (sidebar) · scroll (content)"),
            ("n / x", "link · unlink asset (Assets panel)"),
            ("$", "display currency"),
            ("r", "refresh current panel"),
            ("Esc/b", "back to the account list"),
        ]
    } else if app.tab == Tab::Accounts {
        vec![
            ("j/k, Enter", "navigate · open (headers: collapse)"),
            ("/", "filter accounts (Esc clears)"),
            ("n", "add account (watch · ENS · mnemonic · key)"),
            ("e / x", "rename · delete (account or group)"),
            ("g", "new group"),
            ("N", "edit account networks"),
            ("m", "move mode (reorder)"),
            ("$", "display currency"),
            ("r", "refresh everything"),
        ]
    } else {
        vec![
            ("1-5, Tab", "switch tabs"),
            ("j/k, Enter", "navigate · open"),
            ("n / e / x", "add · edit · delete/toggle"),
            ("$", "display currency"),
            ("r", "refresh"),
        ]
    };

    let width = 56;
    let height = (bindings.len() + 6).min(20) as u16;
    let popup = centered_modal(area, width, height);
    paint_area(frame, popup, PANEL_BG);

    let lines: Vec<Line> = bindings
        .into_iter()
        .map(|(keys, action)| {
            Line::from(vec![
                Span::styled(
                    format!(" {keys:<12}"),
                    Style::default()
                        .fg(Color::Cyan)
                        .bg(PANEL_BG)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(action.to_string(), Style::default().bg(PANEL_BG)),
            ])
        })
        .chain(std::iter::once(Line::from(Span::raw(""))))
        .chain(std::iter::once(Line::from(Span::styled(
            " q quit · ? close this help",
            Style::default().fg(Color::DarkGray).bg(PANEL_BG),
        ))))
        .collect();

    render_panel(frame, popup, "Keys", lines);
}

fn render_top_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    let colors = palette(app.theme);
    paint_area(frame, area, colors.chrome);
    let status = if app.loading_core {
        "Loading accounts & networks…".to_string()
    } else if let Some(notice) = &app.notice {
        notice.clone()
    } else if app.is_loading() {
        format!("{} · fetching balances…", app.status)
    } else {
        app.status.clone()
    };

    let status_color = if app.notice.is_some() {
        Color::Red
    } else if app.connected {
        Color::Green
    } else {
        Color::Yellow
    };

    let nav = [
        ("1", "Accounts", Tab::Accounts),
        ("2", "Assets", Tab::Assets),
        ("3", "Prices", Tab::Prices),
        ("4", "Networks", Tab::Networks),
        ("5", "Settings", Tab::Settings),
    ];

    let mut nav_spans = vec![Span::styled(
        "koi",
        Style::default()
            .fg(colors.accent)
            .bg(colors.chrome)
            .add_modifier(Modifier::BOLD),
    )];
    nav_spans.push(Span::raw(" "));
    let status_text = format!("{} {}", if app.connected { "●" } else { "○" }, status);
    let status_width = status_text.chars().count() as u16;
    let nav_width = area.width.saturating_sub(status_width.saturating_add(1));
    let mut tab_x = area.x.saturating_add(4);

    for (key, label, tab) in nav {
        let selected = app.tab == tab && app.selected_account.is_none();
        let style = if selected {
            Style::default()
                .fg(colors.accent)
                .bg(colors.chrome)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.muted).bg(colors.chrome)
        };
        let tab_label = format!(" {key}:{label} ");
        let width =
            (tab_label.chars().count() as u16).min(nav_width.saturating_sub(tab_x - area.x));
        if width > 0 {
            app.layout.tabs[tab as usize] = (
                Rect {
                    x: tab_x,
                    y: area.y,
                    width,
                    height: area.height,
                },
                tab,
            );
        }
        tab_x = tab_x.saturating_add(tab_label.chars().count() as u16);
        nav_spans.push(Span::styled(tab_label, style));
    }

    frame.render_widget(
        Paragraph::new(Line::from(nav_spans))
            .style(Style::default().bg(colors.chrome))
            .wrap(Wrap { trim: true }),
        Rect {
            x: area.x,
            y: area.y,
            width: nav_width,
            height: area.height,
        },
    );

    let status_line = Line::from(vec![
        Span::styled(
            if app.connected { "● " } else { "○ " },
            Style::default().fg(status_color),
        ),
        Span::styled(
            status,
            Style::default().fg(colors.chrome_fg).bg(colors.chrome),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(status_line)
            .style(Style::default().bg(colors.chrome))
            .wrap(Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Right),
        Rect {
            x: area.x.saturating_add(nav_width),
            y: area.y,
            width: area.width.saturating_sub(nav_width),
            height: area.height,
        },
    );
}

fn render_body(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.selected_account.is_some() {
        render_account_detail(frame, app, area);
        return;
    }

    match app.tab {
        Tab::Accounts => render_accounts_home(frame, app, area),
        Tab::Assets => render_assets_workspace(frame, app, area),
        Tab::Prices => render_prices_workspace(frame, app, area),
        Tab::Networks => render_networks_workspace(frame, app, area),
        Tab::Settings => render_settings(frame, app, area),
    }
}

fn render_assets_workspace(frame: &mut Frame, app: &mut App, area: Rect) {
    render_settings_assets(frame, app, area);
}

fn render_prices_workspace(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    render_price_board(frame, app, chunks[0]);
    render_quoter_table(frame, app, chunks[1], false);
}

fn render_price_board(frame: &mut Frame, app: &mut App, area: Rect) {
    let identities = app.quote_asset_identities();

    let rows = if identities.is_empty() {
        vec![Row::new(vec![
            Cell::from("No assets to price"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        let height = table_body_height(area);
        let (start, end) = visible_window(identities.len(), app.settings.row_scroll, height);
        identities
            .iter()
            .enumerate()
            .skip(start)
            .take(end.saturating_sub(start))
            .map(|(index, identity)| {
                let selected = index == app.settings.row_index;
                let (symbol, name) = app
                    .assets
                    .get(identity)
                    .map(|asset| (asset.asset_symbol.clone(), asset.asset_name.clone()))
                    .unwrap_or_else(|| (identity.clone(), String::new()));
                let (price, change) = asset_quote_cells(app, identity);
                let mut symbol_spans = vec![Span::raw(format!(
                    "    {} ",
                    if selected { "›" } else { " " }
                ))];
                if app.colored_assets
                    && let Some(icon) = app.asset_icons.get(identity)
                {
                    symbol_spans.extend(IconRenderer::color_symbol(&symbol, &icon.colors));
                } else {
                    symbol_spans.push(Span::raw(symbol));
                }
                Row::new(vec![
                    Cell::from(Line::from(symbol_spans)),
                    Cell::from(truncate(&name, 26)),
                    price,
                    change,
                ])
                .style(selected_row_style(app.theme, selected))
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Percentage(40),
            Constraint::Length(18),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["Symbol", "Name", "Price", "24h"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
    render_asset_table_icons(frame, app, area, &identities, app.settings.row_scroll);
    register_resource_list_table(app, area, identities.len());
}

fn render_networks_workspace(frame: &mut Frame, app: &mut App, area: Rect) {
    render_settings_networks(frame, app, area);
}

fn render_accounts_home(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_accounts_list(frame, app, chunks[0]);
    render_selected_account_preview(frame, app, chunks[1]);
}

fn render_accounts_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let show_icons = account_icons_enabled(app);
    let row_height = if show_icons {
        IconRenderer::list_row_height()
    } else {
        1
    };
    let height = if show_icons {
        super::layout::table_visible_rows(area, row_height)
    } else {
        table_body_height(area)
    };
    let list_rows = app.account_rows();
    let (start, end) = visible_window(list_rows.len(), app.list_scroll, height);
    let empty_columns = if show_icons { 4 } else { 3 };
    let rows: Vec<Row> = if list_rows.is_empty() {
        let mut cells = vec![Cell::from("No accounts yet")];
        cells.extend((0..empty_columns).map(|_| Cell::from("")));
        vec![Row::new(cells)]
    } else {
        list_rows
            .iter()
            .enumerate()
            .skip(start)
            .take(end.saturating_sub(start))
            .map(|(index, list_row)| {
                let selected = index == app.list_index;
                let grabbed = app.is_grabbed_row(list_row);
                let mut row_style = selected_row_style(app.theme, selected);
                if grabbed {
                    row_style = row_style.add_modifier(Modifier::REVERSED);
                }
                let caret = if grabbed {
                    "◆"
                } else if selected {
                    "›"
                } else {
                    " "
                };

                let mut cells = Vec::new();
                if show_icons {
                    cells.push(Cell::from(""));
                }
                match list_row {
                    AccountListRow::GroupHeader {
                        name,
                        collapsed,
                        count,
                        ..
                    } => {
                        let arrow = if *collapsed { "▸" } else { "▾" };
                        cells.push(
                            Cell::from(format!("{caret} {arrow} {name} ({count})")).style(
                                row_style.add_modifier(Modifier::BOLD).fg(if selected {
                                    Color::White
                                } else {
                                    Color::Cyan
                                }),
                            ),
                        );
                        cells.extend((0..empty_columns).map(|_| Cell::from("")));
                    }
                    AccountListRow::EmptyGroup => {
                        cells.push(
                            Cell::from(format!("{caret}   (no accounts)"))
                                .style(row_style.fg(Color::DarkGray)),
                        );
                        cells.extend((0..empty_columns).map(|_| Cell::from("")));
                    }
                    AccountListRow::Account { account_id } => {
                        let (balance_text, balance_style) =
                            balance_cell(app, *account_id, app.balance_state(*account_id));
                        let (name, metadata) = app
                            .account_by_id(*account_id)
                            .map(|account| (account.name.clone(), Some(&account.metadata)))
                            .unwrap_or_else(|| (String::new(), None));
                        let indent = if app.groups.is_empty() { "" } else { "  " };
                        cells.extend([
                            Cell::from(format!("{caret} {indent}{name}")).style(row_style),
                            Cell::from(metadata.map(wallet_type_label).unwrap_or_default())
                                .style(row_style),
                            Cell::from(metadata.map(truncate_address).unwrap_or_default())
                                .style(row_style),
                            Cell::from(balance_text).style(row_style.patch(balance_style)),
                        ]);
                    }
                }

                let mut row = Row::new(cells).style(row_style);
                if show_icons {
                    row = row.height(row_height);
                }
                row
            })
            .collect()
    };

    let constraints = if show_icons {
        vec![
            Constraint::Length(IconRenderer::list_column_width()),
            Constraint::Percentage(32),
            Constraint::Length(8),
            Constraint::Percentage(28),
            Constraint::Length(16),
        ]
    } else {
        vec![
            Constraint::Percentage(34),
            Constraint::Length(8),
            Constraint::Percentage(30),
            Constraint::Length(16),
        ]
    };

    let header = if show_icons {
        Row::new(vec!["", "Name", "Type", "Address", "Balance"])
    } else {
        Row::new(vec!["Name", "Type", "Address", "Balance"])
    };

    let table = Table::new(rows, constraints)
        .header(header.style(Style::default().add_modifier(Modifier::BOLD)))
        .column_spacing(2);

    frame.render_widget(table, area);

    if show_icons {
        let addresses: Vec<(usize, String)> = list_rows
            .iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .enumerate()
            .filter_map(|(visible_row, list_row)| {
                let AccountListRow::Account { account_id } = list_row else {
                    return None;
                };
                let account = app.account_by_id(*account_id)?;
                Some((visible_row, account_evm_address(&account.metadata)?))
            })
            .collect();
        if let Some(renderer) = app.icon_renderer.as_mut() {
            for (visible_row, address) in addresses {
                renderer.render_list_icon(
                    frame,
                    account_list_icon_rect(area, visible_row, row_height),
                    &address,
                );
            }
        }
    }

    app.layout.list_table = Some(ListTableLayout {
        area,
        scroll: app.list_scroll,
        len: list_rows.len(),
        row_height,
    });
}

fn render_selected_account_preview(frame: &mut Frame, app: &mut App, area: Rect) {
    let selected = app.selected_list_row();

    if let Some(AccountListRow::GroupHeader {
        name,
        collapsed,
        count,
        ..
    }) = &selected
    {
        let lines = vec![
            Line::from(Span::styled(
                name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("{count} account(s)"),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(format!(
                "Enter: {}",
                if *collapsed { "expand" } else { "collapse" }
            )),
            Line::from("e: rename group"),
            Line::from("x: delete group"),
            Line::from("m: move accounts"),
        ];
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
        return;
    }

    let account = match &selected {
        Some(AccountListRow::Account { account_id }) => app.account_by_id(*account_id),
        _ => None,
    };
    let Some(account) = account else {
        frame.render_widget(Paragraph::new("No account selected"), area);
        return;
    };

    let (balance, balance_style) = balance_cell(
        app,
        account.account_identity.0,
        app.balance_state(account.account_identity.0),
    );
    let mut lines = vec![
        Line::from(Span::styled(
            account.name.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            truncate_address(&account.metadata),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Total balance",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            balance,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
                .patch(balance_style),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} networks", account.networks.len()),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from("Enter: open account"),
        Line::from("r: refresh balances"),
    ];

    if let Some(ResourceState::Error(error)) = app.balance_state(account.account_identity.0) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            truncate_error(error),
            Style::default().fg(Color::Red),
        )));
    }

    let inner = area;
    let address = account_evm_address(&account.metadata);

    if let Some(address) = address.as_deref() {
        if account_icons_enabled(app) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(IconRenderer::icon_height()),
                    Constraint::Min(0),
                ])
                .split(inner);
            render_account_icon(frame, app, chunks[0], address);
            frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), chunks[1]);
        } else {
            frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
        }
    } else {
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
    }
}

fn render_settings(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let section_line = Line::from(
        SettingsSection::ALL
            .iter()
            .enumerate()
            .map(|(index, section)| {
                let selected = index == app.settings.section_index;
                let style = if selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                Span::styled(format!(" {} ", section.title()), style)
            })
            .collect::<Vec<_>>(),
    );

    frame.render_widget(Paragraph::new(section_line), chunks[0]);

    render_settings_detail(frame, app, chunks[1]);
}

fn render_settings_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.settings.section() {
        SettingsSection::General => render_settings_general(frame, app, area),
        SettingsSection::Networks => render_settings_networks(frame, app, area),
        SettingsSection::Assets => render_settings_assets(frame, app, area),
        SettingsSection::PriceFeeds => render_settings_price_feeds(frame, app, area),
        SettingsSection::Vendors => render_settings_vendors(frame, app, area),
    }
}

fn render_settings_general(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![
        Line::from(Span::styled(
            "Display currency",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            app.display_currency.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Press $ to pick a fiat asset as the display currency."),
        Line::from("The choice is saved to the koi/tui.json config file."),
        Line::from(""),
        Line::from(Span::styled("Theme", Style::default().fg(Color::DarkGray))),
        Line::from(format!("{} · press t to switch", app.theme.label())),
        Line::from(""),
        Line::from(Span::styled(
            "Colored assets",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(format!(
            "{} · press c to toggle",
            if app.colored_assets {
                "enabled"
            } else {
                "disabled"
            }
        )),
    ];

    if let Some(notice) = &app.settings.notice {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            notice.clone(),
            Style::default().fg(Color::Yellow),
        )));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

fn render_settings_networks(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(network_id) = app.settings.nested_network {
        render_settings_endpoints(frame, app, network_id, area);
        return;
    }

    let rows = if app.networks.is_empty() {
        vec![Row::new(vec![
            Cell::from("No networks configured"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        let height = table_body_height(area);
        let (start, end) = visible_window(app.networks.len(), app.settings.row_scroll, height);
        app.networks
            .iter()
            .enumerate()
            .skip(start)
            .take(end.saturating_sub(start))
            .map(|(index, network)| {
                let selected = index == app.settings.row_index;
                let style = selected_row_style(app.theme, selected);
                let endpoint_count = app
                    .settings_snapshot()
                    .and_then(|snapshot| snapshot.endpoints.get(&network.network_identity.0))
                    .map(|endpoints| endpoints.len())
                    .unwrap_or_default();
                let health = match app.rpc_state(network.network_identity.0) {
                    Some(ResourceState::Ready(stats)) => {
                        let text = format!("{}/{} alive", stats.alive_count, stats.endpoint_count);
                        let color = if stats.alive_count == 0 && stats.endpoint_count > 0 {
                            Color::Red
                        } else if stats.dead_count > 0 {
                            Color::Yellow
                        } else {
                            Color::Green
                        };
                        Cell::from(text).style(Style::default().fg(color))
                    }
                    Some(ResourceState::Loading) => {
                        Cell::from("…").style(Style::default().fg(Color::Yellow))
                    }
                    Some(ResourceState::Error(_)) => {
                        Cell::from("error").style(Style::default().fg(Color::Red))
                    }
                    _ => Cell::from("—").style(Style::default().fg(Color::DarkGray)),
                };

                Row::new(vec![
                    Cell::from(format!(
                        "{} {}",
                        if selected { "›" } else { " " },
                        network.network_name
                    )),
                    Cell::from(network.network_identity.0.to_string()),
                    health,
                    Cell::from(format!("{} endpoint(s)", endpoint_count)),
                ])
                .style(style)
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Percentage(30),
        ],
    )
    .header(
        Row::new(vec!["Network", "Chain ID", "RPC health", "Endpoints"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
    register_resource_list_table(app, area, app.networks.len());
}

fn render_settings_endpoints(frame: &mut Frame, app: &mut App, network_id: u64, area: Rect) {
    let endpoints = app
        .settings_snapshot()
        .and_then(|snapshot| snapshot.endpoints.get(&network_id));

    let rows = match endpoints {
        Some(endpoints) if endpoints.is_empty() => vec![Row::new(vec![
            Cell::from("No endpoints"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(endpoints) => {
            let height = table_body_height(area);
            let (start, end) = visible_window(endpoints.len(), app.settings.row_scroll, height);
            endpoints
                .iter()
                .enumerate()
                .skip(start)
                .take(end.saturating_sub(start))
                .map(|(index, endpoint)| {
                    let selected = index == app.settings.row_index;
                    let style = selected_row_style(app.theme, selected);
                    let live_status = match app.rpc_state(network_id) {
                        Some(ResourceState::Ready(stats)) => stats
                            .endpoints
                            .iter()
                            .find(|stats| stats.endpoint_identity == endpoint.endpoint_identity)
                            .map(|stats| stats.status.clone()),
                        _ => None,
                    };
                    let status = if endpoint.endpoint_disabled {
                        ("disabled".to_string(), Style::default().fg(Color::DarkGray))
                    } else {
                        match live_status.as_deref() {
                            Some("alive") => {
                                ("alive".to_string(), Style::default().fg(Color::Green))
                            }
                            Some("dead") => ("dead".to_string(), Style::default().fg(Color::Red)),
                            Some(other) => (other.to_string(), Style::default().fg(Color::Yellow)),
                            None => ("enabled".to_string(), Style::default()),
                        }
                    };
                    Row::new(vec![
                        Cell::from(format!(
                            "{} #{}",
                            if selected { "›" } else { " " },
                            endpoint.endpoint_identity
                        )),
                        Cell::from(
                            endpoint
                                .endpoint_label
                                .clone()
                                .unwrap_or_else(|| "—".to_string()),
                        ),
                        Cell::from(endpoint.endpoint_type.clone()),
                        Cell::from(status.0).style(status.1),
                        Cell::from(truncate(&endpoint.endpoint_url, 54)),
                    ])
                    .style(style)
                })
                .collect()
        }
        None => vec![Row::new(vec![
            Cell::from("Loading endpoints…"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["ID", "Label", "Type", "Status", "URL"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
    register_resource_list_table(app, area, endpoints.map(|items| items.len()).unwrap_or(0));
}

fn asset_quote_cells(app: &App, identity: &str) -> (Cell<'static>, Cell<'static>) {
    let price = match app.asset_quotes.get(identity) {
        Some(ResourceState::Ready(value)) => {
            let formatted = format_quote(value, app.display_asset());
            Cell::from(formatted.text.clone()).style(formatted.ratatui_style())
        }
        Some(ResourceState::Loading) => Cell::from("…").style(Style::default().fg(Color::Yellow)),
        _ => Cell::from("—").style(Style::default().fg(Color::DarkGray)),
    };

    let change = app
        .asset_24h_change(identity)
        .and_then(|(current, previous)| percent_change(&current, &previous))
        .map(|change| Cell::from(change.label()).style(change.ratatui_style()))
        .unwrap_or_else(|| Cell::from("—").style(Style::default().fg(Color::DarkGray)));

    (price, change)
}

fn render_settings_assets(frame: &mut Frame, app: &mut App, area: Rect) {
    let identities = app.settings_asset_identities();

    let rows = if identities.is_empty() {
        vec![Row::new(vec![
            Cell::from("No assets configured"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        let height = table_body_height(area);
        let (start, end) = visible_window(identities.len(), app.settings.row_scroll, height);
        identities
            .iter()
            .enumerate()
            .skip(start)
            .take(end.saturating_sub(start))
            .filter_map(|(index, identity)| {
                let asset = app.assets.get(identity)?;
                let selected = index == app.settings.row_index;
                let (price, change) = asset_quote_cells(app, identity);
                let mut symbol_spans = vec![Span::raw(format!(
                    "    {} ",
                    if selected { "›" } else { " " }
                ))];
                if app.colored_assets
                    && let Some(icon) = app.asset_icons.get(identity)
                {
                    symbol_spans.extend(IconRenderer::color_symbol(
                        &asset.asset_symbol,
                        &icon.colors,
                    ));
                } else {
                    symbol_spans.push(Span::raw(asset.asset_symbol.clone()));
                }
                Some(
                    Row::new(vec![
                        Cell::from(Line::from(symbol_spans)),
                        Cell::from(truncate(&asset.asset_name, 26)),
                        price,
                        change,
                        Cell::from(asset.asset_decimals.to_string()),
                        Cell::from(truncate(&asset.asset_identity.to_string(), 42)),
                    ])
                    .style(selected_row_style(app.theme, selected)),
                )
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Percentage(22),
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(24),
        ],
    )
    .header(
        Row::new(vec![
            "Symbol", "Name", "Price", "24h", "Decimals", "Identity",
        ])
        .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
    render_asset_table_icons(frame, app, area, &identities, app.settings.row_scroll);
    register_resource_list_table(app, area, identities.len());
}

fn render_settings_price_feeds(frame: &mut Frame, app: &mut App, area: Rect) {
    render_quoter_table(frame, app, area, true);
}

fn render_quoter_table(frame: &mut Frame, app: &mut App, area: Rect, interactive: bool) {
    let rows = match app.settings_snapshot() {
        Some(snapshot) if snapshot.quoters.is_empty() => vec![Row::new(vec![
            Cell::from("No price feeds configured"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(snapshot) => {
            let height = table_body_height(area);
            let (start, end) =
                visible_window(snapshot.quoters.len(), app.settings.row_scroll, height);
            snapshot
                .quoters
                .iter()
                .enumerate()
                .skip(start)
                .take(end.saturating_sub(start))
                .map(|(index, quoter)| {
                    let selected = interactive && index == app.settings.row_index;
                    Row::new(vec![
                        Cell::from(format!(
                            "{} {}",
                            if selected { "›" } else { " " },
                            quoter.quoter_name
                        )),
                        Cell::from(if quoter.enabled {
                            "enabled"
                        } else {
                            "disabled"
                        }),
                        Cell::from(format!("{} -> {}", quoter.token_a, quoter.token_b)),
                        Cell::from(truncate(&quoter.quoter_identity, 16)),
                    ])
                    .style(selected_row_style(app.theme, selected))
                })
                .collect()
        }
        None => vec![Row::new(vec![
            Cell::from("Loading price feeds…"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
    };

    let quoter_count = app
        .settings_snapshot()
        .map(|snapshot| snapshot.quoters.len())
        .unwrap_or(0);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(28),
            Constraint::Length(10),
            Constraint::Percentage(46),
            Constraint::Length(18),
        ],
    )
    .header(
        Row::new(vec!["Name", "Status", "Pair", "ID"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
    if interactive {
        register_resource_list_table(app, area, quoter_count);
    }
}

fn render_settings_vendors(frame: &mut Frame, app: &mut App, area: Rect) {
    let rows = match app.settings_snapshot() {
        Some(snapshot) if snapshot.all_vendors.is_empty() => vec![Row::new(vec![
            Cell::from("No vendor flags returned"),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(snapshot) => {
            let height = table_body_height(area);
            let (start, end) =
                visible_window(snapshot.all_vendors.len(), app.settings.row_scroll, height);
            snapshot
                .all_vendors
                .iter()
                .enumerate()
                .skip(start)
                .take(end.saturating_sub(start))
                .map(|(index, vendor)| {
                    let flag = vendor.flag.to_string();
                    let enabled = snapshot.enabled_vendors.contains(&flag);
                    let selected = index == app.settings.row_index;
                    Row::new(vec![
                        Cell::from(format!("{} {}", if selected { "›" } else { " " }, flag)),
                        Cell::from(if enabled { "enabled" } else { "disabled" }).style(
                            if enabled {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            },
                        ),
                        Cell::from(if vendor.unfinished { "unfinished" } else { "" }),
                        Cell::from(vendor.comment.clone()),
                    ])
                    .style(selected_row_style(app.theme, selected))
                })
                .collect()
        }
        None => vec![Row::new(vec![
            Cell::from("Loading vendors…"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(28),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["Flag", "Status", "State", "Comment"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
    if let Some(snapshot) = app.settings_snapshot() {
        register_resource_list_table(app, area, snapshot.all_vendors.len());
    }
}

fn render_account_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(account) = app.selected_account() else {
        return;
    };

    let account_id = account.account_identity.0;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(26), Constraint::Min(0)])
        .split(area);

    let (balance_label, balance_amount_style) =
        balance_cell(app, account_id, app.balance_state(account_id));

    let pages = [
        (AccountPanel::Overview, "o", "Overview"),
        (AccountPanel::Assets, "a", "Assets"),
        (AccountPanel::Defi, "d", "DeFi"),
        (AccountPanel::Transactions, "t", "Tx"),
    ];

    let mut sidebar_lines = vec![Line::from(vec![
        Span::styled("Balance ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            balance_label,
            balance_amount_style.patch(balance_sidebar_style(
                app,
                account_id,
                app.balance_state(account_id),
            )),
        ),
    ])];

    for (panel, key, label) in pages {
        let selected = app.account_panel == panel;
        let focused = app.account_focus == AccountFocus::Sidebar && selected;
        let style = if focused {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else if selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        sidebar_lines.push(Line::from(Span::styled(
            format!(" {key}:{label}{}", if selected { " ›" } else { "" }),
            style,
        )));
    }

    let sidebar_focus = if app.account_focus == AccountFocus::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    sidebar_lines.extend([
        Line::from(""),
        Line::from(Span::styled("←→ focus pane", sidebar_focus)),
        Line::from(Span::styled(
            "↑↓ select page",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    let sidebar_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(chunks[0])[1];
    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(IconRenderer::icon_height()),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(sidebar_area);
    let header_lines = vec![
        Line::from(Span::styled(
            truncate(&account.name, 20),
            if app.account_focus == AccountFocus::Sidebar {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            },
        )),
        Line::from(Span::styled(
            truncate_address(&account.metadata),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            wallet_type_label(&account.metadata),
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(Paragraph::new(header_lines), sidebar[2]);
    let sidebar_inner = sidebar[3];

    if let Some(address) = account_evm_address(&account.metadata) {
        if account_icons_enabled(app) {
            render_account_icon(frame, app, sidebar[1], &address);
            let text_area = sidebar_inner;
            frame.render_widget(
                Paragraph::new(sidebar_lines).wrap(Wrap { trim: true }),
                text_area,
            );
            register_account_sidebar_panels(app, chunks[0], text_area);
        } else {
            frame.render_widget(
                Paragraph::new(sidebar_lines).wrap(Wrap { trim: true }),
                sidebar_inner,
            );
            register_account_sidebar_panels(app, chunks[0], sidebar_inner);
        }
    } else {
        frame.render_widget(
            Paragraph::new(sidebar_lines).wrap(Wrap { trim: true }),
            sidebar_inner,
        );
        register_account_sidebar_panels(app, chunks[0], sidebar_inner);
    }

    let content_title = account_content_title(app, account_id);
    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(chunks[1]);
    frame.render_widget(
        Paragraph::new(Span::styled(
            content_title,
            if app.account_focus == AccountFocus::Content {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            },
        )),
        content[0],
    );

    match app.account_panel {
        AccountPanel::Overview => render_account_overview(frame, app, account_id, content[1]),
        AccountPanel::Assets => render_account_assets(frame, app, account_id, content[1]),
        AccountPanel::Defi => render_account_defi(frame, app, account_id, content[1]),
        AccountPanel::Transactions => {
            render_account_transactions(frame, app, account_id, content[1])
        }
    }
}

fn account_content_title(app: &App, account_id: u64) -> String {
    let panel = account_panel_title(app.account_panel);
    let detail = match app.account_panel {
        AccountPanel::Defi => match app.defi_state(account_id) {
            Some(ResourceState::Ready(result)) if !result.errors.is_empty() => {
                format!(" · {} warning(s)", result.errors.len())
            }
            Some(ResourceState::Ready(result)) => {
                format!(" · {} position(s)", result.positions.len())
            }
            _ => String::new(),
        },
        AccountPanel::Transactions => match app.tx_state(account_id) {
            Some(ResourceState::Ready(transactions)) => format!(" · {} tx", transactions.len()),
            _ => String::new(),
        },
        AccountPanel::Overview | AccountPanel::Assets => String::new(),
    };
    format!("{panel}{detail}")
}

fn account_panel_title(panel: AccountPanel) -> &'static str {
    match panel {
        AccountPanel::Overview => "Overview",
        AccountPanel::Assets => "Assets",
        AccountPanel::Defi => "DeFi",
        AccountPanel::Transactions => "Transactions",
    }
}

fn render_horizontal_rule(frame: &mut Frame, area: Rect) {
    let width = area.width.saturating_sub(1) as usize;
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(width.max(1)),
            Style::default().fg(Color::DarkGray),
        ))),
        area,
    );
}

fn render_account_overview(frame: &mut Frame, app: &mut App, account_identity: u64, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let state = app.balance_state(account_identity);
    let (total, total_style, updated) = match state {
        Some(ResourceState::Ready(balances)) => {
            let formatted = balances
                .total_quote
                .as_ref()
                .map(|value| format_quote(value, app.display_asset()))
                .unwrap_or(DisplayAmount {
                    text: "—".to_string(),
                    style: super::format::AmountStyle::Normal,
                });
            let total_style = formatted.ratatui_style();
            (
                formatted.text,
                total_style,
                balances.updated_at.format("%H:%M:%S").to_string(),
            )
        }
        Some(ResourceState::Loading) => (
            "Loading…".to_string(),
            Style::default().fg(Color::Yellow),
            "—".to_string(),
        ),
        Some(ResourceState::Error(error)) => (
            truncate_error(error),
            Style::default().fg(Color::Red),
            "—".to_string(),
        ),
        _ => ("—".to_string(), Style::default(), "—".to_string()),
    };

    let mut summary_lines = vec![
        Line::from(Span::styled(
            "Total balance",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            total,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
                .patch(total_style),
        )),
    ];

    if let Some(ResourceState::Ready(balances)) = state {
        if !balances.errors.is_empty() {
            summary_lines.push(Line::from(Span::styled(
                format!("{} balance warning(s)", balances.errors.len()),
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    summary_lines.push(Line::from(Span::styled(
        if app.is_balance_refreshing(account_identity) {
            format!("Updated {updated} · refreshing")
        } else {
            format!("Updated {updated}")
        },
        Style::default().fg(Color::DarkGray),
    )));

    frame.render_widget(Paragraph::new(summary_lines), chunks[0]);
    render_horizontal_rule(frame, chunks[1]);
    render_account_asset_table(frame, app, account_identity, chunks[2]);
}

fn render_account_assets(frame: &mut Frame, app: &mut App, account_identity: u64, area: Rect) {
    render_account_asset_table(frame, app, account_identity, area);
}

fn render_account_asset_table(frame: &mut Frame, app: &mut App, account_identity: u64, area: Rect) {
    let (table_rows, identities): (Vec<Row>, Vec<String>) =
        match app.balance_state(account_identity) {
            None | Some(ResourceState::Idle) => (
                vec![Row::new(vec![
                    Cell::from("No balance data yet — press r to refresh"),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])],
                Vec::new(),
            ),
            Some(ResourceState::Loading) => (
                vec![Row::new(vec![
                    Cell::from("Loading balances…"),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])],
                Vec::new(),
            ),
            Some(ResourceState::Error(error)) => (
                vec![Row::new(vec![
                    Cell::from(truncate_error(error)),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])],
                Vec::new(),
            ),
            Some(ResourceState::Ready(balances)) => {
                let mut identities: Vec<_> = balances
                    .balances
                    .iter()
                    .map(|balance| {
                        (
                            balance
                                .balance_quote
                                .as_deref()
                                .and_then(|value| value.parse::<u128>().ok())
                                .unwrap_or_default(),
                            balance.asset_identity.to_string(),
                        )
                    })
                    .collect();
                identities.sort_by(|left, right| right.0.cmp(&left.0));
                (
                    asset_rows(app, balances),
                    identities
                        .into_iter()
                        .map(|(_, identity)| identity)
                        .collect(),
                )
            }
        };

    let height = table_body_height(area);
    let (start, end) = visible_window(table_rows.len(), app.account_scroll, height);
    app.account_scroll = start;
    let table_rows: Vec<Row> = table_rows
        .into_iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect();

    let table = Table::new(
        table_rows,
        [
            Constraint::Percentage(32),
            Constraint::Percentage(26),
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["    Asset", "Balance", "Value", "Weight", "24h"]).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::DarkGray),
        ),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
    if !identities.is_empty() {
        render_asset_table_icons(
            frame,
            app,
            area,
            &identities[start.min(identities.len())..],
            0,
        );
    }
}

fn render_account_defi(frame: &mut Frame, app: &mut App, account_identity: u64, area: Rect) {
    let state = app.defi_state(account_identity);
    let show_notice = matches!(
        state,
        Some(ResourceState::Ready(result)) if !result.errors.is_empty()
    );
    let chunks = if show_notice {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(area)
    };

    if show_notice {
        if let Some(ResourceState::Ready(result)) = state {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!(
                        "{} protocol warning(s) — some positions may be missing",
                        result.errors.len()
                    ),
                    Style::default().fg(Color::Yellow),
                ))),
                chunks[0],
            );
        }
    }

    let table_area = if show_notice { chunks[1] } else { chunks[0] };

    let (rows, identities): (Vec<Row>, Vec<String>) = match state {
        None | Some(ResourceState::Idle) => (
            vec![Row::new(vec![
                Cell::from("No DeFi data yet — press d or r to refresh"),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ])],
            Vec::new(),
        ),
        Some(ResourceState::Loading) => (
            vec![Row::new(vec![
                Cell::from("Loading DeFi positions…"),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ])],
            Vec::new(),
        ),
        Some(ResourceState::Error(error)) => (
            vec![Row::new(vec![
                Cell::from(truncate_error(error)),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ])],
            Vec::new(),
        ),
        Some(ResourceState::Ready(result)) => (
            defi_rows(result),
            result
                .positions
                .iter()
                .map(|position| {
                    app.assets
                        .values()
                        .find(|asset| asset.asset_symbol == position.underlying_symbol)
                        .map(|asset| asset.asset_identity.to_string())
                        .unwrap_or_default()
                })
                .collect(),
        ),
    };

    let height = table_body_height(table_area);
    let (start, end) = visible_window(rows.len(), app.account_scroll, height);
    app.account_scroll = start;
    let rows: Vec<Row> = rows
        .into_iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Percentage(28),
            Constraint::Length(16),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Length(18),
        ],
    )
    .header(
        Row::new(vec![
            "Chain",
            "Protocol",
            "    Position",
            "Value",
            "TVL",
            "APR",
            "7d earned",
        ])
        .style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::DarkGray),
        ),
    )
    .column_spacing(2);

    frame.render_widget(table, table_area);
    if !identities.is_empty() {
        render_asset_table_icons_at(
            frame,
            app,
            table_area,
            &identities[start.min(identities.len())..],
            0,
            24,
        );
    }
}

fn render_account_transactions(frame: &mut Frame, app: &App, account_identity: u64, area: Rect) {
    let state = app.tx_state(account_identity);

    let rows: Vec<Row> = match state {
        None | Some(ResourceState::Idle) => vec![Row::new(vec![
            Cell::from("No transaction data yet — press t or r to refresh"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(ResourceState::Loading) => vec![Row::new(vec![
            Cell::from("Loading transactions…"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(ResourceState::Error(error)) => vec![Row::new(vec![
            Cell::from(truncate_error(error)),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(ResourceState::Ready(transactions)) => transaction_rows(
            transactions,
            (app.account_focus == AccountFocus::Content).then(|| app.clamped_tx_index()),
        ),
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(11),
            Constraint::Length(18),
            Constraint::Percentage(24),
            Constraint::Percentage(22),
            Constraint::Percentage(22),
        ],
    )
    .header(
        Row::new(vec!["Nonce", "Status", "Date", "Action", "Target", "Hash"]).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::DarkGray),
        ),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn defi_rows(result: &DefiResult) -> Vec<Row<'static>> {
    if result.positions.is_empty() {
        let message = if result.errors.is_empty() {
            "No tracked DeFi positions found"
        } else {
            "No positions found; some protocols failed"
        };
        return vec![Row::new(vec![
            Cell::from(message),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])];
    }

    result
        .positions
        .iter()
        .map(|position| {
            let value = if position.value_usd > 0.0 {
                format_fiat(position.value_usd)
            } else {
                format!(
                    "{} {}",
                    format_float(position.value, 4),
                    position.underlying_symbol
                )
            };
            let earned = if position.earned_7d_usd > 0.0 {
                format!("+{}", format_fiat(position.earned_7d_usd))
            } else if position.earned_7d > 0.0 {
                format!(
                    "+{} {}",
                    format_float(position.earned_7d, 6),
                    position.underlying_symbol
                )
            } else {
                "—".to_string()
            };

            Row::new(vec![
                Cell::from(truncate(
                    &format!("{}:{}", position.chain_name, position.chain_id),
                    10,
                )),
                Cell::from(position.protocol.clone()),
                Cell::from(format!("    {}", truncate(&position.name, 28))),
                Cell::from(value),
                Cell::from(format_fiat(position.tvl_usd)),
                Cell::from(format_percent(position.apr)),
                Cell::from(earned),
            ])
        })
        .collect()
}

fn transaction_rows(transactions: &[Tx], selected: Option<usize>) -> Vec<Row<'static>> {
    if transactions.is_empty() {
        return vec![Row::new(vec![
            Cell::from("No transactions found"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])];
    }

    transactions
        .iter()
        .enumerate()
        .map(|(index, tx)| {
            let status = transaction_status(tx);
            let status_style = match status.as_str() {
                "Success" => Style::default().fg(Color::Green),
                "Failed" => Style::default().fg(Color::Red),
                "Executed" => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::DarkGray),
            };
            let row_style = if selected == Some(index) {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(transaction_nonce(tx)),
                Cell::from(status).style(status_style),
                Cell::from(transaction_date(tx)),
                Cell::from(truncate(&transaction_action(tx), 28)),
                Cell::from(truncate(&transaction_target(tx), 26)),
                Cell::from(transaction_hash(tx)),
            ])
            .style(row_style)
        })
        .collect()
}

fn decoded_call_label(call: &koi::models::tx::decode::DecodedCall) -> String {
    use koi::models::tx::decode::Decoded;

    match &call.decoded {
        Decoded::Verified(function) => {
            let contract = function
                .contract
                .verified_name
                .clone()
                .unwrap_or_else(|| truncate(&function.contract.address.to_string(), 12));
            format!("{contract}.{}", function.function)
        }
        Decoded::SignatureFallback(fallback) => fallback
            .candidates
            .first()
            .cloned()
            .unwrap_or_else(|| format!("selector {}", fallback.selector)),
        Decoded::Raw(raw) => {
            if raw.data.0.is_empty() {
                "transfer".to_string()
            } else {
                format!("raw call ({} bytes)", raw.data.0.len())
            }
        }
    }
}

fn push_decoded_call_lines(
    lines: &mut Vec<Line<'static>>,
    call: &koi::models::tx::decode::DecodedCall,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    let value = call.value.to_string();
    let suffix = if value != "0" {
        format!(" · value {value}")
    } else {
        String::new()
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("{indent}› ",),
            Style::default().fg(Color::DarkGray).bg(PANEL_BG),
        ),
        Span::styled(
            format!("{}{suffix}", decoded_call_label(call)),
            Style::default().bg(PANEL_BG),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        format!("{indent}  to {}", call.to),
        Style::default().fg(Color::DarkGray).bg(PANEL_BG),
    )));

    if let koi::models::tx::decode::Decoded::Verified(function) = &call.decoded {
        for param in &function.params {
            let name = param.name.clone().unwrap_or_else(|| param.ty.clone());
            let value = serde_json::to_string(&param.value).unwrap_or_default();
            lines.push(Line::from(Span::styled(
                format!("{indent}  {name}: {}", truncate(&value, 60)),
                Style::default().fg(Color::DarkGray).bg(PANEL_BG),
            )));
        }
    }

    for subcall in &call.subcalls {
        push_decoded_call_lines(lines, subcall, depth + 1);
    }
}

fn render_tx_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(tx) = app
        .selected_account_txs()
        .and_then(|transactions| transactions.get(app.clamped_tx_index()))
    else {
        return;
    };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                "Network  ",
                Style::default().fg(Color::DarkGray).bg(PANEL_BG),
            ),
            Span::styled(
                tx.network_identity.0.to_string(),
                Style::default().bg(PANEL_BG),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Status   ",
                Style::default().fg(Color::DarkGray).bg(PANEL_BG),
            ),
            Span::styled(transaction_status(tx), Style::default().bg(PANEL_BG)),
        ]),
        Line::from(vec![
            Span::styled(
                "Date     ",
                Style::default().fg(Color::DarkGray).bg(PANEL_BG),
            ),
            Span::styled(transaction_date(tx), Style::default().bg(PANEL_BG)),
        ]),
        Line::from(vec![
            Span::styled(
                "Hash     ",
                Style::default().fg(Color::DarkGray).bg(PANEL_BG),
            ),
            Span::styled(
                tx.tx_hash
                    .as_ref()
                    .map(|hash| hash.to_string())
                    .unwrap_or_else(|| "—".to_string()),
                Style::default().bg(PANEL_BG),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "From     ",
                Style::default().fg(Color::DarkGray).bg(PANEL_BG),
            ),
            Span::styled(
                tx.from
                    .as_ref()
                    .map(|from| from.to_string())
                    .unwrap_or_else(|| "—".to_string()),
                Style::default().bg(PANEL_BG),
            ),
        ]),
        Line::from(Span::raw("")),
    ];

    match &tx.decoded {
        Some(call) => push_decoded_call_lines(&mut lines, call, 0),
        None => lines.push(Line::from(Span::styled(
            "No decoded call data",
            Style::default().fg(Color::DarkGray).bg(PANEL_BG),
        ))),
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "Esc close",
        Style::default().fg(Color::DarkGray).bg(PANEL_BG),
    )));

    let height = (lines.len() + 2).min(28) as u16;
    let popup = centered_modal(area, 86, height);
    paint_area(frame, popup, PANEL_BG);
    render_panel(frame, popup, "Transaction", lines);
}

fn asset_rows(app: &App, balances: &AccountBalances) -> Vec<Row<'static>> {
    let mut entries: Vec<(u128, String, Line<'static>, String, String, Style, Style)> = balances
        .balances
        .iter()
        .map(|balance| {
            let asset = app.assets.get(&balance.asset_identity.to_string());
            let name = asset
                .map(|asset| asset.asset_name.clone())
                .unwrap_or_else(|| balance.asset_identity.to_string());
            let symbol = asset
                .map(|asset| asset.asset_symbol.clone())
                .unwrap_or_else(|| "?".to_string());

            let amount = balance
                .balance
                .as_ref()
                .map(|value| {
                    format_token(value, asset.map(|asset| asset.asset_decimals).unwrap_or(18))
                })
                .unwrap_or(DisplayAmount {
                    text: "—".to_string(),
                    style: super::format::AmountStyle::Normal,
                });

            let value = balance
                .balance_quote
                .as_ref()
                .map(|value| format_quote(value, app.display_asset()))
                .unwrap_or(DisplayAmount {
                    text: "—".to_string(),
                    style: super::format::AmountStyle::Normal,
                });

            let change = match (
                balance.asset_quote.as_deref(),
                balance.asset_24h_quote.as_deref(),
            ) {
                (Some(current), Some(previous)) => percent_change(current, previous)
                    .map(|change| (change.label(), change.ratatui_style())),
                _ => None,
            };

            let (change_text, change_style) =
                change.unwrap_or_else(|| ("—".to_string(), Style::default().fg(Color::DarkGray)));

            let sort_value = balance
                .balance_quote
                .as_ref()
                .and_then(|raw| raw.parse::<u128>().ok())
                .unwrap_or(0);

            let amount_style = amount.ratatui_style();
            let mut balance_spans = vec![Span::styled(format!("{} ", amount.text), amount_style)];
            if app.colored_assets
                && let Some(icon) = app.asset_icons.get(&balance.asset_identity.to_string())
            {
                balance_spans.extend(IconRenderer::color_symbol(&symbol, &icon.colors));
            } else {
                balance_spans.push(Span::styled(symbol, amount_style));
            }

            (
                sort_value,
                name,
                Line::from(balance_spans),
                value.text.clone(),
                change_text,
                change_style,
                value.ratatui_style(),
            )
        })
        .collect();

    entries.sort_by(|(left, _, _, _, _, _, _), (right, _, _, _, _, _, _)| right.cmp(left));

    if entries.is_empty() {
        return vec![Row::new(vec![
            Cell::from("No assets with balance"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])];
    }

    let total: u128 = balances
        .total_quote
        .as_ref()
        .and_then(|raw| raw.parse::<u128>().ok())
        .unwrap_or(0);
    let weight = |value: u128| {
        if total == 0 {
            "—".to_string()
        } else {
            format!("{:.1}%", (value as f64 / total as f64) * 100.0)
        }
    };

    let mut rows: Vec<Row<'static>> = entries
        .into_iter()
        .map(
            |(sort_value, name, balance_line, value, change, change_style, value_style)| {
                Row::new(vec![
                    Cell::from(format!("    {name}")),
                    Cell::from(balance_line),
                    Cell::from(value).style(value_style),
                    Cell::from(weight(sort_value)).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(change).style(change_style),
                ])
            },
        )
        .collect();

    if let Some(total_quote) = balances.total_quote.as_ref() {
        let formatted = format_quote(total_quote, app.display_asset());
        rows.push(
            Row::new(vec![
                Cell::from("Total"),
                Cell::from(""),
                Cell::from(formatted.text),
                Cell::from("100%"),
                Cell::from(""),
            ])
            .style(Style::default().add_modifier(Modifier::BOLD)),
        );
    }

    rows
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let colors = palette(app.theme);
    paint_area(frame, area, colors.chrome);
    let mode = if app.move_mode.is_some() {
        "MOVE"
    } else if app.filter_input {
        "SEARCH"
    } else if app.form.is_some() {
        "INSERT"
    } else if app.selected_account.is_some() {
        "ACCOUNT"
    } else {
        "NORMAL"
    };
    let keys = if let Some(form) = &app.form {
        if matches!(form, ActiveForm::AddAsset { .. }) {
            "↑↓ field · Tab hint · Enter save · Esc cancel"
        } else {
            "↑↓ field · Enter save · Esc cancel"
        }
    } else if app.move_mode.is_some() {
        "Space grab · j/k move · s save · Esc cancel"
    } else if app.selected_account.is_some() {
        match app.account_panel {
            AccountPanel::Assets => "n link · x unlink · $ currency · r refresh · b back · ? help",
            AccountPanel::Transactions => "→ focus · Enter detail · r refresh · b back · ? help",
            _ => "o/a/d/t panels · h/l focus · $ currency · r refresh · b back · ? help",
        }
    } else if app.tab == Tab::Assets {
        "n add · e edit · x delete · $ currency · r refresh · ? help"
    } else if app.tab == Tab::Prices {
        "n add quoter · x toggle · $ currency · r refresh · ? help"
    } else if app.tab == Tab::Networks {
        if app.settings.nested_network.is_some() {
            "n add · e edit · x delete · b back · ? help"
        } else {
            "Enter endpoints · n add · e edit · x delete · ? help"
        }
    } else if app.tab == Tab::Settings {
        "←→ section · e edit · x toggle · r refresh · ? help"
    } else {
        "n add · g group · m move · e edit · x delete · $ currency · ? help"
    };

    let mode = format!(" {mode} ");
    let mode_width = (mode.chars().count() as u16).min(area.width);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            mode,
            Style::default().fg(colors.selected_fg).bg(colors.selected),
        ))),
        Rect {
            x: area.x,
            y: area.y,
            width: mode_width,
            height: 1,
        },
    );

    let currency = format!(" {} ", app.display_currency);
    let currency_width = (currency.chars().count() as u16).min(area.width);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            currency,
            Style::default().fg(colors.selected_fg).bg(colors.selected),
        )))
        .alignment(ratatui::layout::Alignment::Right),
        Rect {
            x: area
                .x
                .saturating_add(area.width.saturating_sub(currency_width)),
            y: area.y,
            width: currency_width,
            height: 1,
        },
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(keys, Style::default().fg(colors.muted).bg(colors.chrome)),
            Span::raw(" "),
        ]))
        .alignment(ratatui::layout::Alignment::Right),
        Rect {
            x: area.x.saturating_add(mode_width),
            y: area.y,
            width: area
                .width
                .saturating_sub(mode_width.saturating_add(currency_width)),
            height: 1,
        },
    );
}

fn balance_cell(
    app: &App,
    account_id: u64,
    state: Option<&ResourceState<AccountBalances>>,
) -> (String, Style) {
    match state {
        None | Some(ResourceState::Idle) => ("—".to_string(), Style::default()),
        Some(ResourceState::Loading) => ("…".to_string(), Style::default().fg(Color::Yellow)),
        Some(ResourceState::Ready(balances)) => {
            let formatted = balances
                .total_quote
                .as_ref()
                .map(|value| format_quote(value, app.display_asset()))
                .unwrap_or(DisplayAmount {
                    text: "—".to_string(),
                    style: super::format::AmountStyle::Normal,
                });
            let style = if app.is_balance_refreshing(account_id) {
                Style::default().fg(Color::Yellow)
            } else {
                formatted.ratatui_style()
            };
            (formatted.text, style)
        }
        Some(ResourceState::Error(_)) => ("error".to_string(), Style::default().fg(Color::Red)),
    }
}

fn balance_sidebar_style(
    app: &App,
    account_id: u64,
    state: Option<&ResourceState<AccountBalances>>,
) -> Style {
    match state {
        Some(ResourceState::Error(_)) => Style::default().fg(Color::Red),
        Some(ResourceState::Loading) | _ if app.is_balance_refreshing(account_id) => {
            Style::default().fg(Color::Yellow)
        }
        _ => Style::default().fg(Color::Green),
    }
}

fn truncate_error(error: &str) -> String {
    if error.len() <= 48 {
        error.to_string()
    } else {
        format!("{}…", &error[..45])
    }
}

fn account_icons_enabled(app: &App) -> bool {
    app.icon_renderer
        .as_ref()
        .is_some_and(IconRenderer::uses_graphics)
}

fn account_list_icon_rect(table_area: Rect, visible_row: usize, row_height: u16) -> Rect {
    Rect {
        x: table_area.x,
        y: table_area.y + 1 + visible_row as u16 * row_height,
        width: IconRenderer::list_column_width(),
        height: row_height,
    }
}

fn render_account_icon(frame: &mut Frame, app: &mut App, area: Rect, address: &str) {
    if let Some(renderer) = app.icon_renderer.as_mut() {
        renderer.render_large(frame, area, address);
    }
}

fn render_asset_table_icons(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    identities: &[String],
    scroll: usize,
) {
    render_asset_table_icons_at(frame, app, area, identities, scroll, 0);
}

fn render_asset_table_icons_at(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    identities: &[String],
    scroll: usize,
    offset_x: u16,
) {
    let height = table_body_height(area);
    let (_, end) = visible_window(identities.len(), scroll, height);
    let icons: Vec<_> = identities
        .iter()
        .enumerate()
        .skip(scroll)
        .take(end.saturating_sub(scroll))
        .filter_map(|(row, identity)| {
            app.asset_icons
                .get(identity)
                .map(|icon| (row.saturating_sub(scroll), identity.clone(), icon.clone()))
        })
        .collect();
    for (row, identity, icon) in icons {
        let icon_area = Rect {
            x: area.x.saturating_add(offset_x),
            y: area.y + 1 + row as u16,
            width: 3,
            height: 1,
        };
        if let Some(renderer) = app.icon_renderer.as_mut()
            && renderer.uses_graphics()
        {
            renderer.render_asset_icon(frame, icon_area, &identity, &icon.png_data);
        } else {
            frame.render_widget(
                Paragraph::new(Line::from(IconRenderer::asset_symbol_spans(&icon.colors))),
                icon_area,
            );
        }
    }
}

fn account_evm_address(metadata: &WalletType) -> Option<String> {
    metadata.unwrap_address().map(|address| address.to_string())
}

fn wallet_type_label(metadata: &WalletType) -> String {
    match metadata {
        WalletType::Safe(_) => "safe".to_string(),
        WalletType::EOA(_) => "eoa".to_string(),
        WalletType::View(_) => "view".to_string(),
        WalletType::Railgun(_) => "railgun".to_string(),
    }
}

fn truncate_address(metadata: &WalletType) -> String {
    if let Some(address) = metadata.unwrap_address() {
        let address = address.to_string();
        if address.len() <= 12 {
            address
        } else {
            format!("{}…{}", &address[..6], &address[address.len() - 4..])
        }
    } else if let WalletType::Railgun(wallet) = metadata {
        wallet.railgun_address.chars().take(16).collect()
    } else {
        "—".to_string()
    }
}

fn register_resource_list_table(app: &mut App, area: Rect, len: usize) {
    app.layout.list_table = Some(ListTableLayout {
        area,
        scroll: app.settings.row_scroll,
        len,
        row_height: 1,
    });
}

fn register_account_sidebar_panels(app: &mut App, sidebar: Rect, text_area: Rect) {
    let panel_start = text_area.y + 2;
    let panels = [
        AccountPanel::Overview,
        AccountPanel::Assets,
        AccountPanel::Defi,
        AccountPanel::Transactions,
    ];

    app.layout.account_sidebar = Some(AccountSidebarLayout {
        area: sidebar,
        panel_rows: panels.map(|panel| {
            let index = match panel {
                AccountPanel::Overview => 0,
                AccountPanel::Assets => 1,
                AccountPanel::Defi => 2,
                AccountPanel::Transactions => 3,
            };
            (
                Rect {
                    x: text_area.x,
                    y: panel_start + index,
                    width: text_area.width,
                    height: 1,
                },
                panel,
            )
        }),
    });
}

fn selected_row_style(theme: Theme, selected: bool) -> Style {
    if selected {
        let colors = palette(theme);
        Style::default().bg(colors.selected).fg(colors.selected_fg)
    } else {
        Style::default()
    }
}

const PANEL_BG: Color = Color::Rgb(36, 36, 36);
const PANEL_SELECTED_BG: Color = Color::Rgb(58, 58, 58);
const HINT_FG: Color = Color::Rgb(110, 110, 110);
const HINT_ACCEPT_FG: Color = Color::Rgb(80, 90, 80);

fn panel_style() -> Style {
    Style::default().bg(PANEL_BG).fg(Color::White)
}

fn panel_row_style(selected: bool) -> Style {
    if selected {
        Style::default().bg(PANEL_SELECTED_BG).fg(Color::White)
    } else {
        panel_style()
    }
}

fn paint_area(frame: &mut Frame, area: Rect, bg: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let buf = frame.buffer_mut();
    for y in area.y..area.y.saturating_add(area.height) {
        for x in area.x..area.x.saturating_add(area.width) {
            buf[(x, y)].set_char(' ').set_bg(bg).set_fg(bg);
        }
    }
}

fn centered_modal(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(2)).max(32);
    let height = height.min(area.height.saturating_sub(2)).max(6);
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

fn render_form(frame: &mut Frame, form: &ActiveForm, area: Rect, theme: Theme) {
    let (width, height) = form.modal_dimensions();
    let popup = centered_modal(area, width, height);

    if popup.x.saturating_add(1) < area.width && popup.y.saturating_add(1) < area.height {
        let shadow = Rect {
            x: popup.x.saturating_add(1),
            y: popup.y.saturating_add(1),
            width: popup.width,
            height: popup.height,
        };
        paint_area(frame, shadow, palette(theme).overlay);
    }

    paint_area(frame, popup, palette(theme).panel);

    match form {
        ActiveForm::AddAssetType { selected } => {
            let lines: Vec<Line> = AssetType::ALL
                .iter()
                .enumerate()
                .map(|(index, asset_type)| {
                    Line::from(Span::styled(
                        format!(
                            "{} {}",
                            if index == *selected { "›" } else { " " },
                            asset_type.label()
                        ),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter choose · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::AddNetworkMode { selected } => {
            let options = ["Create new network", "Add from preset"];
            let lines: Vec<Line> = options
                .iter()
                .enumerate()
                .map(|(index, label)| {
                    Line::from(Span::styled(
                        format!("{} {}", if index == *selected { "›" } else { " " }, label),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter choose · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::AddNetworkPreset { presets, selected } => {
            let lines: Vec<Line> = presets
                .iter()
                .enumerate()
                .map(|(index, preset)| {
                    Line::from(Span::styled(
                        format!(
                            "{} {} ({})",
                            if index == *selected { "›" } else { " " },
                            preset.network_name,
                            preset.network_identity.0
                        ),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter add preset · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::AddAsset {
            asset_type,
            form: text_form,
            touched,
            hints,
            discovery,
            ..
        } => render_add_asset_form(
            frame,
            popup,
            *asset_type,
            text_form,
            touched,
            hints,
            discovery,
        ),
        ActiveForm::AddNetwork(text_form) => {
            render_text_form(frame, popup, form.title(), text_form, Vec::new());
        }
        ActiveForm::AddEndpoint {
            form: text_form, ..
        } => {
            render_text_form(frame, popup, form.title(), text_form, Vec::new());
        }
        ActiveForm::GroupName {
            form: text_form, ..
        } => {
            render_text_form(frame, popup, form.title(), text_form, Vec::new());
        }
        ActiveForm::PickCurrency { options, selected } => {
            let lines: Vec<Line> = options
                .iter()
                .enumerate()
                .map(|(index, asset)| {
                    Line::from(Span::styled(
                        format!(
                            "{} {} ({})",
                            if index == *selected { "›" } else { " " },
                            asset.asset_name,
                            asset.asset_identity
                        ),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter choose · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::AddAccountType { selected } => {
            let lines: Vec<Line> = AccountKind::ALL
                .iter()
                .enumerate()
                .map(|(index, kind)| {
                    Line::from(Span::styled(
                        format!(
                            "{} {}",
                            if index == *selected { "›" } else { " " },
                            kind.label()
                        ),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter choose · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::AddAccountAddress {
            form: text_form,
            ens_hint,
            ens_primary,
            ..
        } => {
            let mut extras = Vec::new();
            if let Some(hint) = ens_hint {
                extras.push(Line::from(vec![
                    Span::styled(
                        format!("  ↳ {hint}"),
                        Style::default().fg(Color::Green).bg(PANEL_BG),
                    ),
                    Span::styled(
                        "  Tab accept",
                        Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                    ),
                ]));
            }
            if let Some(primary) = ens_primary {
                let mut spans = vec![Span::styled(
                    format!("  ★ {primary}"),
                    Style::default()
                        .fg(Color::Cyan)
                        .bg(PANEL_BG)
                        .add_modifier(Modifier::BOLD),
                )];
                let name_empty = text_form
                    .field(0)
                    .is_none_or(|value| value.trim().is_empty());
                if name_empty {
                    spans.push(Span::styled(
                        "  Tab on Name to use",
                        Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                    ));
                }
                extras.push(Line::from(spans));
            }
            render_text_form(frame, popup, form.title(), text_form, extras);
        }
        ActiveForm::AddAccountMnemonic {
            form: text_form,
            deriving,
        } => {
            if *deriving {
                render_panel(
                    frame,
                    popup,
                    form.title(),
                    vec![
                        Line::from(Span::styled(
                            "Deriving addresses…",
                            Style::default().fg(Color::Yellow).bg(PANEL_BG),
                        )),
                        Line::from(Span::raw("")),
                        Line::from(Span::styled(
                            "Esc cancel",
                            Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                        )),
                    ],
                );
            } else {
                render_text_form(frame, popup, form.title(), text_form, Vec::new());
            }
        }
        ActiveForm::AddAccountPickAddress {
            options, selected, ..
        } => {
            let lines: Vec<Line> = options
                .iter()
                .enumerate()
                .map(|(index, option)| {
                    Line::from(Span::styled(
                        format!(
                            "{} {} · {}",
                            if index == *selected { "›" } else { " " },
                            option.path,
                            option.address
                        ),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter create account · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::RenameAccount {
            form: text_form, ..
        } => {
            render_text_form(frame, popup, form.title(), text_form, Vec::new());
        }
        ActiveForm::ConfirmDeleteAccount { name, .. } => {
            let lines = vec![
                Line::from(Span::styled(
                    format!("Delete account \"{name}\"?"),
                    Style::default().bg(PANEL_BG),
                )),
                Line::from(Span::styled(
                    "Balances and history references are removed.",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                )),
                Line::from(Span::raw("")),
                Line::from(Span::styled(
                    "Enter delete · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                )),
            ];
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::AccountNetworks {
            options, selected, ..
        } => {
            let lines: Vec<Line> = options
                .iter()
                .enumerate()
                .map(|(index, (id, name, checked))| {
                    Line::from(Span::styled(
                        format!(
                            "{} [{}] {} ({})",
                            if index == *selected { "›" } else { " " },
                            if *checked { "x" } else { " " },
                            name,
                            id
                        ),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Space toggle · Enter save · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::PickAccountAsset {
            options, selected, ..
        } => {
            let lines: Vec<Line> = options
                .iter()
                .enumerate()
                .map(|(index, (_, label))| {
                    Line::from(Span::styled(
                        format!("{} {}", if index == *selected { "›" } else { " " }, label),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter apply · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::EditEndpoint {
            form: text_form, ..
        }
        | ActiveForm::EditNetwork {
            form: text_form, ..
        }
        | ActiveForm::EditAsset {
            form: text_form, ..
        } => {
            render_text_form(frame, popup, form.title(), text_form, Vec::new());
        }
        ActiveForm::PickQuoterToken {
            options, selected, ..
        } => {
            let lines: Vec<Line> = options
                .iter()
                .enumerate()
                .map(|(index, (_, label))| {
                    Line::from(Span::styled(
                        format!("{} {}", if index == *selected { "›" } else { " " }, label),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter choose · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::QuoterDiscovering => {
            render_panel(
                frame,
                popup,
                form.title(),
                vec![
                    Line::from(Span::styled(
                        "Searching Uniswap pairs/pools and ERC-4626 vaults…",
                        Style::default().fg(Color::Yellow).bg(PANEL_BG),
                    )),
                    Line::from(Span::raw("")),
                    Line::from(Span::styled(
                        "Esc cancel",
                        Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                    )),
                ],
            );
        }
        ActiveForm::PickQuoterSource {
            options, selected, ..
        } => {
            let lines: Vec<Line> = options
                .iter()
                .enumerate()
                .map(|(index, option)| {
                    Line::from(Span::styled(
                        format!(
                            "{} {}",
                            if index == *selected { "›" } else { " " },
                            option.label
                        ),
                        panel_row_style(index == *selected),
                    ))
                })
                .chain(std::iter::once(Line::from(Span::raw(""))))
                .chain(std::iter::once(Line::from(Span::styled(
                    "Enter create · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                ))))
                .collect();
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::ConfirmDeleteNetwork { name, .. } => {
            let lines = vec![
                Line::from(Span::styled(
                    format!("Delete network \"{name}\"?"),
                    Style::default().bg(PANEL_BG),
                )),
                Line::from(Span::styled(
                    "Its endpoints are removed as well.",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                )),
                Line::from(Span::raw("")),
                Line::from(Span::styled(
                    "Enter delete · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                )),
            ];
            render_panel(frame, popup, form.title(), lines);
        }
        ActiveForm::ConfirmDeleteGroup { name, .. } => {
            let lines = vec![
                Line::from(Span::styled(
                    format!("Delete group \"{name}\"?"),
                    Style::default().bg(PANEL_BG),
                )),
                Line::from(Span::styled(
                    "Its accounts will become ungrouped.",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                )),
                Line::from(Span::raw("")),
                Line::from(Span::styled(
                    "Enter delete · Esc cancel",
                    Style::default().fg(Color::DarkGray).bg(PANEL_BG),
                )),
            ];
            render_panel(frame, popup, form.title(), lines);
        }
    }
}

fn render_panel(frame: &mut Frame, area: Rect, title: &str, lines: Vec<Line>) {
    frame.render_widget(
        Paragraph::new(lines)
            .style(panel_style())
            .block(panel_block(title)),
        area,
    );
}

fn panel_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(panel_style())
        .style(panel_style())
        .title(format!(" {title} "))
}

fn render_add_asset_form(
    frame: &mut Frame,
    area: Rect,
    asset_type: AssetType,
    form: &TextForm,
    touched: &[bool],
    hints: &super::form::AssetFieldHints,
    discovery: &DiscoveryState,
) {
    let mut lines = Vec::new();

    for (index, field) in form.fields.iter().enumerate() {
        lines.push(asset_field_line(
            index == form.focus,
            &field.label,
            field.required,
            &field.value,
            touched.get(index).copied().unwrap_or(false),
            hints.hint_display(asset_type, index),
        ));
    }

    if discovery.is_loading() {
        lines.push(Line::from(Span::styled(
            "  discovering metadata…",
            Style::default().fg(HINT_FG).bg(PANEL_BG),
        )));
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "↑↓ field · Tab accept hint · Enter save · Esc cancel",
        Style::default().fg(Color::DarkGray).bg(PANEL_BG),
    )));

    render_panel(frame, area, &form.title, lines);
}

fn asset_field_line(
    focused: bool,
    label: &str,
    required: bool,
    value: &str,
    touched: bool,
    hint: Option<String>,
) -> Line<'static> {
    let label = if required {
        format!("{label} *")
    } else {
        label.to_string()
    };
    let marker = if focused { "›" } else { " " };
    let label_style = if focused {
        Style::default().fg(Color::Cyan).bg(PANEL_BG)
    } else {
        panel_style()
    };

    let mut spans = vec![
        Span::styled(format!("{marker} "), label_style),
        Span::styled(format!("{label}: "), label_style),
    ];

    if value.is_empty() {
        if !touched {
            if let Some(hint) = hint {
                spans.push(Span::styled(
                    hint,
                    Style::default().fg(HINT_FG).bg(PANEL_BG),
                ));
                if focused {
                    spans.push(Span::styled(
                        "  tab",
                        Style::default().fg(HINT_ACCEPT_FG).bg(PANEL_BG),
                    ));
                }
            } else {
                spans.push(Span::styled("_", label_style));
            }
        } else {
            spans.push(Span::styled("_", label_style));
        }
    } else {
        spans.push(Span::styled(value.to_string(), label_style));
    }

    Line::from(spans)
}

fn render_text_form(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    form: &TextForm,
    extras: Vec<Line<'static>>,
) {
    let mut lines = Vec::new();
    for (index, field) in form.fields.iter().enumerate() {
        let focused = index == form.focus;
        let label = if field.required {
            format!("{} *", field.label)
        } else {
            field.label.clone()
        };
        let value = if field.value.is_empty() {
            "_".to_string()
        } else {
            field.value.clone()
        };
        lines.push(Line::from(Span::styled(
            format!("{} {}: {}", if focused { "›" } else { " " }, label, value),
            if focused {
                Style::default().fg(Color::Cyan).bg(PANEL_BG)
            } else {
                panel_style()
            },
        )));
    }
    lines.extend(extras);
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "↑↓ field · Enter save · Esc cancel",
        Style::default().fg(Color::DarkGray).bg(PANEL_BG),
    )));

    frame.render_widget(
        Paragraph::new(lines).style(panel_style()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(panel_style())
                .style(panel_style())
                .title(format!(" {title} ")),
        ),
        area,
    );
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let mut output = value
        .chars()
        .take(max.saturating_sub(1))
        .collect::<String>();
    output.push('…');
    output
}

fn format_fiat(value: f64) -> String {
    if value == 0.0 {
        "—".to_string()
    } else if value.abs() < 0.01 {
        "<$0.01".to_string()
    } else {
        format!("${}", format_float(value, 2))
    }
}

fn format_float(value: f64, decimals: usize) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let value = value.abs();
    let formatted = format!("{value:.decimals$}");
    let Some((whole, fraction)) = formatted.split_once('.') else {
        return format!("{sign}{}", add_commas(&formatted));
    };
    let fraction = fraction.trim_end_matches('0');
    if fraction.is_empty() {
        format!("{sign}{}", add_commas(whole))
    } else {
        format!("{sign}{}.{fraction}", add_commas(whole))
    }
}

fn add_commas(value: &str) -> String {
    let mut out = String::new();
    let chars = value.chars().collect::<Vec<_>>();
    for (index, ch) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(*ch);
    }
    out
}

fn format_percent(value: f64) -> String {
    if value == 0.0 {
        "—".to_string()
    } else {
        format!("{:.2}%", value * 100.0)
    }
}

fn transaction_nonce(tx: &Tx) -> String {
    tx.extra
        .safe_wallet
        .as_ref()
        .and_then(|safe| safe.nonce)
        .map(|nonce| format!("#{nonce}"))
        .unwrap_or_else(|| "—".to_string())
}

fn transaction_status(tx: &Tx) -> String {
    let Some(safe) = &tx.extra.safe_wallet else {
        return "Unknown".to_string();
    };

    let successful = safe
        .is_successful
        .or_else(|| safe_bool_extra(safe, "isSuccessful"));
    let executed = safe
        .is_executed
        .or_else(|| safe_bool_extra(safe, "isExecuted"));

    match (successful, executed) {
        (Some(true), _) => "Success",
        (Some(false), _) => "Failed",
        (_, Some(true)) => "Executed",
        (_, Some(false)) => "Queued",
        _ => "Unknown",
    }
    .to_string()
}

fn transaction_date(tx: &Tx) -> String {
    tx.extra
        .safe_wallet
        .as_ref()
        .and_then(|safe| {
            safe.execution_date
                .clone()
                .or_else(|| safe_string_extra(safe, "executionDate"))
                .or_else(|| safe_string_extra(safe, "submissionDate"))
        })
        .map(|date| date.replace('T', " ").trim_end_matches('Z').to_string())
        .map(|date| truncate(&date, 18))
        .unwrap_or_else(|| "—".to_string())
}

fn transaction_action(tx: &Tx) -> String {
    let Some(decoded) = &tx.decoded else {
        return "Unknown action".to_string();
    };

    let prefix = if decoded.subcalls.is_empty() {
        None
    } else {
        Some(format!("{} calls via ", decoded.subcalls.len()))
    };

    let action = match &decoded.decoded {
        Decoded::Verified(function) => function.function.clone(),
        Decoded::SignatureFallback(fallback) => fallback.selector.to_string(),
        Decoded::Raw(_) => "Raw call".to_string(),
    };

    match prefix {
        Some(prefix) => format!("{prefix}{action}"),
        None => action,
    }
}

fn transaction_target(tx: &Tx) -> String {
    if let Some(decoded) = &tx.decoded {
        match &decoded.decoded {
            Decoded::Verified(function) => {
                return function
                    .contract
                    .verified_name
                    .clone()
                    .unwrap_or_else(|| short_hash(&function.contract.address.to_string()));
            }
            Decoded::SignatureFallback(fallback) => {
                if let Some(contract) = &fallback.contract {
                    return contract
                        .verified_name
                        .clone()
                        .unwrap_or_else(|| short_hash(&contract.address.to_string()));
                }
            }
            Decoded::Raw(_) => {}
        }
    }

    tx.to
        .map(|address| short_hash(&address.to_string()))
        .unwrap_or_else(|| "—".to_string())
}

fn transaction_hash(tx: &Tx) -> String {
    tx.tx_hash
        .as_ref()
        .map(|hash| short_hash(&hash.to_string()))
        .or_else(|| {
            tx.extra
                .safe_wallet
                .as_ref()
                .and_then(|safe| safe.safe_tx_hash.as_ref())
                .map(|hash| short_hash(&hash.to_string()))
        })
        .unwrap_or_else(|| "—".to_string())
}

fn safe_bool_extra(safe: &koi::models::tx::SafeWalletTxExtra, key: &str) -> Option<bool> {
    safe.extra.get(key).and_then(|value| value.as_bool())
}

fn safe_string_extra(safe: &koi::models::tx::SafeWalletTxExtra, key: &str) -> Option<String> {
    safe.extra
        .get(key)
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn short_hash(value: &str) -> String {
    if value.len() <= 14 {
        value.to_string()
    } else {
        format!(
            "{}…{}",
            &value[..10],
            &value[value.len().saturating_sub(4)..]
        )
    }
}
