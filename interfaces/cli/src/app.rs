use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};

use koi::models::{
    account::{Account, balances::AccountBalances, group::AccountGroup, metadata::WalletType},
    asset::Asset,
    network::{Network, pool::RpcPoolStats},
    tx::Tx,
};

use super::defi::DefiResult;
use super::form::{ActiveForm, FormAction};
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

pub fn normalized_group_id(account: &Account) -> Option<u64> {
    account.group_id.map(|group| group.0).filter(|id| *id > 0)
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

pub struct App {
    pub tab: Tab,
    pub account_panel: AccountPanel,
    pub account_focus: AccountFocus,
    pub accounts: Vec<Account>,
    pub groups: Vec<AccountGroup>,
    pub collapsed_groups: HashSet<Option<u64>>,
    pub move_mode: Option<MoveMode>,
    pub display_currency: String,
    pub networks: Vec<Network>,
    pub assets: HashMap<String, Asset>,
    pub rpc_states: HashMap<u64, ResourceState<RpcPoolStats>>,
    pub asset_quotes: HashMap<String, ResourceState<String>>,
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
            groups: Vec::new(),
            collapsed_groups: HashSet::new(),
            move_mode: None,
            display_currency: "fiat:usd".to_string(),
            networks: Vec::new(),
            assets: HashMap::new(),
            rpc_states: HashMap::new(),
            asset_quotes: HashMap::new(),
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

    pub fn layout_view(&self) -> (&[AccountGroup], &[Account]) {
        match &self.move_mode {
            Some(mode) => (&mode.groups, &mode.accounts),
            None => (&self.groups, &self.accounts),
        }
    }

    pub fn account_rows(&self) -> Vec<AccountListRow> {
        let editing = self.move_mode.is_some();
        let (groups, accounts) = self.layout_view();
        let mut rows = Vec::new();

        for group in sorted_group_refs(groups) {
            let group_id = Some(group.group_identity.0);
            let members = bucket_refs(accounts, group_id);
            let collapsed = !editing && self.collapsed_groups.contains(&group_id);
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

        let ungrouped = bucket_refs(accounts, None);
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
        if let Some(AccountListRow::GroupHeader {
            group_id: Some(group_id),
            name,
            ..
        }) = self.selected_list_row()
        {
            self.form = Some(ActiveForm::open_group_name(Some(group_id), &name));
        }
        KeyAction::None
    }

    fn open_group_delete_selected(&mut self) -> KeyAction {
        if let Some(AccountListRow::GroupHeader {
            group_id: Some(group_id),
            name,
            ..
        }) = self.selected_list_row()
        {
            self.form = Some(ActiveForm::open_confirm_delete_group(group_id, name));
        }
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
            BackgroundUpdate::AssetQuote {
                generation,
                identity,
                state,
            } => {
                if generation != self.refresh_generation {
                    return;
                }
                self.asset_quotes.insert(identity, state);
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

        if self.move_mode.is_some() {
            return self.handle_move_mode_key(code);
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
            KeyCode::Char('$') if self.selected_account.is_none() => self.open_currency_picker(),
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
        }
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
                .insert(identity.clone(), ResourceState::Loading);
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
    CreateGroup(String),
    RenameGroup(u64, String),
    DeleteGroup(u64),
    CommitLayout(koi::models::account::layout::AccountLayoutUpdate),
    SetDisplayCurrency(String),
    RefreshQuotes,
    FetchNetworkPresets,
}
