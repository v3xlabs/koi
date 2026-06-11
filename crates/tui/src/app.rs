use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};

use koi::models::{
    account::{Account, balances::AccountBalances, metadata::WalletType},
    asset::Asset,
    network::{Network, pool::RpcPoolStats},
    tx::Tx,
};

use super::defi::DefiResult;
use super::form::{ActiveForm, FormAction, available_presets};
use super::icon::IconRenderer;
use super::layout::{UiLayout, table_body_height};
use super::settings::{SettingsSection, SettingsSnapshot, SettingsState};

const REFRESH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub enum ResourceState<T> {
    Idle,
    Loading,
    Ready(T),
    Error(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Accounts,
    Assets,
    Prices,
    Networks,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccountPanel {
    Overview,
    Assets,
    Defi,
    Transactions,
}

impl AccountPanel {
    const ALL: [AccountPanel; 4] = [
        AccountPanel::Overview,
        AccountPanel::Assets,
        AccountPanel::Defi,
        AccountPanel::Transactions,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccountFocus {
    Sidebar,
    Content,
}

pub struct App {
    pub tab: Tab,
    pub account_panel: AccountPanel,
    pub account_focus: AccountFocus,
    pub accounts: Vec<Account>,
    pub networks: Vec<Network>,
    pub assets: HashMap<String, Asset>,
    pub rpc_states: HashMap<u64, ResourceState<RpcPoolStats>>,
    pub balance_states: HashMap<u64, ResourceState<AccountBalances>>,
    balance_refreshing: HashSet<u64>,
    pub defi_states: HashMap<u64, ResourceState<DefiResult>>,
    pub tx_states: HashMap<u64, ResourceState<Vec<Tx>>>,
    pub settings: SettingsState,
    pub settings_state: ResourceState<SettingsSnapshot>,
    pub form: Option<ActiveForm>,
    pub list_index: usize,
    pub list_scroll: usize,
    pub selected_account: Option<u64>,
    pub status: String,
    pub notice: Option<String>,
    pub connected: bool,
    pub loading_core: bool,
    pub refresh_in_flight: bool,
    refresh_generation: u64,
    last_refresh: Instant,
    pub(crate) dirty: bool,
    pub icon_renderer: Option<IconRenderer>,
    pub layout: UiLayout,
}

impl App {
    pub fn new() -> Self {
        Self {
            tab: Tab::Accounts,
            account_panel: AccountPanel::Overview,
            account_focus: AccountFocus::Sidebar,
            accounts: Vec::new(),
            networks: Vec::new(),
            assets: HashMap::new(),
            rpc_states: HashMap::new(),
            balance_states: HashMap::new(),
            balance_refreshing: HashSet::new(),
            defi_states: HashMap::new(),
            tx_states: HashMap::new(),
            settings: SettingsState::new(),
            settings_state: ResourceState::Idle,
            form: None,
            list_index: 0,
            list_scroll: 0,
            selected_account: None,
            status: "Connecting…".to_string(),
            notice: None,
            connected: false,
            loading_core: true,
            refresh_in_flight: false,
            refresh_generation: 0,
            last_refresh: Instant::now(),
            dirty: true,
            icon_renderer: None,
            layout: UiLayout::default(),
        }
    }

    pub fn init_icons(&mut self) {
        self.icon_renderer = Some(IconRenderer::new());
    }

    pub fn needs_refresh(&self) -> bool {
        !self.refresh_in_flight && (self.dirty || self.last_refresh.elapsed() >= REFRESH_INTERVAL)
    }

    pub fn is_loading(&self) -> bool {
        self.loading_core
            || self
                .balance_states
                .values()
                .any(|state| matches!(state, ResourceState::Loading))
            || self
                .rpc_states
                .values()
                .any(|state| matches!(state, ResourceState::Loading))
    }

    pub fn begin_refresh(&mut self) -> u64 {
        self.refresh_generation = self.refresh_generation.wrapping_add(1);
        self.refresh_in_flight = true;
        self.loading_core = true;
        self.notice = None;
        self.dirty = false;

        for network in &self.networks {
            self.rpc_states
                .insert(network.network_identity.0, ResourceState::Loading);
        }
        let account_ids: Vec<u64> = self
            .accounts
            .iter()
            .map(|account| account.account_identity.0)
            .collect();
        for account_id in account_ids {
            self.prepare_balance_fetch(account_id);
        }

        self.refresh_generation
    }

    pub fn current_generation(&self) -> u64 {
        self.refresh_generation
    }

    pub fn mark_refresh_failed(&mut self) {
        self.refresh_in_flight = false;
        self.loading_core = false;
        self.last_refresh = Instant::now();
    }

    pub fn apply(&mut self, update: super::loader::BackgroundUpdate) {
        use super::loader::BackgroundUpdate;

        match update {
            BackgroundUpdate::Health {
                generation,
                connected,
                notice,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.connected = connected;
                self.notice = notice;
                if !connected {
                    self.mark_refresh_failed();
                    self.status = "Disconnected".to_string();
                }
            }
            BackgroundUpdate::AccountsLoaded {
                generation,
                accounts,
                notice,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.accounts = accounts;
                if notice.is_some() {
                    self.notice = notice;
                }
                if self.accounts.is_empty() {
                    self.loading_core = false;
                    self.complete_refresh_if_idle();
                }
                self.clamp_list_index();
                let account_ids = self
                    .accounts
                    .iter()
                    .map(|account| account.account_identity.0)
                    .collect::<HashSet<_>>();
                self.balance_states
                    .retain(|account_id, _| account_ids.contains(account_id));
                self.defi_states
                    .retain(|account_id, _| account_ids.contains(account_id));
                self.tx_states
                    .retain(|account_id, _| account_ids.contains(account_id));
                let account_ids: Vec<u64> = self
                    .accounts
                    .iter()
                    .map(|account| account.account_identity.0)
                    .collect();
                for account_id in account_ids {
                    self.prepare_balance_fetch(account_id);
                }
                self.update_status();
            }
            BackgroundUpdate::NetworksLoaded {
                generation,
                networks,
                notice,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.networks = networks;
                if notice.is_some() {
                    self.notice = notice;
                }
                if self.networks.is_empty() {
                    self.loading_core = false;
                    self.complete_refresh_if_idle();
                }
                self.clamp_list_index();
                let network_ids = self
                    .networks
                    .iter()
                    .map(|network| network.network_identity.0)
                    .collect::<HashSet<_>>();
                self.rpc_states
                    .retain(|network_id, _| network_ids.contains(network_id));
                for network in &self.networks {
                    self.rpc_states
                        .entry(network.network_identity.0)
                        .or_insert(ResourceState::Loading);
                }
            }
            BackgroundUpdate::AssetsLoaded {
                generation,
                assets,
                notice,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.assets = assets;
                if notice.is_some() {
                    self.notice = notice;
                }
                self.clamp_list_index();
                self.loading_core = false;
                self.update_status();
                self.complete_refresh_if_idle();
            }
            BackgroundUpdate::Rpc {
                generation,
                network_id,
                state,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.rpc_states.insert(network_id, state);
            }
            BackgroundUpdate::Balance {
                generation,
                account_id,
                state,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.balance_states.insert(account_id, state);
                self.balance_refreshing.remove(&account_id);
                self.update_status();

                let pending = self
                    .balance_states
                    .values()
                    .any(|state| matches!(state, ResourceState::Loading))
                    || !self.balance_refreshing.is_empty()
                    || self
                        .rpc_states
                        .values()
                        .any(|state| matches!(state, ResourceState::Loading));

                if !self.loading_core && !pending {
                    self.complete_refresh_if_idle();
                }
            }
            BackgroundUpdate::Defi {
                generation,
                account_id,
                state,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.defi_states.insert(account_id, state);
            }
            BackgroundUpdate::Transactions {
                generation,
                account_id,
                state,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.tx_states.insert(account_id, state);
            }
            BackgroundUpdate::Settings { generation, state } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.settings_state = state;
            }
            BackgroundUpdate::Notice { generation, notice } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.settings.notice = Some(notice);
            }
            BackgroundUpdate::EndpointNextId {
                generation,
                next_id,
                ..
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.set_endpoint_next_id(next_id);
            }
            BackgroundUpdate::AssetMetadata {
                generation,
                identity,
                result,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                if let Some(form) = &mut self.form {
                    form.apply_asset_metadata(&identity, result);
                }
            }
        }
    }

    fn complete_refresh_if_idle(&mut self) {
        let pending = self
            .balance_states
            .values()
            .any(|state| matches!(state, ResourceState::Loading))
            || !self.balance_refreshing.is_empty()
            || self
                .rpc_states
                .values()
                .any(|state| matches!(state, ResourceState::Loading));

        if !self.loading_core && !pending {
            self.refresh_in_flight = false;
            self.last_refresh = Instant::now();
        }
    }

    pub fn prepare_balance_fetch(&mut self, account_id: u64) {
        if matches!(
            self.balance_states.get(&account_id),
            Some(ResourceState::Ready(_))
        ) {
            self.balance_refreshing.insert(account_id);
        } else {
            self.balance_states
                .insert(account_id, ResourceState::Loading);
        }
    }

    pub fn is_balance_refreshing(&self, account_id: u64) -> bool {
        self.balance_refreshing.contains(&account_id)
    }

    pub fn prepare_defi_fetch(&mut self, account_id: u64) {
        self.defi_states.insert(account_id, ResourceState::Loading);
    }

    pub fn prepare_transactions_fetch(&mut self, account_id: u64) {
        self.tx_states.insert(account_id, ResourceState::Loading);
    }

    pub fn handle_mouse(&mut self, event: MouseEvent) -> KeyAction {
        if self.form.is_some() {
            return KeyAction::None;
        }

        match event.kind {
            MouseEventKind::Moved => {
                self.handle_mouse_move(event.column, event.row);
                KeyAction::None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(event.column, event.row)
            }
            MouseEventKind::ScrollUp => {
                self.handle_mouse_scroll(event.column, event.row, -3);
                KeyAction::None
            }
            MouseEventKind::ScrollDown => {
                self.handle_mouse_scroll(event.column, event.row, 3);
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    fn handle_mouse_move(&mut self, column: u16, row: u16) {
        if let Some(panel) = self.layout.account_panel_at(column, row) {
            if self.selected_account.is_some() {
                self.account_panel = panel;
                self.account_focus = AccountFocus::Sidebar;
            }
            return;
        }

        if let Some(index) = self.layout.list_row_at(column, row) {
            if self.selected_account.is_some() {
                return;
            }

            if self.uses_resource_rows() {
                if self.settings.row_index != index {
                    self.settings.row_index = index;
                    self.reconcile_scroll(table_body_height(self.layout.body));
                }
            } else if self.tab == Tab::Accounts && self.list_index != index {
                self.list_index = index;
                self.reconcile_scroll(table_body_height(self.layout.body));
            }
        }
    }

    fn handle_mouse_click(&mut self, column: u16, row: u16) -> KeyAction {
        if let Some(tab) = self.layout.tab_at(column, row) {
            self.switch_tab(tab);
            if tab == Tab::Settings || tab == Tab::Prices {
                if matches!(self.settings_state, ResourceState::Idle) {
                    return KeyAction::RefreshSettings;
                }
            }
            return KeyAction::None;
        }

        if let Some(panel) = self.layout.account_panel_at(column, row) {
            self.account_panel = panel;
            self.account_focus = AccountFocus::Sidebar;
            if let Some(account_id) = self.selected_account {
                return match panel {
                    AccountPanel::Defi
                        if !matches!(
                            self.defi_states.get(&account_id),
                            Some(ResourceState::Ready(_))
                        ) =>
                    {
                        KeyAction::RefreshDefi(account_id)
                    }
                    AccountPanel::Transactions
                        if !matches!(
                            self.tx_states.get(&account_id),
                            Some(ResourceState::Ready(_))
                        ) =>
                    {
                        KeyAction::RefreshTransactions(account_id)
                    }
                    _ => KeyAction::None,
                };
            }
            return KeyAction::None;
        }

        if let Some(index) = self.layout.list_row_at(column, row) {
            if self.uses_resource_rows() {
                self.settings.row_index = index;
            } else if self.tab == Tab::Accounts {
                self.list_index = index;
            }
            return self.activate();
        }

        KeyAction::None
    }

    fn handle_mouse_scroll(&mut self, column: u16, row: u16, delta: i32) {
        if self.selected_account.is_some() {
            return;
        }

        let Some(table) = self.layout.list_table else {
            return;
        };
        if !super::layout::contains(table.area, column, row) {
            return;
        }

        if self.uses_resource_rows() {
            self.settings.move_row(delta, self.settings_row_count());
            self.reconcile_scroll(table_body_height(self.layout.body));
        } else if self.tab == Tab::Accounts {
            self.move_selection(delta);
            self.reconcile_scroll(table_body_height(self.layout.body));
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) -> KeyAction {
        if self.form.is_some() {
            return self.handle_form_key(code);
        }

        match code {
            KeyCode::Char('q') => KeyAction::Quit,
            KeyCode::Esc | KeyCode::Char('b') => {
                if self.selected_account.is_some() {
                    self.selected_account = None;
                    self.account_panel = AccountPanel::Overview;
                    self.account_focus = AccountFocus::Sidebar;
                } else if self.tab == Tab::Networks && self.settings.nested_network.is_some() {
                    self.settings.nested_network = None;
                    self.settings.row_index = 0;
                    self.settings.row_scroll = 0;
                }
                KeyAction::None
            }
            KeyCode::Char('1') => {
                self.switch_tab(Tab::Accounts);
                KeyAction::None
            }
            KeyCode::Char('2') => {
                self.switch_tab(Tab::Assets);
                KeyAction::None
            }
            KeyCode::Char('3') => {
                self.switch_tab(Tab::Prices);
                if matches!(self.settings_state, ResourceState::Ready(_)) {
                    KeyAction::None
                } else {
                    KeyAction::RefreshSettings
                }
            }
            KeyCode::Char('4') => {
                self.switch_tab(Tab::Networks);
                KeyAction::None
            }
            KeyCode::Char('5') => {
                self.switch_tab(Tab::Settings);
                if matches!(self.settings_state, ResourceState::Ready(_)) {
                    KeyAction::None
                } else {
                    KeyAction::RefreshSettings
                }
            }
            KeyCode::Tab => {
                self.cycle_tab();
                if self.tab == Tab::Settings && matches!(self.settings_state, ResourceState::Idle) {
                    KeyAction::RefreshSettings
                } else {
                    KeyAction::None
                }
            }
            KeyCode::Char('r') => {
                if let Some(account_id) = self.selected_account {
                    match self.account_panel {
                        AccountPanel::Defi => KeyAction::RefreshDefi(account_id),
                        AccountPanel::Transactions => KeyAction::RefreshTransactions(account_id),
                        AccountPanel::Overview | AccountPanel::Assets => {
                            KeyAction::RefreshAccountData(account_id)
                        }
                    }
                } else if matches!(
                    self.tab,
                    Tab::Assets | Tab::Prices | Tab::Networks | Tab::Settings
                ) {
                    KeyAction::RefreshSettings
                } else {
                    self.dirty = true;
                    KeyAction::RefreshAll
                }
            }
            KeyCode::Left if self.selected_account.is_some() => {
                self.account_focus = AccountFocus::Sidebar;
                KeyAction::None
            }
            KeyCode::Right if self.selected_account.is_some() => {
                self.account_focus = AccountFocus::Content;
                KeyAction::None
            }
            KeyCode::Left if self.tab == Tab::Settings => {
                self.settings.move_section(-1);
                KeyAction::None
            }
            KeyCode::Right if self.tab == Tab::Settings => {
                self.settings.move_section(1);
                KeyAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_account.is_some() && self.account_focus == AccountFocus::Sidebar {
                    return self.move_account_panel(-1);
                }

                if self.uses_resource_rows() && self.selected_account.is_none() {
                    self.settings.move_row(-1, self.settings_row_count());
                } else {
                    self.move_selection(-1);
                }
                KeyAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_account.is_some() && self.account_focus == AccountFocus::Sidebar {
                    return self.move_account_panel(1);
                }

                if self.uses_resource_rows() && self.selected_account.is_none() {
                    self.settings.move_row(1, self.settings_row_count());
                } else {
                    self.move_selection(1);
                }
                KeyAction::None
            }
            KeyCode::PageUp => {
                if self.uses_resource_rows() && self.selected_account.is_none() {
                    self.settings.move_row(-10, self.settings_row_count());
                } else {
                    self.move_selection(-10);
                }
                KeyAction::None
            }
            KeyCode::PageDown => {
                if self.uses_resource_rows() && self.selected_account.is_none() {
                    self.settings.move_row(10, self.settings_row_count());
                } else {
                    self.move_selection(10);
                }
                KeyAction::None
            }
            KeyCode::Enter
                if self.selected_account.is_some()
                    && self.account_focus == AccountFocus::Content =>
            {
                self.account_focus = AccountFocus::Sidebar;
                KeyAction::None
            }
            KeyCode::Enter => self.activate(),
            KeyCode::Char('x') if self.uses_resource_rows() && self.selected_account.is_none() => {
                self.settings_action()
            }
            KeyCode::Char('e') if self.uses_resource_rows() && self.selected_account.is_none() => {
                self.settings_edit_action()
            }
            KeyCode::Char('n') if self.selected_account.is_none() => self.open_add_form(),
            KeyCode::Char('a') if self.selected_account.is_some() => {
                self.account_panel = AccountPanel::Assets;
                self.account_focus = AccountFocus::Sidebar;
                KeyAction::None
            }
            KeyCode::Char('d') if self.selected_account.is_some() => {
                self.account_panel = AccountPanel::Defi;
                self.account_focus = AccountFocus::Sidebar;
                let account_id = self.selected_account.unwrap();
                if matches!(
                    self.defi_states.get(&account_id),
                    Some(ResourceState::Ready(_))
                ) {
                    KeyAction::None
                } else {
                    KeyAction::RefreshDefi(account_id)
                }
            }
            KeyCode::Char('t') if self.selected_account.is_some() => {
                self.account_panel = AccountPanel::Transactions;
                self.account_focus = AccountFocus::Sidebar;
                let account_id = self.selected_account.unwrap();
                if matches!(
                    self.tx_states.get(&account_id),
                    Some(ResourceState::Ready(_))
                ) {
                    KeyAction::None
                } else {
                    KeyAction::RefreshTransactions(account_id)
                }
            }
            KeyCode::Char('h') if self.selected_account.is_some() => {
                self.account_panel = AccountPanel::Overview;
                self.account_focus = AccountFocus::Sidebar;
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    fn cycle_tab(&mut self) {
        let next = match self.tab {
            Tab::Accounts => Tab::Assets,
            Tab::Assets => Tab::Prices,
            Tab::Prices => Tab::Networks,
            Tab::Networks => Tab::Settings,
            Tab::Settings => Tab::Accounts,
        };
        self.switch_tab(next);
    }

    fn switch_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.selected_account = None;
        self.account_panel = AccountPanel::Overview;
        self.account_focus = AccountFocus::Sidebar;
        self.form = None;
        self.settings.row_index = 0;
        self.settings.row_scroll = 0;
        self.list_scroll = 0;
        if !matches!(self.tab, Tab::Settings) {
            if !matches!(self.tab, Tab::Networks) {
                self.settings.nested_network = None;
            }
        }
        self.clamp_list_index();
    }

    fn open_add_form(&mut self) -> KeyAction {
        match self.tab {
            Tab::Assets => {
                self.form = Some(ActiveForm::open_add_asset());
                KeyAction::None
            }
            Tab::Networks if self.settings.nested_network.is_some() => {
                let network_id = self.settings.nested_network.unwrap();
                self.form = Some(ActiveForm::open_add_endpoint(network_id));
                KeyAction::FetchEndpointNextId(network_id)
            }
            Tab::Networks => {
                self.form = Some(ActiveForm::open_add_network());
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    fn handle_form_key(&mut self, code: KeyCode) -> KeyAction {
        let Some(mut form) = self.form.take() else {
            return KeyAction::None;
        };

        match form.handle_key(code) {
            FormAction::None => {
                self.form = Some(form);
                KeyAction::None
            }
            FormAction::Cancel => {
                self.form = None;
                KeyAction::None
            }
            FormAction::OpenNetworkPresets => {
                let presets = available_presets(
                    &self
                        .networks
                        .iter()
                        .map(|network| network.network_identity.0)
                        .collect::<Vec<_>>(),
                );
                if presets.is_empty() {
                    self.settings.notice = Some("No network presets available".to_string());
                    self.form = None;
                } else {
                    self.form = Some(ActiveForm::AddNetworkPreset {
                        presets,
                        selected: 0,
                    });
                }
                KeyAction::None
            }
            FormAction::FetchAssetMetadata(identity) => {
                self.form = Some(form);
                KeyAction::FetchAssetMetadata(identity)
            }
            FormAction::SubmitCreateAsset(asset) => {
                self.form = None;
                self.settings.notice = Some(format!("Creating asset {}…", asset.asset_identity));
                KeyAction::CreateAsset(asset)
            }
            FormAction::SubmitCreateNetwork(network) => {
                self.form = None;
                self.settings.notice = Some(format!("Creating network {}…", network.network_name));
                KeyAction::CreateNetwork(network)
            }
            FormAction::SubmitCreateEndpoint(endpoint) => {
                self.form = None;
                self.settings.notice = Some(format!(
                    "Creating endpoint #{}…",
                    endpoint.endpoint_identity
                ));
                KeyAction::CreateNetworkEndpoint(endpoint)
            }
        }
    }

    pub fn set_endpoint_next_id(&mut self, next_id: i32) {
        if let Some(form) = &mut self.form {
            form.set_endpoint_next_id(next_id);
        }
    }

    fn move_account_panel(&mut self, delta: i32) -> KeyAction {
        let current = AccountPanel::ALL
            .iter()
            .position(|panel| *panel == self.account_panel)
            .unwrap_or_default() as i32;
        let next = (current + delta).rem_euclid(AccountPanel::ALL.len() as i32) as usize;
        self.account_panel = AccountPanel::ALL[next];

        let Some(account_id) = self.selected_account else {
            return KeyAction::None;
        };

        match self.account_panel {
            AccountPanel::Defi
                if !matches!(
                    self.defi_states.get(&account_id),
                    Some(ResourceState::Ready(_))
                ) =>
            {
                KeyAction::RefreshDefi(account_id)
            }
            AccountPanel::Transactions
                if !matches!(
                    self.tx_states.get(&account_id),
                    Some(ResourceState::Ready(_))
                ) =>
            {
                KeyAction::RefreshTransactions(account_id)
            }
            _ => KeyAction::None,
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let len = match self.tab {
            Tab::Accounts if self.selected_account.is_none() => self.accounts.len(),
            Tab::Assets | Tab::Prices | Tab::Networks => self.settings_row_count(),
            Tab::Settings => self.settings_row_count(),
            Tab::Accounts => 0,
        };

        if len == 0 {
            return;
        }

        let next = self.list_index as i32 + delta;
        self.list_index = next.clamp(0, len as i32 - 1) as usize;
    }

    fn activate(&mut self) -> KeyAction {
        if matches!(self.tab, Tab::Networks | Tab::Settings) && self.selected_account.is_none() {
            return self.settings_enter_action();
        }

        if self.tab != Tab::Accounts || self.selected_account.is_some() {
            return KeyAction::None;
        }

        let Some(account) = self.accounts.get(self.list_index) else {
            return KeyAction::None;
        };

        let account_id = account.account_identity.0;
        self.selected_account = Some(account_id);
        self.account_panel = AccountPanel::Overview;
        self.account_focus = AccountFocus::Sidebar;

        if matches!(
            self.balance_states.get(&account_id),
            Some(ResourceState::Ready(_))
        ) {
            KeyAction::RefreshDefi(account_id)
        } else {
            KeyAction::RefreshAccountData(account_id)
        }
    }

    fn clamp_list_index(&mut self) {
        let len = match self.tab {
            Tab::Accounts if self.selected_account.is_none() => self.accounts.len(),
            Tab::Assets | Tab::Prices | Tab::Networks => self.settings_row_count(),
            Tab::Settings => self.settings_row_count(),
            Tab::Accounts => 0,
        };

        if len == 0 {
            self.list_index = 0;
            self.list_scroll = 0;
            self.settings.row_index = 0;
            self.settings.row_scroll = 0;
        } else {
            if self.list_index >= len {
                self.list_index = len - 1;
            }
            if self.uses_resource_rows() && self.settings.row_index >= len {
                self.settings.row_index = len - 1;
            }
        }
    }

    pub fn reconcile_scroll(&mut self, viewport_height: usize) {
        use super::scroll::ensure_visible;

        if self.selected_account.is_some() {
            return;
        }

        if self.uses_resource_rows() {
            let len = self.settings_row_count();
            ensure_visible(
                &mut self.settings.row_scroll,
                self.settings.row_index,
                len,
                viewport_height,
            );
        } else if self.tab == Tab::Accounts {
            ensure_visible(
                &mut self.list_scroll,
                self.list_index,
                self.accounts.len(),
                viewport_height,
            );
        }
    }

    pub(crate) fn update_status(&mut self) {
        let loaded_balances = self
            .balance_states
            .values()
            .filter(|state| matches!(state, ResourceState::Ready(_)))
            .count();
        let failed_balances = self
            .balance_states
            .values()
            .filter(|state| matches!(state, ResourceState::Error(_)))
            .count();

        self.status = if self.connected {
            if failed_balances > 0 {
                format!(
                    "Connected · {} account(s) · balances {loaded_balances}/{} ({} failed)",
                    self.accounts.len(),
                    self.accounts.len(),
                    failed_balances
                )
            } else {
                format!("Connected · {} account(s)", self.accounts.len())
            }
        } else {
            "Disconnected".to_string()
        };
    }

    pub fn selected_account(&self) -> Option<&Account> {
        let id = self.selected_account?;
        self.accounts
            .iter()
            .find(|account| account.account_identity.0 == id)
    }

    pub fn balance_state(&self, account_id: u64) -> Option<&ResourceState<AccountBalances>> {
        self.balance_states.get(&account_id)
    }

    pub fn rpc_state(&self, network_id: u64) -> Option<&ResourceState<RpcPoolStats>> {
        self.rpc_states.get(&network_id)
    }

    pub fn defi_state(&self, account_id: u64) -> Option<&ResourceState<DefiResult>> {
        self.defi_states.get(&account_id)
    }

    pub fn tx_state(&self, account_id: u64) -> Option<&ResourceState<Vec<Tx>>> {
        self.tx_states.get(&account_id)
    }

    pub fn account_address(&self, account_id: u64) -> Option<String> {
        let account = self
            .accounts
            .iter()
            .find(|account| account.account_identity.0 == account_id)?;
        match &account.metadata {
            WalletType::Safe(wallet) => Some(wallet.evm_address.to_string()),
            WalletType::EOA(wallet) => Some(wallet.evm_address.to_string()),
            WalletType::View(wallet) => Some(wallet.evm_address.to_string()),
            WalletType::Railgun(_) => None,
        }
    }

    pub fn settings_network_ids(&self) -> Vec<u64> {
        self.networks
            .iter()
            .map(|network| network.network_identity.0)
            .collect()
    }

    pub fn settings_row_count(&self) -> usize {
        let section = match self.tab {
            Tab::Assets => SettingsSection::Assets,
            Tab::Prices => SettingsSection::PriceFeeds,
            Tab::Networks => SettingsSection::Networks,
            _ => self.settings.section(),
        };

        match section {
            SettingsSection::General => 1,
            SettingsSection::Networks => {
                if let Some(network_id) = self.settings.nested_network {
                    return self
                        .settings_snapshot()
                        .and_then(|snapshot| snapshot.endpoints.get(&network_id))
                        .map(|endpoints| endpoints.len())
                        .unwrap_or_default();
                }
                self.networks.len()
            }
            SettingsSection::Assets => self.assets.len(),
            SettingsSection::PriceFeeds => self
                .settings_snapshot()
                .map(|snapshot| snapshot.quoters.len())
                .unwrap_or_default(),
            SettingsSection::Vendors => self
                .settings_snapshot()
                .map(|snapshot| snapshot.all_vendors.len())
                .unwrap_or_default(),
        }
    }

    pub fn settings_snapshot(&self) -> Option<&SettingsSnapshot> {
        match &self.settings_state {
            ResourceState::Ready(snapshot) => Some(snapshot),
            _ => None,
        }
    }

    fn settings_enter_action(&mut self) -> KeyAction {
        let section = match self.tab {
            Tab::Networks => SettingsSection::Networks,
            _ => self.settings.section(),
        };

        if section == SettingsSection::Networks && self.settings.nested_network.is_none() {
            if let Some(network) = self.networks.get(self.settings.row_index) {
                self.settings.nested_network = Some(network.network_identity.0);
                self.settings.row_index = 0;
                self.settings.row_scroll = 0;
            }
        }
        KeyAction::None
    }

    fn settings_action(&mut self) -> KeyAction {
        let section = match self.tab {
            Tab::Assets => SettingsSection::Assets,
            Tab::Prices => SettingsSection::PriceFeeds,
            Tab::Networks => SettingsSection::Networks,
            _ => self.settings.section(),
        };

        match section {
            SettingsSection::Networks => {
                let Some(network_id) = self.settings.nested_network else {
                    return KeyAction::None;
                };
                let Some(endpoint_id) = self
                    .settings_snapshot()
                    .and_then(|snapshot| snapshot.endpoints.get(&network_id))
                    .and_then(|endpoints| endpoints.get(self.settings.row_index))
                    .map(|endpoint| endpoint.endpoint_identity)
                else {
                    return KeyAction::None;
                };
                KeyAction::DeleteNetworkEndpoint(network_id, endpoint_id)
            }
            SettingsSection::Assets => {
                let Some(asset_identity) = self
                    .settings_asset_identities()
                    .get(self.settings.row_index)
                    .cloned()
                else {
                    return KeyAction::None;
                };
                KeyAction::DeleteAsset(asset_identity)
            }
            SettingsSection::Vendors => {
                let Some((flag, enabled)) = self.selected_vendor_toggle() else {
                    return KeyAction::None;
                };
                KeyAction::SetVendor(flag, !enabled)
            }
            SettingsSection::General | SettingsSection::PriceFeeds => KeyAction::None,
        }
    }

    fn settings_edit_action(&mut self) -> KeyAction {
        let section = match self.tab {
            Tab::Assets => SettingsSection::Assets,
            Tab::Prices => SettingsSection::PriceFeeds,
            Tab::Networks => SettingsSection::Networks,
            _ => self.settings.section(),
        };

        match section {
            SettingsSection::Vendors => self.settings_action(),
            _ => {
                self.settings.notice = Some(
                    "Editing/adding complex resources is not interactive yet; use web settings for forms"
                        .to_string(),
                );
                KeyAction::None
            }
        }
    }

    fn selected_vendor_toggle(&self) -> Option<(String, bool)> {
        let snapshot = self.settings_snapshot()?;
        let flag = snapshot
            .all_vendors
            .get(self.settings.row_index)?
            .flag
            .to_string();
        let enabled = snapshot.enabled_vendors.contains(&flag);
        Some((flag, enabled))
    }

    pub fn settings_asset_identities(&self) -> Vec<String> {
        let mut identities = self.assets.keys().cloned().collect::<Vec<_>>();
        identities.sort();
        identities
    }

    fn uses_resource_rows(&self) -> bool {
        matches!(
            self.tab,
            Tab::Assets | Tab::Prices | Tab::Networks | Tab::Settings
        )
    }
}

pub enum KeyAction {
    None,
    Quit,
    RefreshAll,
    RefreshAccountData(u64),
    RefreshDefi(u64),
    RefreshTransactions(u64),
    RefreshSettings,
    DeleteNetworkEndpoint(u64, i32),
    DeleteAsset(String),
    SetVendor(String, bool),
    FetchEndpointNextId(u64),
    FetchAssetMetadata(String),
    CreateAsset(koi::models::asset::Asset),
    CreateNetwork(koi::models::network::Network),
    CreateNetworkEndpoint(koi::models::network::endpoint::NetworkEndpoint),
}
