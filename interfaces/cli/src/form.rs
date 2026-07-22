use std::str::FromStr;

use crossterm::event::KeyCode;
use koi::models::{
    account::rpc::DeriveMnemonicResult,
    asset::{Asset, AssetUpdate, identity::AssetIdentity, metadata::AssetMetadataDiscovery},
    network::{
        Network, NetworkUpdate,
        endpoint::{NetworkEndpoint, NetworkEndpointCreate, NetworkEndpointUpdate},
        identity::NetworkIdentity,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountKind {
    Generate,
    View,
    Safe,
    Mnemonic,
    PrivateKey,
}

impl AccountKind {
    pub(crate) const ALL: [AccountKind; 5] = [
        AccountKind::Generate,
        AccountKind::View,
        AccountKind::Safe,
        AccountKind::Mnemonic,
        AccountKind::PrivateKey,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            AccountKind::Generate => "New wallet (generate mnemonic)",
            AccountKind::View => "Watch address (view only)",
            AccountKind::Safe => "Safe wallet (watch)",
            AccountKind::Mnemonic => "Import mnemonic",
            AccountKind::PrivateKey => "Import private key",
        }
    }
}

#[derive(Clone, Debug)]
pub enum NewAccountWallet {
    View(String),
    Safe(String),
    Eoa(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetType {
    Erc20,
    Native,
    Fiat,
}

impl AssetType {
    pub(crate) const ALL: [AssetType; 3] = [AssetType::Erc20, AssetType::Native, AssetType::Fiat];

    pub(crate) fn label(self) -> &'static str {
        match self {
            AssetType::Erc20 => "ERC-20 token",
            AssetType::Native => "Native token",
            AssetType::Fiat => "Fiat currency",
        }
    }

    fn supports_discovery(self) -> bool {
        matches!(self, AssetType::Erc20 | AssetType::Native)
    }
}

#[derive(Clone, Debug, Default)]
pub struct AssetFieldHints {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub icon_url: Option<String>,
}

impl AssetFieldHints {
    pub fn from_discovery(discovery: &AssetMetadataDiscovery) -> Self {
        let mut hints = Self::default();

        if let Some(erc20) = discovery.options.get("erc20") {
            hints.name = erc20.name.clone();
            hints.symbol = erc20.symbol.clone();
            hints.decimals = erc20.decimals;
        }

        for option in discovery.options.values() {
            if hints.name.is_none() {
                hints.name = option.name.clone();
            }
            if hints.symbol.is_none() {
                hints.symbol = option.symbol.clone();
            }
            if hints.decimals.is_none() {
                hints.decimals = option.decimals;
            }
            if hints.icon_url.is_none() {
                hints.icon_url = option.icon_url.clone();
            }
        }

        hints
    }

    pub fn hint_display(&self, asset_type: AssetType, index: usize) -> Option<String> {
        match asset_type {
            AssetType::Erc20 => match index {
                2 => self.name.clone(),
                3 => self.symbol.clone(),
                4 => self.decimals.map(|d| d.to_string()),
                5 => self.icon_url.clone(),
                _ => None,
            },
            AssetType::Native => match index {
                1 => self.name.clone(),
                2 => self.symbol.clone(),
                3 => self.decimals.map(|d| d.to_string()),
                4 => self.icon_url.clone(),
                _ => None,
            },
            AssetType::Fiat => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscoveryState {
    Idle,
    Loading,
    Ready,
    Failed,
}

impl DiscoveryState {
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }
}

#[derive(Clone, Debug)]
pub struct TextForm {
    pub title: String,
    pub fields: Vec<FormField>,
    pub focus: usize,
}

#[derive(Clone, Debug)]
pub struct FormField {
    pub label: String,
    pub value: String,
    pub required: bool,
}

impl TextForm {
    pub fn new(title: impl Into<String>, fields: Vec<(&str, bool)>) -> Self {
        Self {
            title: title.into(),
            fields: fields
                .into_iter()
                .map(|(label, required)| FormField {
                    label: label.to_string(),
                    value: String::new(),
                    required,
                })
                .collect(),
            focus: 0,
        }
    }

    pub fn field(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(|field| field.value.as_str())
    }

    pub fn move_focus(&mut self, delta: i32) {
        if self.fields.is_empty() {
            return;
        }
        let next = self.focus as i32 + delta;
        self.focus = next.clamp(0, self.fields.len() as i32 - 1) as usize;
    }

    fn edit_current(&mut self, edit: impl FnOnce(&mut String)) {
        if let Some(field) = self.fields.get_mut(self.focus) {
            edit(&mut field.value);
        }
    }

    pub fn can_submit(&self) -> bool {
        self.fields
            .iter()
            .all(|field| !field.required || !field.value.trim().is_empty())
    }
}

#[derive(Debug)]
pub enum ActiveForm {
    AddAssetType {
        selected: usize,
    },
    AddAsset {
        asset_type: AssetType,
        form: TextForm,
        touched: Vec<bool>,
        hints: AssetFieldHints,
        discovery: DiscoveryState,
        pending_identity: Option<String>,
        discovered_identity: Option<String>,
    },
    AddNetworkMode {
        selected: usize,
    },
    AddNetwork(TextForm),
    AddNetworkPreset {
        presets: Vec<Network>,
        selected: usize,
    },
    AddEndpoint {
        network_id: u64,
        form: TextForm,
    },
    GroupName {
        group_id: Option<u64>,
        form: TextForm,
    },
    PickCurrency {
        options: Vec<Asset>,
        selected: usize,
    },
    ConfirmDeleteGroup {
        group_id: u64,
        name: String,
    },
    AddAccountType {
        selected: usize,
    },
    AddAccountAddress {
        kind: AccountKind,
        form: TextForm,
        ens_hint: Option<String>,
        ens_pending: Option<String>,
        ens_primary: Option<String>,
        ens_primary_pending: Option<String>,
    },
    AddAccountMnemonic {
        form: TextForm,
        deriving: bool,
    },
    AddAccountPickAddress {
        name: String,
        options: Vec<DeriveMnemonicResult>,
        selected: usize,
    },
    RenameAccount {
        account_id: u64,
        form: TextForm,
    },
    ConfirmDeleteAccount {
        account_id: u64,
        name: String,
    },
    AccountNetworks {
        account_id: u64,
        options: Vec<(u64, String, bool)>,
        selected: usize,
    },
    PickAccountAsset {
        account_id: u64,
        unlink: bool,
        options: Vec<(String, String)>,
        selected: usize,
    },
    EditEndpoint {
        network_id: u64,
        endpoint_id: i32,
        form: TextForm,
    },
    EditNetwork {
        network_id: u64,
        form: TextForm,
    },
    EditAsset {
        identity: String,
        form: TextForm,
    },
    ConfirmDeleteNetwork {
        network_id: u64,
        name: String,
    },
    PickQuoterToken {
        token_a: Option<String>,
        options: Vec<(String, String)>,
        selected: usize,
    },
    QuoterDiscovering,
    PickQuoterSource {
        token_a: String,
        options: Vec<QuoterSourceOption>,
        selected: usize,
    },
}

#[derive(Debug)]
pub struct QuoterSourceOption {
    pub label: String,
    pub token_b: String,
    pub config: koi::models::quoter::QuoterConfig,
}

pub const ERC4626_SENTINEL: &str = "<erc4626>";

#[derive(Debug)]
pub enum FormAction {
    None,
    Cancel,
    OpenNetworkPresets,
    FetchAssetMetadata(String),
    SubmitCreateAsset(Asset),
    SubmitCreateNetwork(Network),
    SubmitCreateEndpoint {
        network_id: u64,
        input: NetworkEndpointCreate,
    },
    SubmitGroupName {
        group_id: Option<u64>,
        name: String,
    },
    ConfirmDeleteGroup(u64),
    SubmitDisplayCurrency(String),
    FetchDerivationPath,
    GenerateMnemonic,
    ResolveEns(String),
    ReverseEns(String),
    SubmitCreateAccount {
        name: String,
        wallet: NewAccountWallet,
    },
    DeriveMnemonic {
        name: String,
        mnemonic: String,
        base_path: String,
    },
    CreateAccountFromKey {
        name: String,
        key: String,
    },
    SubmitRenameAccount {
        account_id: u64,
        name: String,
    },
    ConfirmDeleteAccount(u64),
    SubmitAccountNetworks {
        account_id: u64,
        networks: Vec<u64>,
    },
    SubmitAccountAsset {
        account_id: u64,
        unlink: bool,
        identity: String,
    },
    PickQuoterTokenA(String),
    DiscoverQuoter {
        token_a: String,
        token_b: Option<String>,
    },
    SubmitCreateQuoter {
        token_a: String,
        token_b: String,
        config: koi::models::quoter::QuoterConfig,
    },
    SubmitEditEndpoint {
        network_id: u64,
        endpoint_id: i32,
        update: NetworkEndpointUpdate,
    },
    SubmitEditNetwork {
        network_id: u64,
        update: NetworkUpdate,
    },
    SubmitEditAsset {
        identity: String,
        update: AssetUpdate,
    },
    ConfirmDeleteNetwork(u64),
}

impl ActiveForm {
    pub fn open_add_asset() -> Self {
        Self::AddAssetType { selected: 0 }
    }

    pub fn open_add_network() -> Self {
        Self::AddNetworkMode { selected: 0 }
    }

    pub fn open_add_account() -> Self {
        Self::AddAccountType { selected: 0 }
    }

    pub fn open_rename_account(account_id: u64, current_name: &str) -> Self {
        let mut form = TextForm::new("Rename account", vec![("Name", true)]);
        if let Some(field) = form.fields.get_mut(0) {
            field.value = current_name.to_string();
        }
        Self::RenameAccount { account_id, form }
    }

    pub fn open_confirm_delete_account(account_id: u64, name: String) -> Self {
        Self::ConfirmDeleteAccount { account_id, name }
    }

    pub fn open_account_networks(account_id: u64, options: Vec<(u64, String, bool)>) -> Self {
        Self::AccountNetworks {
            account_id,
            options,
            selected: 0,
        }
    }

    pub fn open_edit_endpoint(endpoint: &NetworkEndpoint) -> Self {
        let mut form = TextForm::new(
            format!("Edit endpoint #{}", endpoint.endpoint_identity),
            vec![
                ("Label", false),
                ("URL", true),
                ("Type (http/ws)", true),
                ("Enabled (yes/no)", true),
            ],
        );
        let values = [
            endpoint.endpoint_label.clone().unwrap_or_default(),
            endpoint.endpoint_url.clone(),
            endpoint.endpoint_type.clone(),
            if endpoint.endpoint_disabled {
                "no"
            } else {
                "yes"
            }
            .to_string(),
        ];
        for (field, value) in form.fields.iter_mut().zip(values) {
            field.value = value;
        }
        Self::EditEndpoint {
            network_id: endpoint.network_identity.0,
            endpoint_id: endpoint.endpoint_identity,
            form,
        }
    }

    pub fn open_edit_network(network: &Network) -> Self {
        let mut form = TextForm::new(
            format!("Edit network {}", network.network_identity.0),
            vec![("Name", true), ("Icon URL", false)],
        );
        let values = [
            network.network_name.clone(),
            network.network_icon_url.clone().unwrap_or_default(),
        ];
        for (field, value) in form.fields.iter_mut().zip(values) {
            field.value = value;
        }
        Self::EditNetwork {
            network_id: network.network_identity.0,
            form,
        }
    }

    pub fn open_edit_asset(asset: &Asset) -> Self {
        let mut form = TextForm::new(
            format!("Edit asset {}", asset.asset_identity),
            vec![
                ("Name", true),
                ("Symbol", true),
                ("Decimals", true),
                ("Icon URL", false),
            ],
        );
        let values = [
            asset.asset_name.clone(),
            asset.asset_symbol.clone(),
            asset.asset_decimals.to_string(),
            asset.asset_icon_url.clone().unwrap_or_default(),
        ];
        for (field, value) in form.fields.iter_mut().zip(values) {
            field.value = value;
        }
        Self::EditAsset {
            identity: asset.asset_identity.to_string(),
            form,
        }
    }

    pub fn open_confirm_delete_network(network_id: u64, name: String) -> Self {
        Self::ConfirmDeleteNetwork { network_id, name }
    }

    pub fn open_group_name(group_id: Option<u64>, current_name: &str) -> Self {
        let title = if group_id.is_some() {
            "Rename group"
        } else {
            "New group"
        };
        let mut form = TextForm::new(title, vec![("Name", true)]);
        if let Some(field) = form.fields.get_mut(0) {
            field.value = current_name.to_string();
        }
        Self::GroupName { group_id, form }
    }

    pub fn open_confirm_delete_group(group_id: u64, name: String) -> Self {
        Self::ConfirmDeleteGroup { group_id, name }
    }

    pub fn open_add_endpoint(network_id: u64) -> Self {
        let mut form = TextForm::new(
            format!("Add RPC endpoint · network {network_id}"),
            vec![
                ("Label", false),
                ("URL", true),
                ("Type (http/ws)", true),
                ("Enabled (yes/no)", true),
            ],
        );
        if let Some(field) = form.fields.get_mut(2) {
            field.value = "http".to_string();
        }
        if let Some(field) = form.fields.get_mut(3) {
            field.value = "yes".to_string();
        }
        Self::AddEndpoint { network_id, form }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::AddAssetType { .. } => "Add asset · choose type",
            Self::AddAsset { form, .. } => &form.title,
            Self::AddNetworkMode { .. } => "Add network · choose mode",
            Self::AddNetwork(form) => &form.title,
            Self::AddNetworkPreset { .. } => "Add network · choose preset",
            Self::AddEndpoint { form, .. } => &form.title,
            Self::GroupName { form, .. } => &form.title,
            Self::PickCurrency { .. } => "Display currency",
            Self::ConfirmDeleteGroup { .. } => "Delete group",
            Self::AddAccountType { .. } => "Add account · choose type",
            Self::AddAccountAddress { form, .. } => &form.title,
            Self::AddAccountMnemonic { .. } => "Add account · import mnemonic",
            Self::AddAccountPickAddress { .. } => "Add account · pick address",
            Self::RenameAccount { form, .. } => &form.title,
            Self::ConfirmDeleteAccount { .. } => "Delete account",
            Self::AccountNetworks { .. } => "Account networks",
            Self::PickAccountAsset { unlink: false, .. } => "Link asset",
            Self::PickAccountAsset { unlink: true, .. } => "Unlink asset",
            Self::EditEndpoint { form, .. } => &form.title,
            Self::EditNetwork { form, .. } => &form.title,
            Self::EditAsset { form, .. } => &form.title,
            Self::ConfirmDeleteNetwork { .. } => "Delete network",
            Self::PickQuoterToken { token_a: None, .. } => "Add quoter · token to price",
            Self::PickQuoterToken {
                token_a: Some(_), ..
            } => "Add quoter · quote against",
            Self::QuoterDiscovering => "Add quoter · discovering routes",
            Self::PickQuoterSource { .. } => "Add quoter · choose source",
        }
    }

    pub fn modal_dimensions(&self) -> (u16, u16) {
        match self {
            Self::AddAssetType { .. } => (46, 9),
            Self::AddAsset {
                form, discovery, ..
            } => {
                let extra = usize::from(discovery.is_loading());
                (74, (form.fields.len() + extra + 5).min(24) as u16)
            }
            Self::AddNetworkMode { .. } => (48, 9),
            Self::AddNetworkPreset { presets, .. } => (74, (presets.len() + 5).min(20) as u16),
            Self::AddNetwork(form) => (58, (form.fields.len() + 5) as u16),
            Self::AddEndpoint { form, .. } => (62, (form.fields.len() + 5) as u16),
            Self::GroupName { form, .. } => (48, (form.fields.len() + 5) as u16),
            Self::PickCurrency { options, .. } => (48, (options.len() + 5).min(20) as u16),
            Self::ConfirmDeleteGroup { .. } => (56, 7),
            Self::AddAccountType { .. } => (48, 10),
            Self::AddAccountAddress {
                form,
                ens_hint,
                ens_primary,
                ..
            } => (
                74,
                (form.fields.len()
                    + 5
                    + usize::from(ens_hint.is_some())
                    + usize::from(ens_primary.is_some())) as u16,
            ),
            Self::AddAccountMnemonic { form, deriving } => {
                let extra = usize::from(*deriving);
                (78, (form.fields.len() + extra + 5) as u16)
            }
            Self::AddAccountPickAddress { options, .. } => (74, (options.len() + 5).min(20) as u16),
            Self::RenameAccount { form, .. } => (48, (form.fields.len() + 5) as u16),
            Self::ConfirmDeleteAccount { .. } => (56, 7),
            Self::AccountNetworks { options, .. } => (56, (options.len() + 6).min(20) as u16),
            Self::PickAccountAsset { options, .. } => (70, (options.len() + 5).min(20) as u16),
            Self::EditEndpoint { form, .. } => (62, (form.fields.len() + 5) as u16),
            Self::EditNetwork { form, .. } => (58, (form.fields.len() + 5) as u16),
            Self::EditAsset { form, .. } => (62, (form.fields.len() + 5) as u16),
            Self::ConfirmDeleteNetwork { .. } => (56, 7),
            Self::PickQuoterToken { options, .. } => (70, (options.len() + 5).min(20) as u16),
            Self::QuoterDiscovering => (48, 7),
            Self::PickQuoterSource { options, .. } => (74, (options.len() + 5).min(20) as u16),
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) -> FormAction {
        match self {
            Self::AddAssetType { selected } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(AssetType::ALL.len() - 1);
                    FormAction::None
                }
                KeyCode::Enter => {
                    let asset_type = AssetType::ALL[*selected];
                    let form = asset_form(asset_type);
                    let touched = vec![false; form.fields.len()];
                    *self = Self::AddAsset {
                        asset_type,
                        form,
                        touched,
                        hints: AssetFieldHints::default(),
                        discovery: DiscoveryState::Idle,
                        pending_identity: None,
                        discovered_identity: None,
                    };
                    FormAction::None
                }
                _ => FormAction::None,
            },
            Self::AddNetworkMode { selected } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(1);
                    FormAction::None
                }
                KeyCode::Enter => match *selected {
                    0 => {
                        *self = Self::AddNetwork(network_form());
                        FormAction::None
                    }
                    1 => FormAction::OpenNetworkPresets,
                    _ => FormAction::None,
                },
                _ => FormAction::None,
            },
            Self::AddNetworkPreset { presets, selected } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(presets.len().saturating_sub(1));
                    FormAction::None
                }
                KeyCode::Enter => presets
                    .get(*selected)
                    .map(|network| {
                        FormAction::SubmitCreateNetwork(Network {
                            network_identity: network.network_identity.clone(),
                            network_name: network.network_name.clone(),
                            network_icon_url: network.network_icon_url.clone(),
                        })
                    })
                    .unwrap_or(FormAction::None),
                _ => FormAction::None,
            },
            Self::AddAsset {
                asset_type,
                form,
                touched,
                hints,
                discovery,
                pending_identity,
                discovered_identity,
            } => handle_add_asset_key(
                code,
                *asset_type,
                form,
                touched,
                hints,
                discovery,
                pending_identity,
                discovered_identity,
            ),
            Self::AddNetwork(form) => handle_text_form(code, form, |form| {
                build_network(form).map(FormAction::SubmitCreateNetwork)
            }),
            Self::AddEndpoint { network_id, form } => handle_text_form(code, form, |form| {
                build_endpoint(form).map(|input| FormAction::SubmitCreateEndpoint {
                    network_id: *network_id,
                    input,
                })
            }),
            Self::GroupName { group_id, form } => {
                let group_id = *group_id;
                handle_text_form(code, form, |form| {
                    let name = form.field(0)?.trim().to_string();
                    if name.is_empty() {
                        return None;
                    }
                    Some(FormAction::SubmitGroupName { group_id, name })
                })
            }
            Self::ConfirmDeleteGroup { group_id, .. } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Enter => FormAction::ConfirmDeleteGroup(*group_id),
                _ => FormAction::None,
            },
            Self::PickCurrency { options, selected } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(options.len().saturating_sub(1));
                    FormAction::None
                }
                KeyCode::Enter => options
                    .get(*selected)
                    .map(|asset| {
                        FormAction::SubmitDisplayCurrency(asset.asset_identity.to_string())
                    })
                    .unwrap_or(FormAction::None),
                _ => FormAction::None,
            },
            Self::AddAccountType { selected } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(AccountKind::ALL.len() - 1);
                    FormAction::None
                }
                KeyCode::Enter => {
                    let kind = AccountKind::ALL[*selected];
                    *self = match kind {
                        AccountKind::View | AccountKind::Safe => Self::AddAccountAddress {
                            kind,
                            form: TextForm::new(
                                format!("Add account · {}", kind.label()),
                                vec![("Name", true), ("Address", true)],
                            ),
                            ens_hint: None,
                            ens_pending: None,
                            ens_primary: None,
                            ens_primary_pending: None,
                        },
                        AccountKind::Mnemonic | AccountKind::Generate => Self::AddAccountMnemonic {
                            form: TextForm::new(
                                if kind == AccountKind::Generate {
                                    "Add account · new wallet (write the mnemonic down!)"
                                } else {
                                    "Add account · import mnemonic"
                                },
                                vec![("Name", true), ("Mnemonic", true), ("Base path", true)],
                            ),
                            deriving: false,
                        },
                        AccountKind::PrivateKey => Self::AddAccountAddress {
                            kind,
                            form: TextForm::new(
                                "Add account · import private key",
                                vec![("Name", true), ("Private key", true)],
                            ),
                            ens_hint: None,
                            ens_pending: None,
                            ens_primary: None,
                            ens_primary_pending: None,
                        },
                    };
                    match kind {
                        AccountKind::Generate => FormAction::GenerateMnemonic,
                        AccountKind::Mnemonic => FormAction::FetchDerivationPath,
                        _ => FormAction::None,
                    }
                }
                _ => FormAction::None,
            },
            Self::AddAccountAddress {
                kind,
                form,
                ens_hint,
                ens_pending,
                ens_primary,
                ens_primary_pending,
            } => {
                let kind = *kind;
                let supports_ens = matches!(kind, AccountKind::View | AccountKind::Safe);
                match code {
                    KeyCode::Esc => FormAction::Cancel,
                    KeyCode::Up | KeyCode::BackTab => {
                        form.move_focus(-1);
                        FormAction::None
                    }
                    KeyCode::Down => {
                        form.move_focus(1);
                        FormAction::None
                    }
                    KeyCode::Tab => {
                        if supports_ens {
                            if let Some(hint) = ens_hint.take() {
                                if let Some(field) = form.fields.get_mut(1) {
                                    field.value = hint.clone();
                                }
                                form.focus = 1;
                                *ens_primary = None;
                                *ens_primary_pending = Some(hint.to_lowercase());
                                return FormAction::ReverseEns(hint);
                            }
                            if form.focus == 0 {
                                let name_empty =
                                    form.field(0).is_none_or(|value| value.trim().is_empty());
                                if name_empty {
                                    if let Some(primary) = ens_primary.clone() {
                                        if let Some(field) = form.fields.get_mut(0) {
                                            field.value = primary;
                                        }
                                        return FormAction::None;
                                    }
                                }
                            }
                        }
                        form.move_focus(1);
                        FormAction::None
                    }
                    KeyCode::Backspace => {
                        form.edit_current(|value| {
                            value.pop();
                        });
                        ens_actions(
                            supports_ens,
                            form,
                            ens_hint,
                            ens_pending,
                            ens_primary,
                            ens_primary_pending,
                        )
                    }
                    KeyCode::Char(ch) if !ch.is_control() => {
                        form.edit_current(|value| value.push(ch));
                        ens_actions(
                            supports_ens,
                            form,
                            ens_hint,
                            ens_pending,
                            ens_primary,
                            ens_primary_pending,
                        )
                    }
                    KeyCode::Enter if form.can_submit() => {
                        let name = form.field(0).unwrap_or_default().trim().to_string();
                        let value = form.field(1).unwrap_or_default().trim().to_string();
                        if name.is_empty() || value.is_empty() {
                            return FormAction::None;
                        }
                        match kind {
                            AccountKind::View => FormAction::SubmitCreateAccount {
                                name,
                                wallet: NewAccountWallet::View(value),
                            },
                            AccountKind::Safe => FormAction::SubmitCreateAccount {
                                name,
                                wallet: NewAccountWallet::Safe(value),
                            },
                            AccountKind::PrivateKey => {
                                FormAction::CreateAccountFromKey { name, key: value }
                            }
                            AccountKind::Mnemonic | AccountKind::Generate => FormAction::None,
                        }
                    }
                    _ => FormAction::None,
                }
            }
            Self::AddAccountMnemonic { form, deriving } => {
                if *deriving {
                    return match code {
                        KeyCode::Esc => FormAction::Cancel,
                        _ => FormAction::None,
                    };
                }
                handle_text_form(code, form, |form| {
                    let name = form.field(0)?.trim().to_string();
                    let mnemonic = form.field(1)?.trim().to_string();
                    let base_path = form.field(2)?.trim().to_string();
                    if name.is_empty() || mnemonic.is_empty() || base_path.is_empty() {
                        return None;
                    }
                    Some(FormAction::DeriveMnemonic {
                        name,
                        mnemonic,
                        base_path,
                    })
                })
            }
            Self::AddAccountPickAddress {
                name,
                options,
                selected,
            } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(options.len().saturating_sub(1));
                    FormAction::None
                }
                KeyCode::Enter => options
                    .get(*selected)
                    .map(|result| FormAction::SubmitCreateAccount {
                        name: name.clone(),
                        wallet: NewAccountWallet::Eoa(result.address.clone()),
                    })
                    .unwrap_or(FormAction::None),
                _ => FormAction::None,
            },
            Self::RenameAccount { account_id, form } => {
                let account_id = *account_id;
                handle_text_form(code, form, |form| {
                    let name = form.field(0)?.trim().to_string();
                    if name.is_empty() {
                        return None;
                    }
                    Some(FormAction::SubmitRenameAccount { account_id, name })
                })
            }
            Self::ConfirmDeleteAccount { account_id, .. } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Enter => FormAction::ConfirmDeleteAccount(*account_id),
                _ => FormAction::None,
            },
            Self::AccountNetworks {
                account_id,
                options,
                selected,
            } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(options.len().saturating_sub(1));
                    FormAction::None
                }
                KeyCode::Char(' ') => {
                    if let Some(option) = options.get_mut(*selected) {
                        option.2 = !option.2;
                    }
                    FormAction::None
                }
                KeyCode::Enter => FormAction::SubmitAccountNetworks {
                    account_id: *account_id,
                    networks: options
                        .iter()
                        .filter(|option| option.2)
                        .map(|option| option.0)
                        .collect(),
                },
                _ => FormAction::None,
            },
            Self::EditEndpoint {
                network_id,
                endpoint_id,
                form,
            } => {
                let network_id = *network_id;
                let endpoint_id = *endpoint_id;
                handle_text_form(code, form, |form| {
                    let url = form.field(1)?.trim().to_string();
                    if url.is_empty() {
                        return None;
                    }
                    let enabled = form.field(3)?.trim();
                    Some(FormAction::SubmitEditEndpoint {
                        network_id,
                        endpoint_id,
                        update: NetworkEndpointUpdate {
                            endpoint_label: optional_field(form, 0),
                            endpoint_url: Some(url),
                            endpoint_type: optional_field(form, 2),
                            endpoint_disabled: Some(
                                !enabled.eq_ignore_ascii_case("yes")
                                    && !enabled.eq_ignore_ascii_case("y")
                                    && !enabled.eq_ignore_ascii_case("true")
                                    && enabled != "1",
                            ),
                        },
                    })
                })
            }
            Self::EditNetwork { network_id, form } => {
                let network_id = *network_id;
                handle_text_form(code, form, |form| {
                    let name = form.field(0)?.trim().to_string();
                    if name.is_empty() {
                        return None;
                    }
                    Some(FormAction::SubmitEditNetwork {
                        network_id,
                        update: NetworkUpdate {
                            network_name: Some(name),
                            network_icon_url: optional_field(form, 1),
                        },
                    })
                })
            }
            Self::EditAsset { identity, form } => {
                let identity = identity.clone();
                handle_text_form(code, form, |form| {
                    let name = form.field(0)?.trim().to_string();
                    let symbol = form.field(1)?.trim().to_string();
                    let decimals = form.field(2)?.trim().parse::<u8>().ok()?;
                    if name.is_empty() || symbol.is_empty() {
                        return None;
                    }
                    Some(FormAction::SubmitEditAsset {
                        identity,
                        update: AssetUpdate {
                            asset_name: Some(name),
                            asset_symbol: Some(symbol),
                            asset_decimals: Some(decimals),
                            asset_icon_url: optional_field(form, 3),
                        },
                    })
                })
            }
            Self::ConfirmDeleteNetwork { network_id, .. } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Enter => FormAction::ConfirmDeleteNetwork(*network_id),
                _ => FormAction::None,
            },
            Self::PickQuoterToken {
                token_a,
                options,
                selected,
            } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(options.len().saturating_sub(1));
                    FormAction::None
                }
                KeyCode::Enter => {
                    let Some((identity, _)) = options.get(*selected) else {
                        return FormAction::None;
                    };
                    match token_a {
                        None => FormAction::PickQuoterTokenA(identity.clone()),
                        Some(token_a) => FormAction::DiscoverQuoter {
                            token_a: token_a.clone(),
                            token_b: (identity != ERC4626_SENTINEL).then(|| identity.clone()),
                        },
                    }
                }
                _ => FormAction::None,
            },
            Self::QuoterDiscovering => match code {
                KeyCode::Esc => FormAction::Cancel,
                _ => FormAction::None,
            },
            Self::PickQuoterSource {
                token_a,
                options,
                selected,
            } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(options.len().saturating_sub(1));
                    FormAction::None
                }
                KeyCode::Enter => {
                    let Some(option) = options.get_mut(*selected) else {
                        return FormAction::None;
                    };
                    let config = std::mem::replace(
                        &mut option.config,
                        koi::models::quoter::QuoterConfig::Erc4626(
                            koi::models::quoter::Erc4626QuoterConfig {},
                        ),
                    );
                    FormAction::SubmitCreateQuoter {
                        token_a: token_a.clone(),
                        token_b: option.token_b.clone(),
                        config,
                    }
                }
                _ => FormAction::None,
            },
            Self::PickAccountAsset {
                account_id,
                unlink,
                options,
                selected,
            } => match code {
                KeyCode::Esc => FormAction::Cancel,
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.saturating_sub(1);
                    FormAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1).min(options.len().saturating_sub(1));
                    FormAction::None
                }
                KeyCode::Enter => options
                    .get(*selected)
                    .map(|(identity, _)| FormAction::SubmitAccountAsset {
                        account_id: *account_id,
                        unlink: *unlink,
                        identity: identity.clone(),
                    })
                    .unwrap_or(FormAction::None),
                _ => FormAction::None,
            },
        }
    }

    pub fn apply_asset_metadata(
        &mut self,
        identity: &str,
        discovery: Result<AssetMetadataDiscovery, String>,
    ) {
        let Self::AddAsset {
            hints,
            discovery: state,
            pending_identity,
            discovered_identity,
            ..
        } = self
        else {
            return;
        };

        if pending_identity.as_deref() != Some(identity) {
            return;
        }

        *discovered_identity = Some(identity.to_string());
        match discovery {
            Ok(data) => {
                *hints = AssetFieldHints::from_discovery(&data);
                *state = DiscoveryState::Ready;
            }
            Err(_) => {
                *state = DiscoveryState::Failed;
            }
        }
        *pending_identity = None;
    }
}

fn handle_add_asset_key(
    code: KeyCode,
    asset_type: AssetType,
    form: &mut TextForm,
    touched: &mut [bool],
    hints: &mut AssetFieldHints,
    discovery: &mut DiscoveryState,
    pending_identity: &mut Option<String>,
    discovered_identity: &mut Option<String>,
) -> FormAction {
    match code {
        KeyCode::Esc => FormAction::Cancel,
        KeyCode::Up => {
            form.move_focus(-1);
            FormAction::None
        }
        KeyCode::Down => {
            form.move_focus(1);
            FormAction::None
        }
        KeyCode::BackTab => {
            form.move_focus(-1);
            FormAction::None
        }
        KeyCode::Tab => {
            if try_accept_hint(form, touched, hints, asset_type, form.focus) {
                return FormAction::None;
            }
            form.move_focus(1);
            FormAction::None
        }
        KeyCode::Backspace => {
            if let Some(flag) = touched.get_mut(form.focus) {
                *flag = true;
            }
            form.edit_current(|value| {
                value.pop();
            });
            maybe_discovery_action(
                asset_type,
                form,
                form.focus,
                discovery,
                pending_identity,
                discovered_identity,
                hints,
            )
        }
        KeyCode::Char(ch) if !ch.is_control() => {
            if let Some(flag) = touched.get_mut(form.focus) {
                *flag = true;
            }
            form.edit_current(|value| value.push(ch));
            maybe_discovery_action(
                asset_type,
                form,
                form.focus,
                discovery,
                pending_identity,
                discovered_identity,
                hints,
            )
        }
        KeyCode::Enter if form.can_submit() => build_asset(asset_type, form)
            .map(FormAction::SubmitCreateAsset)
            .unwrap_or(FormAction::None),
        _ => FormAction::None,
    }
}

fn maybe_discovery_action(
    asset_type: AssetType,
    form: &TextForm,
    focus: usize,
    discovery: &mut DiscoveryState,
    pending_identity: &mut Option<String>,
    discovered_identity: &mut Option<String>,
    hints: &mut AssetFieldHints,
) -> FormAction {
    if !field_triggers_discovery(asset_type, focus) {
        return FormAction::None;
    }
    discovery_action(
        asset_type,
        form,
        discovery,
        pending_identity,
        discovered_identity,
        hints,
    )
}

fn field_triggers_discovery(asset_type: AssetType, focus: usize) -> bool {
    match asset_type {
        AssetType::Erc20 => focus <= 1,
        AssetType::Native => focus == 0,
        AssetType::Fiat => false,
    }
}

fn try_accept_hint(
    form: &mut TextForm,
    touched: &[bool],
    hints: &AssetFieldHints,
    asset_type: AssetType,
    index: usize,
) -> bool {
    let Some(field) = form.fields.get_mut(index) else {
        return false;
    };
    if !field.value.is_empty() {
        return false;
    }
    if touched.get(index).copied().unwrap_or(false) {
        return false;
    }
    let Some(hint) = hints.hint_display(asset_type, index) else {
        return false;
    };
    field.value = hint;
    true
}

fn discovery_action(
    asset_type: AssetType,
    form: &TextForm,
    discovery: &mut DiscoveryState,
    pending_identity: &mut Option<String>,
    discovered_identity: &mut Option<String>,
    hints: &mut AssetFieldHints,
) -> FormAction {
    if !asset_type.supports_discovery() {
        return FormAction::None;
    }

    let Some(identity) = asset_identity_from_form(asset_type, form) else {
        *discovery = DiscoveryState::Idle;
        *pending_identity = None;
        *discovered_identity = None;
        hints.clear_metadata();
        return FormAction::None;
    };

    if pending_identity.as_deref() == Some(identity.as_str()) {
        return FormAction::None;
    }

    if discovered_identity.as_deref() == Some(identity.as_str()) {
        return FormAction::None;
    }

    *discovery = DiscoveryState::Loading;
    *pending_identity = Some(identity.clone());
    hints.clear_metadata();
    FormAction::FetchAssetMetadata(identity)
}

impl AssetFieldHints {
    fn clear_metadata(&mut self) {
        self.name = None;
        self.symbol = None;
        self.decimals = None;
        self.icon_url = None;
    }
}

fn asset_identity_from_form(asset_type: AssetType, form: &TextForm) -> Option<String> {
    match asset_type {
        AssetType::Erc20 => {
            let network_id = form.field(0)?.trim().parse::<u64>().ok()?;
            let address = form.field(1)?.trim();
            if address.len() < 42 || !address.starts_with("0x") {
                return None;
            }
            Some(format!("erc20:{network_id}:{address}"))
        }
        AssetType::Native => {
            let network_id = form.field(0)?.trim().parse::<u64>().ok()?;
            Some(format!("native:{network_id}"))
        }
        AssetType::Fiat => None,
    }
}

fn looks_like_ens_name(value: &str) -> bool {
    !value.starts_with("0x") && value.len() >= 3 && value.contains('.') && !value.ends_with('.')
}

fn looks_like_address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].chars().all(|ch| ch.is_ascii_hexdigit())
}

fn ens_actions(
    supports_ens: bool,
    form: &TextForm,
    ens_hint: &mut Option<String>,
    ens_pending: &mut Option<String>,
    ens_primary: &mut Option<String>,
    ens_primary_pending: &mut Option<String>,
) -> FormAction {
    if !supports_ens {
        return FormAction::None;
    }

    let address_value = form.field(1).unwrap_or_default().trim().to_string();
    if !looks_like_address(&address_value) {
        *ens_primary = None;
        *ens_primary_pending = None;
    }

    let focused_value = form
        .field(form.focus)
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    if looks_like_ens_name(&focused_value) {
        if ens_pending.as_deref() == Some(focused_value.as_str()) {
            return FormAction::None;
        }
        *ens_hint = None;
        *ens_pending = Some(focused_value.clone());
        return FormAction::ResolveEns(focused_value);
    }
    if form.focus == 1 {
        *ens_hint = None;
        *ens_pending = None;
    }

    if looks_like_address(&address_value)
        && ens_primary_pending.as_deref() != Some(address_value.to_lowercase().as_str())
    {
        *ens_primary = None;
        *ens_primary_pending = Some(address_value.to_lowercase());
        return FormAction::ReverseEns(address_value);
    }

    FormAction::None
}

fn handle_text_form(
    code: KeyCode,
    form: &mut TextForm,
    submit: impl FnOnce(&TextForm) -> Option<FormAction>,
) -> FormAction {
    match code {
        KeyCode::Esc => FormAction::Cancel,
        KeyCode::Up => {
            form.move_focus(-1);
            FormAction::None
        }
        KeyCode::Down => {
            form.move_focus(1);
            FormAction::None
        }
        KeyCode::Tab => {
            form.move_focus(1);
            FormAction::None
        }
        KeyCode::BackTab => {
            form.move_focus(-1);
            FormAction::None
        }
        KeyCode::Backspace => {
            form.edit_current(|value| {
                value.pop();
            });
            FormAction::None
        }
        KeyCode::Char(ch) if !ch.is_control() => {
            form.edit_current(|value| value.push(ch));
            FormAction::None
        }
        KeyCode::Enter if form.can_submit() => submit(form).unwrap_or(FormAction::None),
        _ => FormAction::None,
    }
}

fn asset_form(asset_type: AssetType) -> TextForm {
    let fields = match asset_type {
        AssetType::Erc20 => vec![
            ("Network ID", true),
            ("Contract address", true),
            ("Name", true),
            ("Symbol", true),
            ("Decimals", true),
            ("Icon URL", false),
        ],
        AssetType::Native => vec![
            ("Network ID", true),
            ("Name", true),
            ("Symbol", true),
            ("Decimals", true),
            ("Icon URL", false),
        ],
        AssetType::Fiat => vec![
            ("Currency code", true),
            ("Name", true),
            ("Symbol", true),
            ("Decimals", true),
            ("Icon URL", false),
        ],
    };

    TextForm::new(format!("Add asset · {}", asset_type.label()), fields)
}

fn network_form() -> TextForm {
    TextForm::new(
        "Add network · new",
        vec![("Chain ID", true), ("Name", true), ("Icon URL", false)],
    )
}

fn build_asset(asset_type: AssetType, form: &TextForm) -> Option<Asset> {
    let icon = optional_field(form, icon_field_index(asset_type));
    match asset_type {
        AssetType::Erc20 => {
            let network_id = form.field(0)?.trim().parse::<u64>().ok()?;
            let address = form.field(1)?.trim();
            let identity =
                AssetIdentity::from_str(&format!("erc20:{network_id}:{address}")).ok()?;
            Some(Asset {
                asset_identity: identity,
                asset_name: form.field(2)?.trim().to_string(),
                asset_symbol: form.field(3)?.trim().to_string(),
                asset_decimals: form.field(4)?.trim().parse().ok()?,
                asset_icon_url: icon,
            })
        }
        AssetType::Native => {
            let network_id = form.field(0)?.trim().parse::<u64>().ok()?;
            let identity = AssetIdentity::from_str(&format!("native:{network_id}")).ok()?;
            Some(Asset {
                asset_identity: identity,
                asset_name: form.field(1)?.trim().to_string(),
                asset_symbol: form.field(2)?.trim().to_string(),
                asset_decimals: form.field(3)?.trim().parse().ok()?,
                asset_icon_url: icon,
            })
        }
        AssetType::Fiat => {
            let code = form.field(0)?.trim().to_string();
            let identity = AssetIdentity::from_str(&format!("fiat:{code}")).ok()?;
            Some(Asset {
                asset_identity: identity,
                asset_name: form.field(1)?.trim().to_string(),
                asset_symbol: form.field(2)?.trim().to_string(),
                asset_decimals: form.field(3)?.trim().parse().ok()?,
                asset_icon_url: icon,
            })
        }
    }
}

fn icon_field_index(asset_type: AssetType) -> usize {
    match asset_type {
        AssetType::Erc20 => 5,
        AssetType::Native | AssetType::Fiat => 4,
    }
}

fn build_network(form: &TextForm) -> Option<Network> {
    let chain_id = form.field(0)?.trim().parse::<u64>().ok()?;
    let name = form.field(1)?.trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some(Network {
        network_identity: NetworkIdentity(chain_id),
        network_name: name,
        network_icon_url: optional_field(form, 2),
    })
}

fn build_endpoint(form: &TextForm) -> Option<NetworkEndpointCreate> {
    let url = form.field(1)?.trim().to_string();
    if url.is_empty() {
        return None;
    }
    let endpoint_type = form.field(2)?.trim();
    let endpoint_type = if endpoint_type.is_empty() {
        "http".to_string()
    } else {
        endpoint_type.to_string()
    };
    let enabled = form.field(3)?.trim().eq_ignore_ascii_case("yes")
        || form.field(3)?.trim().eq_ignore_ascii_case("y")
        || form.field(3)?.trim().eq_ignore_ascii_case("true")
        || form.field(3)?.trim() == "1"
        || form.field(3)?.trim().is_empty();
    Some(NetworkEndpointCreate {
        endpoint_label: optional_field(form, 0),
        endpoint_type,
        endpoint_url: url,
        endpoint_disabled: !enabled,
    })
}

fn optional_field(form: &TextForm, index: usize) -> Option<String> {
    let value = form.field(index)?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
