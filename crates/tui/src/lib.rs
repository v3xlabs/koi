mod app;
mod defi;
mod format;
mod loader;
mod settings;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use koi_client::ApiClient;
use app::{App, KeyAction};
use loader::{prepare_refresh_all, BackgroundUpdate, Loader};

const TICK_MS: u64 = 50;

pub async fn run(api_url: String) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client = ApiClient::new(api_url);
    let (update_tx, mut update_rx) = mpsc::unbounded_channel();
    let (input_tx, mut input_rx) = mpsc::unbounded_channel();
    let loader = Loader::new(client, update_tx);

    let mut app = App::new();
    let generation = prepare_refresh_all(&mut app);
    loader.spawn_refresh_all(generation);
    spawn_input_task(input_tx);
    terminal.draw(|frame| ui::render(frame, &app))?;

    let result = run_loop(
        &mut terminal,
        &mut app,
        &mut update_rx,
        &mut input_rx,
        &loader,
    )
    .await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: &mut mpsc::UnboundedReceiver<BackgroundUpdate>,
    input_rx: &mut mpsc::UnboundedReceiver<KeyCode>,
    loader: &Loader,
) -> Result<()> {
    let mut tick = tokio::time::interval(Duration::from_millis(TICK_MS));
    let mut dirty = true;

    loop {
        tokio::select! {
            code = input_rx.recv() => {
                if let Some(code) = code {
                    match app.handle_key(code) {
                        KeyAction::Quit => break,
                        KeyAction::RefreshAll => {
                            let generation = prepare_refresh_all(app);
                            loader.spawn_refresh_all(generation);
                        }
                        KeyAction::RefreshAccountData(account_id) => {
                            app.prepare_balance_fetch(account_id);
                            loader.spawn_balance(app.current_generation(), account_id);
                            if let Some(address) = app.account_address(account_id) {
                                app.prepare_defi_fetch(account_id);
                                loader.spawn_defi(app.current_generation(), account_id, address);
                            }
                        }
                        KeyAction::RefreshDefi(account_id) => {
                            if let Some(address) = app.account_address(account_id) {
                                app.prepare_defi_fetch(account_id);
                                loader.spawn_defi(app.current_generation(), account_id, address);
                            }
                        }
                        KeyAction::RefreshTransactions(account_id) => {
                            app.prepare_transactions_fetch(account_id);
                            loader.spawn_transactions(app.current_generation(), account_id);
                        }
                        KeyAction::RefreshSettings => {
                            app.settings_state = app::ResourceState::Loading;
                            loader.spawn_settings(app.current_generation(), app.settings_network_ids());
                        }
                        KeyAction::DeleteNetworkEndpoint(network_id, endpoint_id) => {
                            app.settings.notice = Some(format!(
                                "Deleting endpoint #{endpoint_id} on network {network_id}…"
                            ));
                            loader.delete_network_endpoint(
                                app.current_generation(),
                                network_id,
                                endpoint_id,
                                app.settings_network_ids(),
                            );
                        }
                        KeyAction::DeleteAsset(asset_identity) => {
                            app.settings.notice = Some(format!("Deleting asset {asset_identity}…"));
                            loader.delete_asset(
                                app.current_generation(),
                                asset_identity,
                                app.settings_network_ids(),
                            );
                        }
                        KeyAction::SetVendor(flag, enabled) => {
                            app.settings.notice = Some(format!(
                                "{} vendor {flag}…",
                                if enabled { "Enabling" } else { "Disabling" }
                            ));
                            loader.set_vendor(
                                app.current_generation(),
                                flag,
                                enabled,
                                app.settings_network_ids(),
                            );
                        }
                        KeyAction::None => {}
                    }
                    dirty = true;
                }
            }
            _ = tick.tick() => {
                if app.needs_refresh() {
                    let generation = prepare_refresh_all(app);
                    loader.spawn_refresh_all(generation);
                    dirty = true;
                }
            }
        }

        for _ in 0..64 {
            let Ok(update) = rx.try_recv() else {
                break;
            };
            app.apply(update);
            dirty = true;
        }

        if dirty {
            terminal.draw(|frame| ui::render(frame, app))?;
            dirty = false;
        }
    }

    Ok(())
}

fn spawn_input_task(tx: mpsc::UnboundedSender<KeyCode>) {
    tokio::task::spawn_blocking(move || loop {
        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                if tx.send(key.code).is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    });
}
