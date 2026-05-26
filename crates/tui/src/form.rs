use std::str::FromStr;

use crossterm::event::KeyCode;
use koi::models::{
    asset::{Asset, identity::AssetIdentity, metadata::AssetMetadataDiscovery},
    network::{Network, endpoint::NetworkEndpoint, identity::NetworkIdentity},
};

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
        next_id: i32,
        form: TextForm,
    },
}

#[derive(Debug)]
pub enum FormAction {
    None,
    Cancel,
    OpenNetworkPresets,
    FetchAssetMetadata(String),
    SubmitCreateAsset(Asset),
    SubmitCreateNetwork(Network),
    SubmitCreateEndpoint(NetworkEndpoint),
}

impl ActiveForm {
    pub fn open_add_asset() -> Self {
        Self::AddAssetType { selected: 0 }
    }

    pub fn open_add_network() -> Self {
        Self::AddNetworkMode { selected: 0 }
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
        Self::AddEndpoint {
            network_id,
            next_id: 0,
            form,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::AddAssetType { .. } => "Add asset · choose type",
            Self::AddAsset { form, .. } => &form.title,
            Self::AddNetworkMode { .. } => "Add network · choose mode",
            Self::AddNetwork(form) => &form.title,
            Self::AddNetworkPreset { .. } => "Add network · choose preset",
            Self::AddEndpoint { form, .. } => &form.title,
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
            Self::AddEndpoint {
                network_id,
                next_id,
                form,
            } => handle_text_form(code, form, |form| {
                build_endpoint(*network_id, *next_id, form).map(FormAction::SubmitCreateEndpoint)
            }),
        }
    }

    pub fn set_endpoint_next_id(&mut self, next_id: i32) {
        if let Self::AddEndpoint { next_id: id, .. } = self {
            *id = next_id;
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
        KeyCode::Up | KeyCode::Char('k') => {
            form.move_focus(-1);
            FormAction::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
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

fn handle_text_form(
    code: KeyCode,
    form: &mut TextForm,
    submit: impl FnOnce(&TextForm) -> Option<FormAction>,
) -> FormAction {
    match code {
        KeyCode::Esc => FormAction::Cancel,
        KeyCode::Up | KeyCode::Char('k') => {
            form.move_focus(-1);
            FormAction::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
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

fn build_endpoint(network_id: u64, next_id: i32, form: &TextForm) -> Option<NetworkEndpoint> {
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
    Some(NetworkEndpoint {
        endpoint_identity: next_id,
        endpoint_label: optional_field(form, 0),
        endpoint_type,
        endpoint_url: url,
        endpoint_disabled: !enabled,
        network_identity: NetworkIdentity(network_id),
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

pub fn available_presets(existing_network_ids: &[u64]) -> Vec<Network> {
    let existing = existing_network_ids
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    Network::presets()
        .into_iter()
        .filter(|preset| !existing.contains(&preset.network_identity.0))
        .collect()
}
