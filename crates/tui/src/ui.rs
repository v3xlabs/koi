use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use koi::models::{
    account::{balances::AccountBalances, metadata::WalletType},
    tx::{Tx, decode::Decoded},
};

use super::{
    app::{AccountFocus, AccountPanel, App, ResourceState, Tab},
    defi::DefiResult,
    format::{format_token, format_usd, percent_change, DisplayAmount},
    settings::SettingsSection,
};

pub fn render(frame: &mut Frame, app: &App) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_top_bar(frame, app, vertical[0]);
    render_body(frame, app, vertical[1]);
    render_footer(frame, app, vertical[2]);
}

fn render_top_bar(frame: &mut Frame, app: &App, area: Rect) {
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

    let status_text = format!(
        "{} {}",
        if app.connected { "●" } else { "○" },
        status
    );
    let status_width = status_text.chars().count().saturating_add(2) as u16;
    let nav_width = area.width.saturating_sub(status_width).max(1);

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

fn render_body(frame: &mut Frame, app: &App, area: Rect) {
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

fn render_assets_workspace(frame: &mut Frame, app: &App, area: Rect) {
    render_settings_assets(frame, app, area);
}

fn render_prices_workspace(frame: &mut Frame, app: &App, area: Rect) {
    render_settings_price_feeds(frame, app, area);
}

fn render_networks_workspace(frame: &mut Frame, app: &App, area: Rect) {
    render_settings_networks(frame, app, area);
}

fn render_accounts_home(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_accounts_list(frame, app, chunks[0]);
    render_selected_account_preview(frame, app, chunks[1]);
}

fn render_accounts_list(frame: &mut Frame, app: &App, area: Rect) {
    let (start, end) = visible_range(app.accounts.len(), app.list_index, table_body_height(area));
    let rows: Vec<Row> = if app.accounts.is_empty() {
        vec![Row::new(vec![
            Cell::from("No accounts yet"),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        app.accounts
            .iter()
            .enumerate()
            .skip(start)
            .take(end.saturating_sub(start))
            .map(|(index, account)| {
                let (balance_text, balance_style) =
                    balance_cell(app.balance_state(account.account_identity.0));

                let row_style = if index == app.list_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };

                Row::new(vec![
                    Cell::from(format!(
                        "{} {}",
                        if index == app.list_index { "›" } else { " " },
                        account.name
                    ))
                    .style(row_style),
                    Cell::from(wallet_type_label(&account.metadata)).style(row_style),
                    Cell::from(truncate_address(&account.metadata)).style(row_style),
                    Cell::from(balance_text).style(row_style.patch(balance_style)),
                ])
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Length(8),
            Constraint::Percentage(30),
            Constraint::Length(16),
        ],
    )
    .header(
        Row::new(vec!["Name", "Type", "Address", "Balance"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " Accounts {} ",
                position_label(app.accounts.len(), app.list_index)
            )),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_selected_account_preview(frame: &mut Frame, app: &App, area: Rect) {
    let Some(account) = app.accounts.get(app.list_index) else {
        frame.render_widget(
            Paragraph::new("No account selected").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Preview "),
            ),
            area,
        );
        return;
    };

    let (balance, balance_style) = balance_cell(app.balance_state(account.account_identity.0));
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
        Line::from(Span::styled("Total balance", Style::default().fg(Color::DarkGray))),
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

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title(" Preview ")),
        area,
    );
}

fn render_settings(frame: &mut Frame, app: &App, area: Rect) {
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

fn render_settings_detail(frame: &mut Frame, app: &App, area: Rect) {
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
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" General settings "),
            ),
        area,
    );
}

fn render_settings_networks(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(network_id) = app.settings.nested_network {
        render_settings_endpoints(frame, app, network_id, area);
        return;
    }

    let rows = if app.networks.is_empty() {
        vec![Row::new(vec![Cell::from("No networks configured"), Cell::from("")])]
    } else {
        app.networks
            .iter()
            .enumerate()
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

    let table = Table::new(rows, [Constraint::Percentage(55), Constraint::Percentage(45)])
        .header(
            Row::new(vec!["Network", "RPC endpoints"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Networks · Enter endpoints "),
        )
        .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_settings_endpoints(frame: &mut Frame, app: &App, network_id: u64, area: Rect) {
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
        Some(endpoints) => endpoints
            .iter()
            .enumerate()
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
            .collect(),
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Endpoints for network {network_id} · x delete · b back ")),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_settings_assets(frame: &mut Frame, app: &App, area: Rect) {
    let identities = app.settings_asset_identities();

    let rows = if identities.is_empty() {
        vec![Row::new(vec![
            Cell::from("No assets configured"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        identities
            .iter()
            .enumerate()
            .filter_map(|(index, identity)| {
                let asset = app.assets.get(identity)?;
                let selected = index == app.settings.row_index;
                Some(Row::new(vec![
                    Cell::from(format!(
                        "{} {}",
                        if selected { "›" } else { " " },
                        asset.asset_symbol
                    )),
                    Cell::from(truncate(&asset.asset_name, 26)),
                    Cell::from(asset.asset_decimals.to_string()),
                    Cell::from(truncate(&asset.asset_identity.to_string(), 42)),
                ])
                .style(selected_row_style(selected)))
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Assets · x delete "),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_settings_price_feeds(frame: &mut Frame, app: &App, area: Rect) {
    let rows = match app.settings_snapshot() {
        Some(snapshot) if snapshot.quoters.is_empty() => vec![Row::new(vec![
            Cell::from("No price feeds configured"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(snapshot) => snapshot
            .quoters
            .iter()
            .enumerate()
            .map(|(index, quoter)| {
                let selected = index == app.settings.row_index;
                Row::new(vec![
                    Cell::from(format!(
                        "{} {}",
                        if selected { "›" } else { " " },
                        quoter.quoter_name
                    )),
                    Cell::from(if quoter.enabled { "enabled" } else { "disabled" }),
                    Cell::from(format!(
                        "{} -> {}",
                        quoter.token_a,
                        quoter.token_b
                    )),
                    Cell::from(truncate(&quoter.quoter_identity, 16)),
                ])
                .style(selected_row_style(selected))
            })
            .collect(),
        None => vec![Row::new(vec![
            Cell::from("Loading price feeds…"),
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
            Constraint::Percentage(46),
            Constraint::Length(18),
        ],
    )
    .header(
        Row::new(vec!["Name", "Status", "Pair", "ID"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Price feeds · manage complex forms in web settings "),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_settings_vendors(frame: &mut Frame, app: &App, area: Rect) {
    let rows = match app.settings_snapshot() {
        Some(snapshot) if snapshot.all_vendors.is_empty() => vec![Row::new(vec![
            Cell::from("No vendor flags returned"),
            Cell::from(""),
            Cell::from(""),
        ])],
        Some(snapshot) => snapshot
            .all_vendors
            .iter()
            .enumerate()
            .map(|(index, vendor)| {
                let flag = vendor.flag.to_string();
                let enabled = snapshot.enabled_vendors.contains(&flag);
                let selected = index == app.settings.row_index;
                Row::new(vec![
                    Cell::from(format!(
                        "{} {}",
                        if selected { "›" } else { " " },
                        flag
                    )),
                    Cell::from(if enabled { "enabled" } else { "disabled" }).style(if enabled {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }),
                    Cell::from(if vendor.unfinished { "unfinished" } else { "" }),
                    Cell::from(vendor.comment.clone()),
                ])
                .style(selected_row_style(selected))
            })
            .collect(),
        None => vec![Row::new(vec![
            Cell::from("Loading vendors…"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
    };

    let title = app
        .settings_snapshot()
        .map(|snapshot| format!(" Vendors · {} enabled · e/x toggle ", snapshot.enabled_vendor_count()))
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
}

fn render_account_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(account) = app.selected_account() else {
        return;
    };

    let account_id = account.account_identity.0;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(area);

    let (balance_label, balance_amount_style) = balance_cell(app.balance_state(account_id));

    let pages = [
        (AccountPanel::Overview, "h", "Overview"),
        (AccountPanel::Assets, "a", "Assets"),
        (AccountPanel::Defi, "d", "Defi"),
        (AccountPanel::Transactions, "t", "Transactions"),
    ];

    let mut sidebar_lines = vec![
        Line::from(Span::styled(
            account.name.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
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
                balance_amount_style.patch(balance_sidebar_style(app.balance_state(account_id))),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Pages",
            Style::default().fg(Color::DarkGray),
        )),
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
            Style::default()
        };
        sidebar_lines.push(Line::from(Span::styled(
            format!(" {} {key}:{label}", if selected { "›" } else { " " }),
            style,
        )));
    }

    sidebar_lines.extend([
        Line::from(""),
        Line::from(Span::styled(
            "← sidebar · → content",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "↑/↓ choose page",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    let sidebar_title = if app.account_focus == AccountFocus::Sidebar {
        " Account · focus "
    } else {
        " Account "
    };

    frame.render_widget(
        Paragraph::new(sidebar_lines)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title(sidebar_title)),
        chunks[0],
    );

    let content_title = if app.account_focus == AccountFocus::Content {
        format!(" {} · focus ", account_panel_title(app.account_panel))
    } else {
        format!(" {} ", account_panel_title(app.account_panel))
    };

    let content = Block::default().borders(Borders::ALL).title(content_title);
    let inner = content.inner(chunks[1]);
    frame.render_widget(content, chunks[1]);

    match app.account_panel {
        AccountPanel::Overview => render_account_overview(frame, app, account_id, inner),
        AccountPanel::Assets => render_account_assets(frame, app, account_id, inner),
        AccountPanel::Defi => render_account_defi(frame, app, account_id, inner),
        AccountPanel::Transactions => render_account_transactions(frame, app, account_id, inner),
    }
}

fn account_panel_title(panel: AccountPanel) -> &'static str {
    match panel {
        AccountPanel::Overview => "Overview",
        AccountPanel::Assets => "Assets",
        AccountPanel::Defi => "Defi",
        AccountPanel::Transactions => "Transactions",
    }
}

fn render_account_overview(frame: &mut Frame, app: &App, account_identity: u64, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
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
        _ => (
            "—".to_string(),
            Style::default(),
            "—".to_string(),
        ),
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
        format!("Updated {updated}"),
        Style::default().fg(Color::DarkGray),
    )));

    let summary = Paragraph::new(summary_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Dashboard "),
    );

    frame.render_widget(summary, chunks[0]);
    render_account_assets(frame, app, account_identity, chunks[1]);
}

fn render_account_assets(frame: &mut Frame, app: &App, account_identity: u64, area: Rect) {
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
        Row::new(vec!["Asset", "Balance", "Value", "24h"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Asset overview "),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_account_defi(frame: &mut Frame, app: &App, account_identity: u64, area: Rect) {
    let state = app.defi_state(account_identity);

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

    let title = match state {
        Some(ResourceState::Ready(result)) if !result.errors.is_empty() => {
            format!(" DeFi positions · {} warning(s) ", result.errors.len())
        }
        _ => " DeFi positions ".to_string(),
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
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title(title))
    .column_spacing(2);

    frame.render_widget(table, area);
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

    let title = match state {
        Some(ResourceState::Ready(transactions)) => {
            format!(" Transaction history · {} tx ", transactions.len())
        }
        _ => " Transaction history ".to_string(),
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
        Row::new(vec!["Nonce", "Status", "Date", "Action", "Target", "Hash"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title(title))
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
                format!("{} {}", format_float(position.value, 4), position.underlying_symbol)
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
                (Some(current), Some(previous)) => percent_change(current, previous).map(|change| {
                    (change.label(), change.ratatui_style())
                }),
                _ => None,
            };

            let (change_text, change_style) = change.unwrap_or_else(|| {
                (
                    "—".to_string(),
                    Style::default().fg(Color::DarkGray),
                )
            });

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
        .map(|(_, name, amount, value, amount_style, change, change_style, value_style)| {
            Row::new(vec![
                Cell::from(name),
                Cell::from(amount).style(amount_style),
                Cell::from(value).style(value_style),
                Cell::from(change).style(change_style),
            ])
        })
        .collect()
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = if app.selected_account.is_some() {
        "account workspace  ← sidebar  → content  ↑↓ choose page in sidebar  h/a/d/t jump  r refresh  b back  q quit"
    } else if app.tab == Tab::Networks {
        "networks workspace  ↑↓ row  Enter endpoints  x delete endpoint  r reload  b back  q quit"
    } else if app.tab == Tab::Settings {
        "settings workspace  ←→ section  ↑↓ row  Enter drill  e/x action  r reload  b back  q quit"
    } else {
        "workspace  1 accounts  2 assets  3 prices  4 networks  5 settings  Tab next  ↑↓ select  Enter open  r reload  q quit"
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" command ", Style::default().fg(Color::Cyan)),
            Span::styled(help, Style::default().fg(Color::DarkGray)),
        ])),
        area,
    );
}

fn balance_cell(state: Option<&ResourceState<AccountBalances>>) -> (String, Style) {
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
            let style = formatted.ratatui_style();
            (formatted.text, style)
        }
        Some(ResourceState::Error(_)) => ("error".to_string(), Style::default().fg(Color::Red)),
    }
}

fn balance_sidebar_style(state: Option<&ResourceState<AccountBalances>>) -> Style {
    match state {
        Some(ResourceState::Error(_)) => Style::default().fg(Color::Red),
        Some(ResourceState::Loading) => Style::default().fg(Color::Yellow),
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

fn table_body_height(area: Rect) -> usize {
    // border top/bottom + header row
    area.height.saturating_sub(3) as usize
}

fn visible_range(len: usize, selected: usize, height: usize) -> (usize, usize) {
    if len == 0 || height == 0 {
        return (0, 0);
    }

    let half = height / 2;
    let mut start = selected.saturating_sub(half);
    if start + height > len {
        start = len.saturating_sub(height);
    }
    let end = (start + height).min(len);
    (start, end)
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

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let mut output = value.chars().take(max.saturating_sub(1)).collect::<String>();
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
    let executed = safe.is_executed.or_else(|| safe_bool_extra(safe, "isExecuted"));

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
        format!("{}…{}", &value[..10], &value[value.len().saturating_sub(4)..])
    }
}
