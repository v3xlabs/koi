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
    app::{AccountFocus, AccountPanel, App, ResourceState, Tab},
    defi::DefiResult,
    form::{ActiveForm, AssetType, DiscoveryState, TextForm},
    format::{DisplayAmount, format_token, format_usd, percent_change},
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
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
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

    if let Some(form) = &app.form {
        render_form(frame, form, frame.area());
    }
}

fn render_top_bar(frame: &mut Frame, app: &mut App, area: Rect) {
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
        " koi ",
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )];
    nav_spans.push(Span::raw(" "));
    for (key, label, tab) in nav {
        let selected = app.tab == tab && app.selected_account.is_none();
        let style = if selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        nav_spans.push(Span::styled(format!(" {key}:{label} "), style));
    }

    let status_text = format!("{} {}", if app.connected { "●" } else { "○" }, status);
    let status_width = status_text.chars().count().saturating_add(2) as u16;
    let nav_width = area.width.saturating_sub(status_width).max(1);

    app.layout.nav = Rect {
        x: area.x,
        y: area.y,
        width: nav_width,
        height: area.height,
    };

    frame.render_widget(
        Paragraph::new(Line::from(nav_spans))
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL)),
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
        Span::raw(status),
    ]);

    frame.render_widget(
        Paragraph::new(status_line)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL)),
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
    render_settings_price_feeds(frame, app, area);
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
    let (start, end) = visible_window(app.accounts.len(), app.list_scroll, height);
    let rows: Vec<Row> = if app.accounts.is_empty() {
        if show_icons {
            vec![Row::new(vec![
                Cell::from("No accounts yet"),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ])]
        } else {
            vec![Row::new(vec![
                Cell::from("No accounts yet"),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ])]
        }
    } else {
        app.accounts
            .iter()
            .enumerate()
            .skip(start)
            .take(end.saturating_sub(start))
            .map(|(index, account)| {
                let (balance_text, balance_style) = balance_cell(
                    app,
                    account.account_identity.0,
                    app.balance_state(account.account_identity.0),
                );

                let row_style = if index == app.list_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };

                let mut cells = Vec::new();
                if show_icons {
                    cells.push(Cell::from(""));
                }
                cells.extend([
                    Cell::from(format!(
                        "{} {}",
                        if index == app.list_index { "›" } else { " " },
                        account.name
                    ))
                    .style(row_style),
                    Cell::from(wallet_type_label(&account.metadata)).style(row_style),
                    Cell::from(truncate_address(&account.metadata)).style(row_style),
                    Cell::from(balance_text).style(row_style.patch(balance_style)),
                ]);

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
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Accounts {} ",
            position_label(app.accounts.len(), app.list_index)
        )))
        .column_spacing(2);

    frame.render_widget(table, area);

    if show_icons {
        if let Some(renderer) = app.icon_renderer.as_mut() {
            for (visible_row, account) in app
                .accounts
                .iter()
                .skip(start)
                .take(end.saturating_sub(start))
                .enumerate()
            {
                let Some(address) = account_evm_address(&account.metadata) else {
                    continue;
                };
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
        len: app.accounts.len(),
        row_height,
    });
}

fn render_selected_account_preview(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(account) = app.accounts.get(app.list_index) else {
        frame.render_widget(
            Paragraph::new("No account selected")
                .block(Block::default().borders(Borders::ALL).title(" Preview ")),
            area,
        );
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

    let block = Block::default().borders(Borders::ALL).title(" Preview ");
    let inner = block.inner(area);
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

    frame.render_widget(block, area);
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

    frame.render_widget(
        Paragraph::new(section_line).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Settings sections · ←/→ "),
        ),
        chunks[0],
    );

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
            "fiat:usd",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("The TUI currently requests balances in fiat:usd."),
        Line::from("A persistent display-currency backend setting does not exist yet."),
    ];

    if let Some(notice) = &app.settings.notice {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            notice.clone(),
            Style::default().fg(Color::Yellow),
        )));
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" General settings "),
        ),
        area,
    );
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
                let style = selected_row_style(selected);
                let endpoint_count = app
                    .settings_snapshot()
                    .and_then(|snapshot| snapshot.endpoints.get(&network.network_identity.0))
                    .map(|endpoints| endpoints.len())
                    .unwrap_or_default();

                Row::new(vec![
                    Cell::from(format!(
                        "{} {}",
                        if selected { "›" } else { " " },
                        network.network_name
                    )),
                    Cell::from(format!("{} endpoint(s)", endpoint_count)),
                ])
                .style(style)
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [Constraint::Percentage(55), Constraint::Percentage(45)],
    )
    .header(
        Row::new(vec!["Network", "RPC endpoints"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Networks · Enter endpoints · n add {} ",
        position_label(app.networks.len(), app.settings.row_index)
    )))
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
                    let style = selected_row_style(selected);
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
                        Cell::from(if endpoint.endpoint_disabled {
                            "disabled"
                        } else {
                            "enabled"
                        }),
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
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Endpoints for network {network_id} · n add · x delete · b back {} ",
        position_label(
            endpoints.map(|items| items.len()).unwrap_or(0),
            app.settings.row_index,
        )
    )))
    .column_spacing(2);

    frame.render_widget(table, area);
    register_resource_list_table(app, area, endpoints.map(|items| items.len()).unwrap_or(0));
}

fn render_settings_assets(frame: &mut Frame, app: &mut App, area: Rect) {
    let identities = app.settings_asset_identities();

    let rows = if identities.is_empty() {
        vec![Row::new(vec![
            Cell::from("No assets configured"),
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
                Some(
                    Row::new(vec![
                        Cell::from(format!(
                            "{} {}",
                            if selected { "›" } else { " " },
                            asset.asset_symbol
                        )),
                        Cell::from(truncate(&asset.asset_name, 26)),
                        Cell::from(asset.asset_decimals.to_string()),
                        Cell::from(truncate(&asset.asset_identity.to_string(), 42)),
                    ])
                    .style(selected_row_style(selected)),
                )
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Percentage(28),
            Constraint::Length(8),
            Constraint::Min(24),
        ],
    )
    .header(
        Row::new(vec!["Symbol", "Name", "Decimals", "Identity"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Assets · n add · x delete {} ",
        position_label(identities.len(), app.settings.row_index)
    )))
    .column_spacing(2);

    frame.render_widget(table, area);
    register_resource_list_table(app, area, identities.len());
}

fn render_settings_price_feeds(frame: &mut Frame, app: &mut App, area: Rect) {
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
                    let selected = index == app.settings.row_index;
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
                    .style(selected_row_style(selected))
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
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Price feeds {} ",
        position_label(quoter_count, app.settings.row_index)
    )))
    .column_spacing(2);

    frame.render_widget(table, area);
    register_resource_list_table(app, area, quoter_count);
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
                    .style(selected_row_style(selected))
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

    let title = app
        .settings_snapshot()
        .map(|snapshot| {
            format!(
                " Vendors · {} enabled · e/x toggle {} ",
                snapshot.enabled_vendor_count(),
                position_label(snapshot.all_vendors.len(), app.settings.row_index)
            )
        })
        .unwrap_or_else(|| " Vendors ".to_string());

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
    .block(Block::default().borders(Borders::ALL).title(title))
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
        (AccountPanel::Overview, "h", "Overview"),
        (AccountPanel::Assets, "a", "Assets"),
        (AccountPanel::Defi, "d", "DeFi"),
        (AccountPanel::Transactions, "t", "Tx"),
    ];

    let mut sidebar_lines = vec![
        Line::from(Span::styled(
            truncate_address(&account.metadata),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            wallet_type_label(&account.metadata),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(vec![
            Span::styled("Balance ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                balance_label,
                balance_amount_style.patch(balance_sidebar_style(
                    app,
                    account_id,
                    app.balance_state(account_id),
                )),
            ),
        ]),
        Line::from(""),
    ];

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

    let sidebar_title = truncate(&account.name, 20);
    let sidebar_block = {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {sidebar_title} "));
        if app.account_focus == AccountFocus::Sidebar {
            block = block.border_style(Style::default().fg(Color::Cyan));
        }
        block
    };
    let sidebar_inner = sidebar_block.inner(chunks[0]);

    if let Some(address) = account_evm_address(&account.metadata) {
        if account_icons_enabled(app) {
            let body = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(IconRenderer::icon_height()),
                    Constraint::Min(0),
                ])
                .split(sidebar_inner);
            render_account_icon(frame, app, body[0], &address);
            let text_area = body[1];
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

    frame.render_widget(sidebar_block, chunks[0]);

    let content_title = account_content_title(app, account_id);
    let content = account_content_block(&content_title, app.account_focus == AccountFocus::Content);
    let inner = content.inner(chunks[1]);
    frame.render_widget(content, chunks[1]);

    match app.account_panel {
        AccountPanel::Overview => render_account_overview(frame, app, account_id, inner),
        AccountPanel::Assets => render_account_assets(frame, app, account_id, inner),
        AccountPanel::Defi => render_account_defi(frame, app, account_id, inner),
        AccountPanel::Transactions => render_account_transactions(frame, app, account_id, inner),
    }
}

fn account_content_block(title: &str, focused: bool) -> Block<'_> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "));
    if focused {
        block = block.border_style(Style::default().fg(Color::Cyan));
    }
    block
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

fn render_account_overview(frame: &mut Frame, app: &App, account_identity: u64, area: Rect) {
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
                .map(|value| format_usd(value))
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

fn render_account_assets(frame: &mut Frame, app: &App, account_identity: u64, area: Rect) {
    render_account_asset_table(frame, app, account_identity, area);
}

fn render_account_asset_table(frame: &mut Frame, app: &App, account_identity: u64, area: Rect) {
    let state = app.balance_state(account_identity);

    let table_rows: Vec<Row> = match state {
        None | Some(ResourceState::Idle) => vec![Row::new(vec![
            Cell::from("No balance data yet — press r to refresh"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(ResourceState::Loading) => vec![Row::new(vec![
            Cell::from("Loading balances…"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(ResourceState::Error(error)) => vec![Row::new(vec![
            Cell::from(truncate_error(error)),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(ResourceState::Ready(balances)) => asset_rows(app, balances),
    };

    let table = Table::new(
        table_rows,
        [
            Constraint::Percentage(34),
            Constraint::Percentage(28),
            Constraint::Length(14),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Asset", "Balance", "Value", "24h"]).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::DarkGray),
        ),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_account_defi(frame: &mut Frame, app: &App, account_identity: u64, area: Rect) {
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

    let rows: Vec<Row> = match state {
        None | Some(ResourceState::Idle) => vec![Row::new(vec![
            Cell::from("No DeFi data yet — press d or r to refresh"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(ResourceState::Loading) => vec![Row::new(vec![
            Cell::from("Loading DeFi positions…"),
            Cell::from(""),
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
            Cell::from(""),
        ])],
        Some(ResourceState::Ready(result)) => defi_rows(result),
    };

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
            "Position",
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
        Some(ResourceState::Ready(transactions)) => transaction_rows(transactions),
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
                Cell::from(truncate(&position.name, 32)),
                Cell::from(value),
                Cell::from(format_fiat(position.tvl_usd)),
                Cell::from(format_percent(position.apr)),
                Cell::from(earned),
            ])
        })
        .collect()
}

fn transaction_rows(transactions: &[Tx]) -> Vec<Row<'static>> {
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
        .map(|tx| {
            let status = transaction_status(tx);
            let status_style = match status.as_str() {
                "Success" => Style::default().fg(Color::Green),
                "Failed" => Style::default().fg(Color::Red),
                "Executed" => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::DarkGray),
            };

            Row::new(vec![
                Cell::from(transaction_nonce(tx)),
                Cell::from(status).style(status_style),
                Cell::from(transaction_date(tx)),
                Cell::from(truncate(&transaction_action(tx), 28)),
                Cell::from(truncate(&transaction_target(tx), 26)),
                Cell::from(transaction_hash(tx)),
            ])
        })
        .collect()
}

fn asset_rows(app: &App, balances: &AccountBalances) -> Vec<Row<'static>> {
    let mut entries: Vec<(u128, String, String, String, Style, String, Style, Style)> = balances
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
                .map(|value| format_usd(value))
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

            (
                sort_value,
                name,
                format!("{} {}", amount.text, symbol),
                value.text.clone(),
                amount.ratatui_style(),
                change_text,
                change_style,
                value.ratatui_style(),
            )
        })
        .collect();

    entries.sort_by(|(left, _, _, _, _, _, _, _), (right, _, _, _, _, _, _, _)| right.cmp(left));

    if entries.is_empty() {
        return vec![Row::new(vec![
            Cell::from("No assets with balance"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])];
    }

    entries
        .into_iter()
        .map(
            |(_, name, amount, value, amount_style, change, change_style, value_style)| {
                Row::new(vec![
                    Cell::from(name),
                    Cell::from(amount).style(amount_style),
                    Cell::from(value).style(value_style),
                    Cell::from(change).style(change_style),
                ])
            },
        )
        .collect()
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = if let Some(form) = &app.form {
        if matches!(form, ActiveForm::AddAsset { .. }) {
            "form  ↑↓ field · Tab accept hint · Enter save · Esc cancel"
        } else {
            "form  ↑↓ field · Tab next · Enter save · Esc cancel"
        }
    } else if app.selected_account.is_some() {
        "account  click sidebar pages · scroll · ←→ focus  r refresh  b back  q quit"
    } else if app.tab == Tab::Assets {
        "assets  click/hover rows · scroll · n add  x delete  r reload  q quit"
    } else if app.tab == Tab::Prices {
        "prices  click/hover rows · scroll · r reload  q quit"
    } else if app.tab == Tab::Networks {
        if app.settings.nested_network.is_some() {
            "endpoints  click/hover rows · scroll · n add  x delete  b back  r reload  q quit"
        } else {
            "networks  click/hover rows · scroll · n add  Enter endpoints  r reload  q quit"
        }
    } else if app.tab == Tab::Settings {
        "settings  click tabs/rows · scroll · e/x action  r reload  q quit"
    } else {
        "accounts  click/hover rows · scroll · Enter open  r reload  q quit"
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" command ", Style::default().fg(Color::Cyan)),
            Span::styled(help, Style::default().fg(Color::DarkGray)),
        ])),
        area,
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
                .map(|value| format_usd(value))
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
        x: table_area.x + 1,
        y: table_area.y + 2 + visible_row as u16 * row_height,
        width: IconRenderer::list_column_width(),
        height: row_height,
    }
}

fn render_account_icon(frame: &mut Frame, app: &mut App, area: Rect, address: &str) {
    if let Some(renderer) = app.icon_renderer.as_mut() {
        renderer.render_large(frame, area, address);
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
    let panel_start = text_area.y + 4;
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

fn position_label(len: usize, selected: usize) -> String {
    if len == 0 {
        "(0)".to_string()
    } else {
        format!("({}/{len})", selected.saturating_add(1).min(len))
    }
}

fn selected_row_style(selected: bool) -> Style {
    if selected {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    } else {
        Style::default()
    }
}

const OVERLAY_BG: Color = Color::Rgb(18, 18, 18);
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

fn render_form(frame: &mut Frame, form: &ActiveForm, area: Rect) {
    let (width, height) = form.modal_dimensions();
    let popup = centered_modal(area, width, height);

    if popup.x.saturating_add(1) < area.width && popup.y.saturating_add(1) < area.height {
        let shadow = Rect {
            x: popup.x.saturating_add(1),
            y: popup.y.saturating_add(1),
            width: popup.width,
            height: popup.height,
        };
        paint_area(frame, shadow, OVERLAY_BG);
    }

    paint_area(frame, popup, PANEL_BG);

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
            render_text_form(frame, popup, form.title(), text_form, false);
        }
        ActiveForm::AddEndpoint {
            form: text_form, ..
        } => {
            render_text_form(frame, popup, form.title(), text_form, false);
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
    _with_hints: bool,
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
