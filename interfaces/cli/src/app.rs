use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};

use koi::models::{
    account::{Account, balances::AccountBalances, group::AccountGroup, metadata::WalletType},
    asset::{Asset, AssetIconData},
    network::{Network, pool::RpcPoolStats},
    tx::Tx,
};

use super::config::Theme;
use super::defi::DefiResult;
use super::form::{ActiveForm, FormAction, NewAccountWallet};
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GrabbedItem {
    Account(u64),
    Group(u64),
}

pub struct MoveMode {
    pub groups: Vec<AccountGroup>,
    pub accounts: Vec<Account>,
    pub grabbed: Option<GrabbedItem>,
    pub dirty: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub enum AccountListRow {
    GroupHeader {
        group_id: Option<u64>,
        name: String,
        collapsed: bool,
        count: usize,
    },
    Account {
        account_id: u64,
    },
    EmptyGroup,
}

#[derive(Clone)]
pub enum CommandChoice {
    Tab(Tab),
    Account(u64),
    Theme(Theme),
    DisplayCurrency,
}

impl CommandChoice {
    pub fn label(&self, app: &App) -> String {
        match self {
            Self::Tab(Tab::Accounts) => "Go to Accounts".to_string(),
            Self::Tab(Tab::Assets) => "Go to Assets".to_string(),
            Self::Tab(Tab::Prices) => "Go to Prices".to_string(),
            Self::Tab(Tab::Networks) => "Go to Networks".to_string(),
            Self::Tab(Tab::Settings) => "Go to Settings".to_string(),
            Self::Account(id) => app
                .account_by_id(*id)
                .map(|account| format!("Open account: {}", account.name))
                .unwrap_or_else(|| "Open account".to_string()),
            Self::Theme(theme) => format!("Theme: {}", theme.label()),
            Self::DisplayCurrency => "Change display currency".to_string(),
        }
    }
}

pub struct CommandPalette {
    pub query: String,
    pub selected: usize,
}

pub fn normalized_group_id(account: &Account) -> Option<u64> {
    account.group_id.map(|group| group.0).filter(|id| *id > 0)
}

fn derivation_paths(base: &str) -> Vec<String> {
    let trimmed = base.trim();
    match trimmed.rsplit_once('/') {
        Some((prefix, last)) if last.parse::<u32>().is_ok() => {
            (0..10).map(|index| format!("{prefix}/{index}")).collect()
        }
        _ => vec![trimmed.to_string()],
    }
}

fn sorted_group_refs(groups: &[AccountGroup]) -> Vec<&AccountGroup> {
    let mut sorted: Vec<&AccountGroup> = groups.iter().collect();
    sorted.sort_by_key(|group| (group.display_order, group.group_identity.0));
    sorted
}

fn bucket_refs<'a>(accounts: &'a [Account], group_id: Option<u64>) -> Vec<&'a Account> {
    let mut bucket: Vec<&Account> = accounts
        .iter()
        .filter(|account| normalized_group_id(account) == group_id)
        .collect();
    bucket.sort_by_key(|account| (account.display_order, account.account_identity.0));
    bucket
}

fn account_matches(account: &Account, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query = query.to_lowercase();
    if account.name.to_lowercase().contains(&query) {
        return true;
    }
    let address = match &account.metadata {
        WalletType::Safe(wallet) => wallet.evm_address.to_string(),
        WalletType::EOA(wallet) => wallet.evm_address.to_string(),
        WalletType::View(wallet) => wallet.evm_address.to_string(),
        WalletType::Railgun(wallet) => wallet.railgun_address.clone(),
    };
    address.to_lowercase().contains(&query)
}

pub struct App {
    pub tab: Tab,
    pub account_panel: AccountPanel,
    pub account_focus: AccountFocus,
    pub account_scroll: usize,
    pub accounts: Vec<Account>,
    pub groups: Vec<AccountGroup>,
    pub collapsed_groups: HashSet<Option<u64>>,
    pub move_mode: Option<MoveMode>,
    pub display_currency: String,
    pub theme: Theme,
    pub colored_assets: bool,
    pub command_palette: Option<CommandPalette>,
    pub networks: Vec<Network>,
    pub assets: HashMap<String, Asset>,
    pub asset_icons: HashMap<String, AssetIconData>,
    pub rpc_states: HashMap<u64, ResourceState<RpcPoolStats>>,
    pub asset_quotes: HashMap<String, ResourceState<String>>,
    balance_cache: HashMap<(String, u64), ResourceState<AccountBalances>>,
    asset_quote_cache: HashMap<(String, String), ResourceState<String>>,
    pub balance_states: HashMap<u64, ResourceState<AccountBalances>>,
    balance_refreshing: HashSet<u64>,
    pub defi_states: HashMap<u64, ResourceState<DefiResult>>,
    pub tx_states: HashMap<u64, ResourceState<Vec<Tx>>>,
    pub settings: SettingsState,
    pub settings_state: ResourceState<SettingsSnapshot>,
    pub form: Option<ActiveForm>,
    pub show_help: bool,
    pub account_filter: String,
    pub filter_input: bool,
    pub tx_index: usize,
    pub tx_detail: bool,
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
            account_scroll: 0,
            account_focus: AccountFocus::Sidebar,
            accounts: Vec::new(),
            groups: Vec::new(),
            collapsed_groups: HashSet::new(),
            move_mode: None,
            display_currency: "fiat:usd".to_string(),
            theme: Theme::default(),
            colored_assets: true,
            command_palette: None,
            networks: Vec::new(),
            assets: HashMap::new(),
            asset_icons: HashMap::new(),
            rpc_states: HashMap::new(),
            asset_quotes: HashMap::new(),
            balance_cache: HashMap::new(),
            asset_quote_cache: HashMap::new(),
            balance_states: HashMap::new(),
            balance_refreshing: HashSet::new(),
            defi_states: HashMap::new(),
            tx_states: HashMap::new(),
            settings: SettingsState::new(),
            settings_state: ResourceState::Idle,
            form: None,
            show_help: false,
            account_filter: String::new(),
            filter_input: false,
            tx_index: 0,
            tx_detail: false,
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

    pub fn set_account_panel(&mut self, panel: AccountPanel) {
        if self.account_panel != panel {
            self.account_scroll = 0;
        }
        self.account_panel = panel;
    }

    pub fn handle_resize(&mut self) {
        if let Some(renderer) = &mut self.icon_renderer {
            renderer.handle_resize();
        }
    }

    pub fn layout_view(&self) -> (&[AccountGroup], &[Account]) {
        match &self.move_mode {
            Some(mode) => (&mode.groups, &mode.accounts),
            None => (&self.groups, &self.accounts),
        }
    }

    pub fn account_rows(&self) -> Vec<AccountListRow> {
        let editing = self.move_mode.is_some();
        let filtering = !editing && !self.account_filter.is_empty();
        let (groups, accounts) = self.layout_view();
        let mut rows = Vec::new();

        for group in sorted_group_refs(groups) {
            let group_id = Some(group.group_identity.0);
            let mut members = bucket_refs(accounts, group_id);
            if filtering {
                members.retain(|account| account_matches(account, &self.account_filter));
                if members.is_empty() {
                    continue;
                }
            }
            let collapsed = !editing && !filtering && self.collapsed_groups.contains(&group_id);
            rows.push(AccountListRow::GroupHeader {
                group_id,
                name: group.name.clone(),
                collapsed,
                count: members.len(),
            });
            if collapsed {
                continue;
            }
            if members.is_empty() {
                rows.push(AccountListRow::EmptyGroup);
            }
            rows.extend(members.into_iter().map(|account| AccountListRow::Account {
                account_id: account.account_identity.0,
            }));
        }

        let mut ungrouped = bucket_refs(accounts, None);
        if filtering {
            ungrouped.retain(|account| account_matches(account, &self.account_filter));
            if !ungrouped.is_empty() {
                if !rows.is_empty() {
                    rows.push(AccountListRow::GroupHeader {
                        group_id: None,
                        name: "Ungrouped".to_string(),
                        collapsed: false,
                        count: ungrouped.len(),
                    });
                }
                rows.extend(
                    ungrouped
                        .into_iter()
                        .map(|account| AccountListRow::Account {
                            account_id: account.account_identity.0,
                        }),
                );
            }
            return rows;
        }
        if groups.is_empty() {
            rows.extend(
                ungrouped
                    .into_iter()
                    .map(|account| AccountListRow::Account {
                        account_id: account.account_identity.0,
                    }),
            );
        } else if editing || !ungrouped.is_empty() {
            let collapsed = !editing && self.collapsed_groups.contains(&None);
            rows.push(AccountListRow::GroupHeader {
                group_id: None,
                name: "Ungrouped".to_string(),
                collapsed,
                count: ungrouped.len(),
            });
            if !collapsed {
                if editing && ungrouped.is_empty() {
                    rows.push(AccountListRow::EmptyGroup);
                }
                rows.extend(
                    ungrouped
                        .into_iter()
                        .map(|account| AccountListRow::Account {
                            account_id: account.account_identity.0,
                        }),
                );
            }
        }

        rows
    }

    pub fn is_grabbed_row(&self, row: &AccountListRow) -> bool {
        let Some(mode) = &self.move_mode else {
            return false;
        };
        match (mode.grabbed, row) {
            (Some(GrabbedItem::Account(id)), AccountListRow::Account { account_id }) => {
                id == *account_id
            }
            (
                Some(GrabbedItem::Group(id)),
                AccountListRow::GroupHeader {
                    group_id: Some(group_id),
                    ..
                },
            ) => id == *group_id,
            _ => false,
        }
    }

    pub fn selected_list_row(&self) -> Option<AccountListRow> {
        self.account_rows().get(self.list_index).cloned()
    }

    pub fn account_by_id(&self, account_id: u64) -> Option<&Account> {
        self.accounts
            .iter()
            .find(|account| account.account_identity.0 == account_id)
    }

    pub fn toggle_group_collapsed(&mut self, group_id: Option<u64>) {
        if !self.collapsed_groups.remove(&group_id) {
            self.collapsed_groups.insert(group_id);
        }
        self.clamp_list_index();
    }

    fn enter_move_mode(&mut self) {
        if self.move_mode.is_none() {
            self.account_filter.clear();
            self.filter_input = false;
            self.move_mode = Some(MoveMode {
                groups: self.groups.clone(),
                accounts: self.accounts.clone(),
                grabbed: None,
                dirty: false,
            });
            self.clamp_list_index();
        }
    }

    fn cancel_move_mode(&mut self) {
        self.move_mode = None;
        self.clamp_list_index();
    }

    fn handle_move_mode_key(&mut self, code: KeyCode) -> KeyAction {
        match code {
            KeyCode::Char('q') => KeyAction::Quit,
            KeyCode::Esc => {
                let grabbed = self
                    .move_mode
                    .as_ref()
                    .is_some_and(|mode| mode.grabbed.is_some());
                if grabbed {
                    if let Some(mode) = &mut self.move_mode {
                        mode.grabbed = None;
                    }
                } else {
                    self.cancel_move_mode();
                }
                KeyAction::None
            }
            KeyCode::Char('m') => {
                self.cancel_move_mode();
                KeyAction::None
            }
            KeyCode::Char('s') => {
                let Some(mode) = &self.move_mode else {
                    return KeyAction::None;
                };
                if mode.dirty {
                    let update =
                        super::layout_edit::build_layout_update(&mode.groups, &mode.accounts);
                    self.notice = Some("Saving layout…".to_string());
                    KeyAction::CommitLayout(update)
                } else {
                    self.cancel_move_mode();
                    KeyAction::None
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_or_select(-1);
                KeyAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_or_select(1);
                KeyAction::None
            }
            KeyCode::PageUp => {
                self.move_selection(-10);
                KeyAction::None
            }
            KeyCode::PageDown => {
                self.move_selection(10);
                KeyAction::None
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.toggle_grab();
                KeyAction::None
            }
            KeyCode::Char('g') => self.open_group_create(),
            KeyCode::Char('e') => self.open_group_rename_selected(),
            KeyCode::Char('x') => self.open_group_delete_selected(),
            _ => KeyAction::None,
        }
    }

    fn move_or_select(&mut self, delta: i32) {
        let grabbed = self.move_mode.as_ref().and_then(|mode| mode.grabbed);
        match grabbed {
            Some(GrabbedItem::Group(group_id)) => {
                if let Some(mode) = &mut self.move_mode {
                    if super::layout_edit::move_group(&mut mode.groups, group_id, delta) {
                        mode.dirty = true;
                    }
                }
                self.follow_grabbed();
            }
            Some(GrabbedItem::Account(account_id)) => {
                self.move_grabbed_account(account_id, delta);
                self.follow_grabbed();
            }
            None => self.move_selection(delta),
        }
        let height = table_body_height(self.layout.body);
        self.reconcile_scroll(height);
    }

    fn move_grabbed_account(&mut self, account_id: u64, delta: i32) {
        let Some(mode) = &mut self.move_mode else {
            return;
        };
        if super::layout_edit::move_account_within(&mut mode.accounts, account_id, delta) {
            mode.dirty = true;
            return;
        }

        let current = mode
            .accounts
            .iter()
            .find(|account| account.account_identity.0 == account_id)
            .map(normalized_group_id);
        let Some(current) = current else {
            return;
        };
        let mut sequence: Vec<Option<u64>> = sorted_group_refs(&mode.groups)
            .iter()
            .map(|group| Some(group.group_identity.0))
            .collect();
        sequence.push(None);
        let Some(position) = sequence.iter().position(|bucket| *bucket == current) else {
            return;
        };

        let target = if delta > 0 {
            let Some(bucket) = sequence.get(position + 1) else {
                return;
            };
            *bucket
        } else {
            let Some(previous) = position.checked_sub(1) else {
                return;
            };
            sequence[previous]
        };
        let index = if delta > 0 {
            0
        } else {
            super::layout_edit::bucket_ids(&mode.accounts, target).len()
        };
        if super::layout_edit::move_account_to_group(&mut mode.accounts, account_id, target, index)
        {
            mode.dirty = true;
        }
    }

    fn follow_grabbed(&mut self) {
        let Some(grabbed) = self.move_mode.as_ref().and_then(|mode| mode.grabbed) else {
            return;
        };
        let rows = self.account_rows();
        let target = rows.iter().position(|row| match (grabbed, row) {
            (GrabbedItem::Account(id), AccountListRow::Account { account_id }) => id == *account_id,
            (
                GrabbedItem::Group(id),
                AccountListRow::GroupHeader {
                    group_id: Some(group_id),
                    ..
                },
            ) => id == *group_id,
            _ => false,
        });
        if let Some(index) = target {
            self.list_index = index;
        }
    }

    fn toggle_grab(&mut self) {
        let selected = self.selected_list_row();
        let Some(mode) = &mut self.move_mode else {
            return;
        };
        if mode.grabbed.is_some() {
            mode.grabbed = None;
            return;
        }
        mode.grabbed = match selected {
            Some(AccountListRow::Account { account_id }) => Some(GrabbedItem::Account(account_id)),
            Some(AccountListRow::GroupHeader {
                group_id: Some(group_id),
                ..
            }) => Some(GrabbedItem::Group(group_id)),
            _ => None,
        };
    }

    fn sync_move_mode_draft(&mut self) {
        let Some(mode) = self.move_mode.as_mut() else {
            return;
        };

        let canonical_groups: HashSet<u64> = self
            .groups
            .iter()
            .map(|group| group.group_identity.0)
            .collect();
        mode.groups
            .retain(|group| canonical_groups.contains(&group.group_identity.0));
        for group in &self.groups {
            if let Some(draft) = mode
                .groups
                .iter_mut()
                .find(|draft| draft.group_identity.0 == group.group_identity.0)
            {
                draft.name = group.name.clone();
            } else {
                let next_order = mode
                    .groups
                    .iter()
                    .map(|draft| draft.display_order + 1)
                    .max()
                    .unwrap_or(0);
                let mut added = group.clone();
                added.display_order = next_order;
                mode.groups.push(added);
            }
        }

        let draft_groups: HashSet<u64> = mode
            .groups
            .iter()
            .map(|group| group.group_identity.0)
            .collect();
        let canonical_accounts: HashSet<u64> = self
            .accounts
            .iter()
            .map(|account| account.account_identity.0)
            .collect();
        mode.accounts
            .retain(|account| canonical_accounts.contains(&account.account_identity.0));
        for account in &mut mode.accounts {
            if let Some(group_id) = normalized_group_id(account) {
                if !draft_groups.contains(&group_id) {
                    account.group_id = None;
                }
            }
        }
        for account in &self.accounts {
            if !mode
                .accounts
                .iter()
                .any(|draft| draft.account_identity.0 == account.account_identity.0)
            {
                mode.accounts.push(account.clone());
            }
        }

        if let Some(grabbed) = mode.grabbed {
            let still_present = match grabbed {
                GrabbedItem::Account(id) => canonical_accounts.contains(&id),
                GrabbedItem::Group(id) => draft_groups.contains(&id),
            };
            if !still_present {
                mode.grabbed = None;
            }
        }
    }

    fn open_group_create(&mut self) -> KeyAction {
        self.form = Some(ActiveForm::open_group_name(None, ""));
        KeyAction::None
    }

    fn open_group_rename_selected(&mut self) -> KeyAction {
        match self.selected_list_row() {
            Some(AccountListRow::GroupHeader {
                group_id: Some(group_id),
                name,
                ..
            }) => {
                self.form = Some(ActiveForm::open_group_name(Some(group_id), &name));
            }
            Some(AccountListRow::Account { account_id }) => {
                if let Some(account) = self.account_by_id(account_id) {
                    let name = account.name.clone();
                    self.form = Some(ActiveForm::open_rename_account(account_id, &name));
                }
            }
            _ => {}
        }
        KeyAction::None
    }

    fn open_group_delete_selected(&mut self) -> KeyAction {
        match self.selected_list_row() {
            Some(AccountListRow::GroupHeader {
                group_id: Some(group_id),
                name,
                ..
            }) => {
                self.form = Some(ActiveForm::open_confirm_delete_group(group_id, name));
            }
            Some(AccountListRow::Account { account_id }) => {
                if let Some(account) = self.account_by_id(account_id) {
                    let name = account.name.clone();
                    self.form = Some(ActiveForm::open_confirm_delete_account(account_id, name));
                }
            }
            _ => {}
        }
        KeyAction::None
    }

    fn open_account_networks_selected(&mut self) -> KeyAction {
        let Some(AccountListRow::Account { account_id }) = self.selected_list_row() else {
            return KeyAction::None;
        };
        let Some(account) = self.account_by_id(account_id) else {
            return KeyAction::None;
        };
        let member: HashSet<u64> = account.networks.iter().map(|network| network.0).collect();
        let options: Vec<(u64, String, bool)> = self
            .networks
            .iter()
            .map(|network| {
                (
                    network.network_identity.0,
                    network.network_name.clone(),
                    member.contains(&network.network_identity.0),
                )
            })
            .collect();
        if options.is_empty() {
            self.notice = Some("No networks configured".to_string());
            return KeyAction::None;
        }
        self.form = Some(ActiveForm::open_account_networks(account_id, options));
        KeyAction::None
    }

    pub fn needs_refresh(&self) -> bool {
        !self.refresh_in_flight
            && self.form.is_none()
            && self.move_mode.is_none()
            && (self.dirty || self.last_refresh.elapsed() >= REFRESH_INTERVAL)
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

    // Lighter than begin_refresh: keeps the generation and core state so in-flight
    // updates still apply; only balances re-fetch, with cached values left on screen.
    pub fn begin_balance_refresh(&mut self) -> u64 {
        let account_ids: Vec<u64> = self
            .accounts
            .iter()
            .map(|account| account.account_identity.0)
            .collect();
        // with no accounts nothing will arrive to clear the in-flight flag
        if !account_ids.is_empty() {
            self.refresh_in_flight = true;
        }
        for account_id in account_ids {
            self.prepare_balance_fetch(account_id);
        }
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
            BackgroundUpdate::LayoutLoaded {
                generation,
                groups,
                accounts,
                notice,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.accounts = accounts;
                self.groups = groups;
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
                self.sync_move_mode_draft();
                self.update_status();
            }
            BackgroundUpdate::LayoutCommitted {
                generation,
                groups,
                accounts,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.groups = groups;
                self.accounts = accounts;
                self.move_mode = None;
                self.notice = Some("Layout saved".to_string());
                self.clamp_list_index();
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
                self.asset_icons
                    .retain(|identity, _| self.assets.contains_key(identity));
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
                display_currency,
                state,
                refreshing,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.balance_cache
                    .insert((display_currency.clone(), account_id), state.clone());
                if display_currency == self.display_currency {
                    self.balance_states.insert(account_id, state);
                }
                if refreshing {
                    self.balance_refreshing.insert(account_id);
                } else {
                    self.balance_refreshing.remove(&account_id);
                }
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
            BackgroundUpdate::AssetQuote {
                generation,
                identity,
                display_currency,
                state,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.asset_quote_cache
                    .insert((display_currency.clone(), identity.clone()), state.clone());
                if display_currency == self.display_currency {
                    self.asset_quotes.insert(identity, state);
                }
            }
            BackgroundUpdate::AssetIcon {
                generation,
                identity,
                icon,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                if let Some(icon) = icon {
                    self.asset_icons.insert(identity, icon);
                }
            }
            BackgroundUpdate::NetworkPresets {
                generation,
                presets,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                let existing: HashSet<u64> = self
                    .networks
                    .iter()
                    .map(|network| network.network_identity.0)
                    .collect();
                let available: Vec<Network> = presets
                    .into_iter()
                    .filter(|preset| !existing.contains(&preset.network_identity.0))
                    .collect();

                if matches!(self.form, Some(ActiveForm::AddNetworkMode { .. })) {
                    if available.is_empty() {
                        self.settings.notice = Some("No network presets available".to_string());
                        self.form = None;
                    } else {
                        self.form = Some(ActiveForm::AddNetworkPreset {
                            presets: available,
                            selected: 0,
                        });
                    }
                }
            }
            BackgroundUpdate::DerivationPath { generation, path } => {
                if generation != self.refresh_generation {
                    return;
                }
                if let Some(ActiveForm::AddAccountMnemonic {
                    form,
                    deriving: false,
                }) = &mut self.form
                {
                    if let Some(field) = form.fields.get_mut(2) {
                        if field.value.is_empty() {
                            field.value = path;
                        }
                    }
                }
            }
            BackgroundUpdate::EnsResolved {
                generation,
                name,
                address,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                if let Some(ActiveForm::AddAccountAddress {
                    ens_hint,
                    ens_pending,
                    ..
                }) = &mut self.form
                {
                    if ens_pending.as_deref() == Some(name.as_str()) {
                        *ens_hint = address;
                    }
                }
            }
            BackgroundUpdate::EnsReversed {
                generation,
                address,
                name,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                if let Some(ActiveForm::AddAccountAddress {
                    ens_primary,
                    ens_primary_pending,
                    ..
                }) = &mut self.form
                {
                    if ens_primary_pending.as_deref() == Some(address.to_lowercase().as_str()) {
                        *ens_primary = name;
                    }
                }
            }
            BackgroundUpdate::GeneratedMnemonic {
                generation,
                mnemonic,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                if let Some(ActiveForm::AddAccountMnemonic {
                    form,
                    deriving: false,
                }) = &mut self.form
                {
                    if let Some(field) = form.fields.get_mut(1) {
                        if field.value.is_empty() {
                            field.value = mnemonic;
                        }
                    }
                }
            }
            BackgroundUpdate::DerivedAddresses {
                generation,
                name,
                result,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                let Some(ActiveForm::AddAccountMnemonic { deriving, .. }) = &mut self.form else {
                    return;
                };
                match result {
                    Ok(options) if !options.is_empty() => {
                        self.form = Some(ActiveForm::AddAccountPickAddress {
                            name,
                            options,
                            selected: 0,
                        });
                    }
                    Ok(_) => {
                        *deriving = false;
                        self.notice = Some("No addresses derived".to_string());
                    }
                    Err(error) => {
                        *deriving = false;
                        self.notice = Some(format!("Derivation failed: {error}"));
                    }
                }
            }
            BackgroundUpdate::AccountAssets {
                generation,
                account_id,
                unlink,
                identities,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.open_asset_picker(account_id, unlink, &identities);
            }
            BackgroundUpdate::QuoterDiscovered {
                generation,
                token_a,
                token_b,
                result,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                if !matches!(self.form, Some(ActiveForm::QuoterDiscovering)) {
                    return;
                }
                let response = match result {
                    Ok(response) => response,
                    Err(error) => {
                        self.settings.notice = Some(format!("Quoter discovery failed: {error}"));
                        self.form = None;
                        return;
                    }
                };

                use koi::models::quoter::{
                    Erc4626QuoterConfig, QuoterConfig, UniswapV2QuoterConfig, UniswapV3QuoterConfig,
                };
                let mut options = Vec::new();
                if let (Some(pair), Some(token_b)) = (&response.uniswap_v2, &token_b) {
                    options.push(super::form::QuoterSourceOption {
                        label: format!("Uniswap V2 pair {}", pair.pair_address),
                        token_b: token_b.clone(),
                        config: QuoterConfig::UniswapV2(UniswapV2QuoterConfig {
                            pair_address: pair.pair_address.clone(),
                        }),
                    });
                }
                if let (Some(pools), Some(token_b)) = (&response.uniswap_v3, &token_b) {
                    for pool in pools {
                        options.push(super::form::QuoterSourceOption {
                            label: format!(
                                "Uniswap V3 {:.2}% pool {}",
                                pool.fee as f64 / 10_000.0,
                                pool.pool_address
                            ),
                            token_b: token_b.clone(),
                            config: QuoterConfig::UniswapV3(UniswapV3QuoterConfig {
                                pool_address: pool.pool_address.clone(),
                            }),
                        });
                    }
                }
                if let Some(underlying) = &response.erc4626 {
                    options.push(super::form::QuoterSourceOption {
                        label: format!("ERC-4626 vault → {underlying}"),
                        token_b: underlying.to_string(),
                        config: QuoterConfig::Erc4626(Erc4626QuoterConfig {}),
                    });
                }

                if options.is_empty() {
                    self.settings.notice =
                        Some("No quoter routes discovered for that pair".to_string());
                    self.form = None;
                } else {
                    self.form = Some(ActiveForm::PickQuoterSource {
                        token_a,
                        options,
                        selected: 0,
                    });
                }
            }
        }
    }

    fn open_asset_picker(&mut self, account_id: u64, unlink: bool, linked: &[String]) {
        let label = |identity: &str| {
            self.assets
                .get(identity)
                .map(|asset| format!("{} · {}", asset.asset_symbol, asset.asset_name))
                .unwrap_or_else(|| identity.to_string())
        };

        let options: Vec<(String, String)> = if unlink {
            linked
                .iter()
                .map(|identity| (identity.clone(), label(identity)))
                .collect()
        } else {
            let member_networks: HashSet<u64> = self
                .account_by_id(account_id)
                .map(|account| account.networks.iter().map(|network| network.0).collect())
                .unwrap_or_default();
            let mut candidates: Vec<(String, String)> = self
                .assets
                .values()
                .filter(|asset| {
                    let identity = asset.asset_identity.to_string();
                    !linked.contains(&identity)
                        && asset
                            .asset_identity
                            .unwrap_network()
                            .is_some_and(|network| member_networks.contains(&network.0))
                })
                .map(|asset| {
                    let identity = asset.asset_identity.to_string();
                    let label = label(&identity);
                    (identity, label)
                })
                .collect();
            candidates.sort();
            candidates
        };

        if options.is_empty() {
            self.notice = Some(
                if unlink {
                    "No linked assets to remove"
                } else {
                    "No linkable assets on this account's networks"
                }
                .to_string(),
            );
            return;
        }

        self.form = Some(ActiveForm::PickAccountAsset {
            account_id,
            unlink,
            options,
            selected: 0,
        });
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

    pub fn set_display_currency(&mut self, display_currency: String) {
        self.display_currency = display_currency;
        for account in &self.accounts {
            let account_id = account.account_identity.0;
            let state = self
                .balance_cache
                .get(&(self.display_currency.clone(), account_id))
                .cloned()
                .unwrap_or(ResourceState::Loading);
            self.balance_states.insert(account_id, state);
        }
        for identity in self.quote_asset_identities() {
            let state = self
                .asset_quote_cache
                .get(&(self.display_currency.clone(), identity.clone()))
                .cloned()
                .unwrap_or(ResourceState::Loading);
            self.asset_quotes.insert(identity, state);
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
        if self.form.is_some() || self.show_help || self.command_palette.is_some() {
            return KeyAction::None;
        }

        if let Some(mode) = &self.move_mode {
            let grabbed = mode.grabbed.is_some();
            match event.kind {
                MouseEventKind::Moved if !grabbed => {
                    if let Some(index) = self.layout.list_row_at(event.column, event.row) {
                        self.list_index = index;
                    }
                }
                MouseEventKind::Down(MouseButton::Left) => {
                    if let Some(index) = self.layout.list_row_at(event.column, event.row) {
                        self.list_index = index;
                        self.toggle_grab();
                    }
                }
                MouseEventKind::ScrollUp if !grabbed => self.move_or_select(-3),
                MouseEventKind::ScrollDown if !grabbed => self.move_or_select(3),
                _ => {}
            }
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
                self.set_account_panel(panel);
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
            self.set_account_panel(panel);
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
            if self.account_panel == AccountPanel::Transactions {
                let len = self.selected_account_txs().map(<[Tx]>::len).unwrap_or(0);
                let current = self.clamped_tx_index();
                self.tx_index = if delta < 0 {
                    current.saturating_sub(delta.unsigned_abs() as usize)
                } else {
                    (current + delta as usize).min(len.saturating_sub(1))
                };
            } else {
                self.account_scroll = self.account_scroll.saturating_add_signed(delta as isize);
            }
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
        if self.show_help {
            self.show_help = false;
            return KeyAction::None;
        }

        if self.form.is_some() {
            return self.handle_form_key(code);
        }

        if self.command_palette.is_some() {
            return self.handle_command_palette_key(code);
        }

        if code == KeyCode::Char('?') && !self.filter_input {
            self.show_help = true;
            return KeyAction::None;
        }

        if self.move_mode.is_some() {
            return self.handle_move_mode_key(code);
        }

        if self.filter_input {
            match code {
                KeyCode::Esc => {
                    self.account_filter.clear();
                    self.filter_input = false;
                }
                KeyCode::Enter => self.filter_input = false,
                KeyCode::Backspace => {
                    if self.account_filter.pop().is_none() {
                        self.filter_input = false;
                    }
                }
                KeyCode::Char(ch) if !ch.is_control() => self.account_filter.push(ch),
                _ => {}
            }
            self.clamp_list_index();
            return KeyAction::None;
        }

        match code {
            KeyCode::Char('q') => KeyAction::Quit,
            KeyCode::Char(':') => {
                self.command_palette = Some(CommandPalette {
                    query: String::new(),
                    selected: 0,
                });
                KeyAction::None
            }
            KeyCode::Char('/') if self.tab == Tab::Accounts && self.selected_account.is_none() => {
                self.filter_input = true;
                KeyAction::None
            }
            KeyCode::Esc | KeyCode::Char('b') => {
                if self.tx_detail {
                    self.tx_detail = false;
                } else if code == KeyCode::Esc
                    && !self.account_filter.is_empty()
                    && self.selected_account.is_none()
                {
                    self.account_filter.clear();
                    self.clamp_list_index();
                } else if self.selected_account.is_some() {
                    self.selected_account = None;
                    self.set_account_panel(AccountPanel::Overview);
                    self.account_focus = AccountFocus::Sidebar;
                    self.tx_index = 0;
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
                KeyAction::RefreshQuotes
            }
            KeyCode::Char('3') => {
                self.switch_tab(Tab::Prices);
                if matches!(self.settings_state, ResourceState::Ready(_)) {
                    KeyAction::RefreshQuotes
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
            KeyCode::Left | KeyCode::Char('h') if self.selected_account.is_some() => {
                self.account_focus = AccountFocus::Sidebar;
                KeyAction::None
            }
            KeyCode::Right | KeyCode::Char('l') if self.selected_account.is_some() => {
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
            KeyCode::Up | KeyCode::Char('k')
                if self.selected_account.is_some()
                    && self.account_focus == AccountFocus::Content
                    && self.account_panel == AccountPanel::Transactions =>
            {
                self.tx_index = self.clamped_tx_index().saturating_sub(1);
                KeyAction::None
            }
            KeyCode::Down | KeyCode::Char('j')
                if self.selected_account.is_some()
                    && self.account_focus == AccountFocus::Content
                    && self.account_panel == AccountPanel::Transactions =>
            {
                let len = self.selected_account_txs().map(<[Tx]>::len).unwrap_or(0);
                self.tx_index = (self.clamped_tx_index() + 1).min(len.saturating_sub(1));
                KeyAction::None
            }
            KeyCode::Enter
                if self.selected_account.is_some()
                    && self.account_focus == AccountFocus::Content
                    && self.account_panel == AccountPanel::Transactions =>
            {
                if self
                    .selected_account_txs()
                    .is_some_and(|transactions| !transactions.is_empty())
                {
                    self.tx_index = self.clamped_tx_index();
                    self.tx_detail = true;
                }
                KeyAction::None
            }
            KeyCode::Up | KeyCode::Char('k')
                if self.selected_account.is_some()
                    && self.account_focus == AccountFocus::Content =>
            {
                self.account_scroll = self.account_scroll.saturating_sub(1);
                KeyAction::None
            }
            KeyCode::Down | KeyCode::Char('j')
                if self.selected_account.is_some()
                    && self.account_focus == AccountFocus::Content =>
            {
                self.account_scroll = self.account_scroll.saturating_add(1);
                KeyAction::None
            }
            KeyCode::PageUp
                if self.selected_account.is_some()
                    && self.account_focus == AccountFocus::Content =>
            {
                self.account_scroll = self.account_scroll.saturating_sub(10);
                KeyAction::None
            }
            KeyCode::PageDown
                if self.selected_account.is_some()
                    && self.account_focus == AccountFocus::Content =>
            {
                self.account_scroll = self.account_scroll.saturating_add(10);
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
            KeyCode::Char('g') if self.tab == Tab::Accounts && self.selected_account.is_none() => {
                self.open_group_create()
            }
            KeyCode::Char('e') if self.tab == Tab::Accounts && self.selected_account.is_none() => {
                self.open_group_rename_selected()
            }
            KeyCode::Char('x') if self.tab == Tab::Accounts && self.selected_account.is_none() => {
                self.open_group_delete_selected()
            }
            KeyCode::Char('m') if self.tab == Tab::Accounts && self.selected_account.is_none() => {
                self.enter_move_mode();
                KeyAction::None
            }
            KeyCode::Char('$') => self.open_currency_picker(),
            KeyCode::Char('c')
                if self.tab == Tab::Settings
                    && self.settings.section() == SettingsSection::General =>
            {
                self.colored_assets = !self.colored_assets;
                KeyAction::None
            }
            KeyCode::Char('t' | 'T')
                if self.tab == Tab::Settings
                    && self.settings.section() == SettingsSection::General =>
            {
                KeyAction::SetTheme(self.theme.next())
            }
            KeyCode::Char('N') if self.tab == Tab::Accounts && self.selected_account.is_none() => {
                self.open_account_networks_selected()
            }
            KeyCode::Char('n')
                if self.selected_account.is_some()
                    && self.account_panel == AccountPanel::Assets =>
            {
                KeyAction::OpenAssetPicker {
                    account_id: self.selected_account.unwrap_or_default(),
                    unlink: false,
                }
            }
            KeyCode::Char('x')
                if self.selected_account.is_some()
                    && self.account_panel == AccountPanel::Assets =>
            {
                KeyAction::OpenAssetPicker {
                    account_id: self.selected_account.unwrap_or_default(),
                    unlink: true,
                }
            }
            KeyCode::Char('x') if self.uses_resource_rows() && self.selected_account.is_none() => {
                self.settings_action()
            }
            KeyCode::Char('e') if self.uses_resource_rows() && self.selected_account.is_none() => {
                self.settings_edit_action()
            }
            KeyCode::Char('n') if self.selected_account.is_none() => self.open_add_form(),
            KeyCode::Char('a') if self.selected_account.is_some() => {
                self.set_account_panel(AccountPanel::Assets);
                self.account_focus = AccountFocus::Sidebar;
                KeyAction::None
            }
            KeyCode::Char('d') if self.selected_account.is_some() => {
                self.set_account_panel(AccountPanel::Defi);
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
                self.set_account_panel(AccountPanel::Transactions);
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
            KeyCode::Char('o') if self.selected_account.is_some() => {
                self.set_account_panel(AccountPanel::Overview);
                self.account_focus = AccountFocus::Sidebar;
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    pub fn command_choices(&self) -> Vec<CommandChoice> {
        let mut choices = vec![
            CommandChoice::Tab(Tab::Accounts),
            CommandChoice::Tab(Tab::Assets),
            CommandChoice::Tab(Tab::Networks),
            CommandChoice::Tab(Tab::Settings),
            CommandChoice::Theme(Theme::Dark),
            CommandChoice::Theme(Theme::Midnight),
            CommandChoice::Theme(Theme::Light),
            CommandChoice::Theme(Theme::Paper),
            CommandChoice::Theme(Theme::Terminal),
            CommandChoice::DisplayCurrency,
        ];
        choices.extend(
            self.accounts
                .iter()
                .map(|account| CommandChoice::Account(account.account_identity.0)),
        );
        let query = self
            .command_palette
            .as_ref()
            .map(|palette| palette.query.to_lowercase())
            .unwrap_or_default();
        choices.retain(|choice| choice.label(self).to_lowercase().contains(&query));
        choices
    }

    fn handle_command_palette_key(&mut self, code: KeyCode) -> KeyAction {
        match code {
            KeyCode::Esc => self.command_palette = None,
            KeyCode::Up => {
                let len = self.command_choices().len();
                if let Some(palette) = &mut self.command_palette {
                    palette.selected = palette
                        .selected
                        .saturating_sub(1)
                        .min(len.saturating_sub(1));
                }
            }
            KeyCode::Down => {
                let len = self.command_choices().len();
                if let Some(palette) = &mut self.command_palette {
                    palette.selected = (palette.selected + 1).min(len.saturating_sub(1));
                }
            }
            KeyCode::Backspace => {
                if let Some(palette) = &mut self.command_palette {
                    palette.query.pop();
                    palette.selected = 0;
                }
            }
            KeyCode::Char(ch) if !ch.is_control() => {
                if let Some(palette) = &mut self.command_palette {
                    palette.query.push(ch);
                    palette.selected = 0;
                }
            }
            KeyCode::Enter => {
                let selected = self
                    .command_palette
                    .as_ref()
                    .map(|palette| palette.selected);
                let choice = selected.and_then(|index| self.command_choices().get(index).cloned());
                self.command_palette = None;
                return match choice {
                    Some(CommandChoice::Tab(tab)) => self.navigate_to_tab(tab),
                    Some(CommandChoice::Account(account_id)) => {
                        self.open_account_from_palette(account_id)
                    }
                    Some(CommandChoice::Theme(theme)) if theme != self.theme => {
                        KeyAction::SetTheme(theme)
                    }
                    Some(CommandChoice::DisplayCurrency) => self.open_currency_picker(),
                    _ => KeyAction::None,
                };
            }
            _ => {}
        }
        KeyAction::None
    }

    fn navigate_to_tab(&mut self, tab: Tab) -> KeyAction {
        self.switch_tab(tab);
        match tab {
            Tab::Assets => KeyAction::RefreshQuotes,
            Tab::Prices | Tab::Settings
                if !matches!(self.settings_state, ResourceState::Ready(_)) =>
            {
                KeyAction::RefreshSettings
            }
            _ => KeyAction::None,
        }
    }

    fn open_account_from_palette(&mut self, account_id: u64) -> KeyAction {
        self.switch_tab(Tab::Accounts);
        if let Some(index) = self.account_rows().iter().position(
            |row| matches!(row, AccountListRow::Account { account_id: id } if *id == account_id),
        ) {
            self.list_index = index;
            self.activate()
        } else {
            KeyAction::None
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
        self.set_account_panel(AccountPanel::Overview);
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
            Tab::Accounts => {
                self.form = Some(ActiveForm::open_add_account());
                KeyAction::None
            }
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
            Tab::Prices => {
                let options = self.quoter_token_options(None);
                if options.is_empty() {
                    self.settings.notice =
                        Some("No ERC-20 assets configured to build a quoter from".to_string());
                    return KeyAction::None;
                }
                self.form = Some(ActiveForm::PickQuoterToken {
                    token_a: None,
                    options,
                    selected: 0,
                });
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
                self.form = Some(form);
                KeyAction::FetchNetworkPresets
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
            FormAction::SubmitGroupName { group_id, name } => {
                self.form = None;
                match group_id {
                    Some(group_id) => KeyAction::RenameGroup(group_id, name),
                    None => KeyAction::CreateGroup(name),
                }
            }
            FormAction::ConfirmDeleteGroup(group_id) => {
                self.form = None;
                KeyAction::DeleteGroup(group_id)
            }
            FormAction::SubmitDisplayCurrency(identity) => {
                self.form = None;
                if identity == self.display_currency {
                    KeyAction::None
                } else {
                    KeyAction::SetDisplayCurrency(identity)
                }
            }
            FormAction::FetchDerivationPath => {
                self.form = Some(form);
                KeyAction::FetchDerivationPath
            }
            FormAction::GenerateMnemonic => {
                self.form = Some(form);
                KeyAction::GenerateMnemonic
            }
            FormAction::ResolveEns(name) => {
                self.form = Some(form);
                KeyAction::ResolveEns(name)
            }
            FormAction::ReverseEns(address) => {
                self.form = Some(form);
                KeyAction::ReverseEns(address)
            }
            FormAction::SubmitCreateAccount { name, wallet } => {
                self.form = None;
                self.notice = Some(format!("Creating account {name}…"));
                KeyAction::CreateAccount { name, wallet }
            }
            FormAction::DeriveMnemonic {
                name,
                mnemonic,
                base_path,
            } => {
                if let ActiveForm::AddAccountMnemonic { deriving, .. } = &mut form {
                    *deriving = true;
                }
                self.form = Some(form);
                KeyAction::DeriveAddresses {
                    name,
                    mnemonic,
                    paths: derivation_paths(&base_path),
                }
            }
            FormAction::CreateAccountFromKey { name, key } => {
                self.form = None;
                self.notice = Some(format!("Creating account {name}…"));
                KeyAction::CreateAccountFromKey { name, key }
            }
            FormAction::SubmitRenameAccount { account_id, name } => {
                self.form = None;
                KeyAction::RenameAccount(account_id, name)
            }
            FormAction::ConfirmDeleteAccount(account_id) => {
                self.form = None;
                KeyAction::DeleteAccount(account_id)
            }
            FormAction::SubmitAccountNetworks {
                account_id,
                networks,
            } => {
                self.form = None;
                KeyAction::UpdateAccountNetworks(account_id, networks)
            }
            FormAction::SubmitAccountAsset {
                account_id,
                unlink,
                identity,
            } => {
                self.form = None;
                KeyAction::AccountAssetAction {
                    account_id,
                    identity,
                    unlink,
                }
            }
            FormAction::SubmitEditEndpoint {
                network_id,
                endpoint_id,
                update,
            } => {
                self.form = None;
                KeyAction::UpdateEndpoint {
                    network_id,
                    endpoint_id,
                    update,
                }
            }
            FormAction::SubmitEditNetwork { network_id, update } => {
                self.form = None;
                KeyAction::UpdateNetwork { network_id, update }
            }
            FormAction::SubmitEditAsset { identity, update } => {
                self.form = None;
                KeyAction::UpdateAsset { identity, update }
            }
            FormAction::ConfirmDeleteNetwork(network_id) => {
                self.form = None;
                KeyAction::DeleteNetwork(network_id)
            }
            FormAction::PickQuoterTokenA(token_a) => {
                let mut options = self.quoter_token_options(Some(&token_a));
                options.insert(
                    0,
                    (
                        super::form::ERC4626_SENTINEL.to_string(),
                        "(no pair · discover ERC-4626 underlying)".to_string(),
                    ),
                );
                self.form = Some(ActiveForm::PickQuoterToken {
                    token_a: Some(token_a),
                    options,
                    selected: 0,
                });
                KeyAction::None
            }
            FormAction::DiscoverQuoter { token_a, token_b } => {
                self.form = Some(ActiveForm::QuoterDiscovering);
                KeyAction::DiscoverQuoter { token_a, token_b }
            }
            FormAction::SubmitCreateQuoter {
                token_a,
                token_b,
                config,
            } => {
                self.form = None;
                self.settings.notice = Some("Creating quoter…".to_string());
                KeyAction::CreateQuoter {
                    token_a,
                    token_b,
                    config,
                }
            }
        }
    }

    fn quoter_token_options(&self, exclude: Option<&str>) -> Vec<(String, String)> {
        let mut options: Vec<(String, String)> = self
            .assets
            .values()
            .filter(|asset| {
                let identity = asset.asset_identity.to_string();
                identity.starts_with("erc20:") && Some(identity.as_str()) != exclude
            })
            .map(|asset| {
                (
                    asset.asset_identity.to_string(),
                    format!("{} · {}", asset.asset_symbol, asset.asset_name),
                )
            })
            .collect();
        options.sort();
        options
    }

    pub fn open_currency_picker(&mut self) -> KeyAction {
        let mut options: Vec<Asset> = self
            .assets
            .values()
            .filter(|asset| asset.asset_identity.to_string().starts_with("fiat:"))
            .cloned()
            .collect();
        options.sort_by_key(|asset| asset.asset_identity.to_string());

        if options.is_empty() {
            self.settings.notice =
                Some("No fiat assets configured; add one on the Assets tab first".to_string());
            return KeyAction::None;
        }

        let selected = options
            .iter()
            .position(|asset| asset.asset_identity.to_string() == self.display_currency)
            .unwrap_or(0);
        self.form = Some(ActiveForm::PickCurrency { options, selected });
        KeyAction::None
    }

    pub fn display_asset(&self) -> Option<&Asset> {
        self.assets.get(&self.display_currency)
    }

    pub fn quote_asset_identities(&self) -> Vec<String> {
        let mut identities: Vec<String> = self
            .assets
            .keys()
            .filter(|identity| **identity != self.display_currency)
            .cloned()
            .collect();
        identities.sort();
        identities
    }

    pub fn prepare_asset_quotes(&mut self, identities: &[String]) {
        self.asset_quotes
            .retain(|identity, _| identities.contains(identity));
        for identity in identities {
            self.asset_quotes
                .entry(identity.clone())
                .or_insert(ResourceState::Loading);
        }
    }

    pub fn asset_24h_change(&self, identity: &str) -> Option<(String, String)> {
        self.balance_states.values().find_map(|state| {
            let ResourceState::Ready(balances) = state else {
                return None;
            };
            balances.balances.iter().find_map(|balance| {
                if balance.asset_identity.to_string() != identity {
                    return None;
                }
                Some((
                    balance.asset_quote.clone()?,
                    balance.asset_24h_quote.clone()?,
                ))
            })
        })
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
        self.set_account_panel(AccountPanel::ALL[next]);

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
            Tab::Accounts if self.selected_account.is_none() => self.account_rows().len(),
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

        let account_id = match self.selected_list_row() {
            Some(AccountListRow::GroupHeader { group_id, .. }) => {
                self.toggle_group_collapsed(group_id);
                return KeyAction::None;
            }
            Some(AccountListRow::Account { account_id }) => account_id,
            Some(AccountListRow::EmptyGroup) | None => return KeyAction::None,
        };

        self.selected_account = Some(account_id);
        self.set_account_panel(AccountPanel::Overview);
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
            Tab::Accounts if self.selected_account.is_none() => self.account_rows().len(),
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
            let len = self.account_rows().len();
            ensure_visible(&mut self.list_scroll, self.list_index, len, viewport_height);
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

    pub fn selected_account_txs(&self) -> Option<&[Tx]> {
        let account_id = self.selected_account?;
        match self.tx_states.get(&account_id) {
            Some(ResourceState::Ready(transactions)) => Some(transactions.as_slice()),
            _ => None,
        }
    }

    pub fn clamped_tx_index(&self) -> usize {
        let len = self.selected_account_txs().map(<[Tx]>::len).unwrap_or(0);
        self.tx_index.min(len.saturating_sub(1))
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
        if self.tab == Tab::Prices {
            return self.quote_asset_identities().len();
        }

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
                    if let Some(network) = self.networks.get(self.settings.row_index) {
                        self.form = Some(ActiveForm::open_confirm_delete_network(
                            network.network_identity.0,
                            network.network_name.clone(),
                        ));
                    }
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
            SettingsSection::PriceFeeds => {
                let Some((identity, enabled)) = self
                    .settings_snapshot()
                    .and_then(|snapshot| snapshot.quoters.get(self.settings.row_index))
                    .map(|quoter| (quoter.quoter_identity.clone(), quoter.enabled))
                else {
                    return KeyAction::None;
                };
                KeyAction::ToggleQuoter(identity, !enabled)
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
            SettingsSection::General => KeyAction::None,
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
            SettingsSection::Vendors | SettingsSection::PriceFeeds => self.settings_action(),
            SettingsSection::Networks => {
                if let Some(network_id) = self.settings.nested_network {
                    let endpoint = self
                        .settings_snapshot()
                        .and_then(|snapshot| snapshot.endpoints.get(&network_id))
                        .and_then(|endpoints| endpoints.get(self.settings.row_index));
                    if let Some(endpoint) = endpoint {
                        self.form = Some(ActiveForm::open_edit_endpoint(endpoint));
                    }
                } else if let Some(network) = self.networks.get(self.settings.row_index) {
                    self.form = Some(ActiveForm::open_edit_network(network));
                }
                KeyAction::None
            }
            SettingsSection::Assets => {
                let asset = self
                    .settings_asset_identities()
                    .get(self.settings.row_index)
                    .and_then(|identity| self.assets.get(identity));
                if let Some(asset) = asset {
                    self.form = Some(ActiveForm::open_edit_asset(asset));
                }
                KeyAction::None
            }
            SettingsSection::General => KeyAction::None,
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
    CreateGroup(String),
    RenameGroup(u64, String),
    DeleteGroup(u64),
    CommitLayout(koi::models::account::layout::AccountLayoutUpdate),
    SetDisplayCurrency(String),
    SetTheme(Theme),
    RefreshQuotes,
    FetchNetworkPresets,
    FetchDerivationPath,
    GenerateMnemonic,
    ResolveEns(String),
    ReverseEns(String),
    DeriveAddresses {
        name: String,
        mnemonic: String,
        paths: Vec<String>,
    },
    CreateAccount {
        name: String,
        wallet: NewAccountWallet,
    },
    CreateAccountFromKey {
        name: String,
        key: String,
    },
    RenameAccount(u64, String),
    DeleteAccount(u64),
    UpdateAccountNetworks(u64, Vec<u64>),
    OpenAssetPicker {
        account_id: u64,
        unlink: bool,
    },
    AccountAssetAction {
        account_id: u64,
        identity: String,
        unlink: bool,
    },
    UpdateEndpoint {
        network_id: u64,
        endpoint_id: i32,
        update: koi::models::network::endpoint::NetworkEndpointUpdate,
    },
    UpdateNetwork {
        network_id: u64,
        update: koi::models::network::NetworkUpdate,
    },
    DeleteNetwork(u64),
    UpdateAsset {
        identity: String,
        update: koi::models::asset::AssetUpdate,
    },
    ToggleQuoter(String, bool),
    DiscoverQuoter {
        token_a: String,
        token_b: Option<String>,
    },
    CreateQuoter {
        token_a: String,
        token_b: String,
        config: koi::models::quoter::QuoterConfig,
    },
}
