use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Account, AccountCreate as AccountCreateInput, AccountUpdate as AccountUpdateInput,
    balances::{AccountBalance, AccountBalances},
    derive::{
        default_derivation_path, derive_address_from_private_key, derive_addresses_from_mnemonic,
        generate_mnemonic,
    },
    group::{
        AccountGroup, AccountGroupCreate as GroupCreateInput,
        AccountGroupUpdate as GroupUpdateInput, GroupIdentity,
    },
    identity::AccountIdentity,
    layout::{AccountLayout, AccountLayoutUpdate as LayoutUpdateInput},
    metadata::WalletType,
};
use crate::{
    error::KoiError,
    models::{
        asset::identity::AssetIdentity,
        tx::{Tx, TxBase},
    },
    rpc::{EmptyParams, RpcHandler},
    rpc_method,
    state::AppState,
    vendor::safe_wallet::tx::fetch_safewallet_tx,
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountParams {
    pub account_identity: AccountIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountAssetParams {
    pub account_identity: AccountIdentity,
    pub asset_identity: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountAssetBalanceParams {
    pub account_identity: AccountIdentity,
    pub asset_identity: AssetIdentity,
    pub display_currency: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct AccountBalancesParams {
    pub account_identity: AccountIdentity,
    pub display_currency: AssetIdentity,
    #[serde(default)]
    pub fresh: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountCreateParams {
    pub input: AccountCreateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountUpdateParams {
    pub account_identity: AccountIdentity,
    pub input: AccountUpdateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct LayoutUpdateParams {
    pub input: LayoutUpdateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct GroupCreateParams {
    pub input: GroupCreateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct GroupUpdateParams {
    pub group_identity: GroupIdentity,
    pub input: GroupUpdateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct GroupParams {
    pub group_identity: GroupIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DeriveMnemonicInput {
    pub mnemonic: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DeriveMnemonicParams {
    pub input: DeriveMnemonicInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DeriveMnemonicResult {
    pub path: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DerivePrivateKeyParams {
    pub input: String,
}

rpc_method!(AccountList, "account.list", EmptyParams => Vec<Account>);
rpc_method!(AccountGet, "account.get", AccountParams => Account);
rpc_method!(AccountCreate, "account.create", AccountCreateParams => Account);
rpc_method!(AccountUpdate, "account.update", AccountUpdateParams => Account);
rpc_method!(AccountDelete, "account.delete", AccountParams => ());
rpc_method!(AccountAssetList, "account.asset.list", AccountParams => Vec<AssetIdentity>);
rpc_method!(AccountAssetAdd, "account.asset.add", AccountAssetParams => ());
rpc_method!(AccountAssetRemove, "account.asset.remove", AccountAssetParams => ());
rpc_method!(AccountAssetBalance, "account.asset.balance", AccountAssetBalanceParams => AccountBalance);
rpc_method!(AccountBalanceList, "account.balance.list", AccountBalancesParams => AccountBalances);
rpc_method!(AccountLayoutGet, "account.layout.get", EmptyParams => AccountLayout);
rpc_method!(AccountLayoutUpdate, "account.layout.update", LayoutUpdateParams => AccountLayout);
rpc_method!(AccountGroupCreate, "account.group.create", GroupCreateParams => AccountGroup);
rpc_method!(AccountGroupUpdate, "account.group.update", GroupUpdateParams => AccountGroup);
rpc_method!(AccountGroupDelete, "account.group.delete", GroupParams => ());
rpc_method!(AccountTransactionList, "account.transaction.list", AccountParams => Vec<Tx>);
rpc_method!(AccountTransactionPending, "account.transaction.pending", AccountParams => Vec<Tx>);
rpc_method!(AccountMnemonicGenerate, "account.mnemonic.generate", EmptyParams => String);
rpc_method!(AccountDerivationDefaultPath, "account.derivation.defaultPath", EmptyParams => String);
rpc_method!(AccountDerivationFromMnemonic, "account.derivation.fromMnemonic", DeriveMnemonicParams => Vec<DeriveMnemonicResult>);
rpc_method!(AccountDerivationFromPrivateKey, "account.derivation.fromPrivateKey", DerivePrivateKeyParams => String);

impl RpcHandler for AccountList {
    async fn handle(state: &AppState, _params: EmptyParams) -> Result<Vec<Account>, KoiError> {
        Account::all(&state.database).await
    }
}

impl RpcHandler for AccountGet {
    async fn handle(state: &AppState, params: AccountParams) -> Result<Account, KoiError> {
        Account::get_by_id(&state.database, params.account_identity).await
    }
}

impl RpcHandler for AccountCreate {
    async fn handle(state: &AppState, params: AccountCreateParams) -> Result<Account, KoiError> {
        Account::create(&state.database, params.input).await
    }
}

impl RpcHandler for AccountUpdate {
    async fn handle(state: &AppState, params: AccountUpdateParams) -> Result<Account, KoiError> {
        Account::update(&state.database, params.account_identity, params.input).await
    }
}

impl RpcHandler for AccountDelete {
    async fn handle(state: &AppState, params: AccountParams) -> Result<(), KoiError> {
        Account::delete(&state.database, params.account_identity).await
    }
}

impl RpcHandler for AccountAssetList {
    async fn handle(
        state: &AppState,
        params: AccountParams,
    ) -> Result<Vec<AssetIdentity>, KoiError> {
        Account::get_assets(&state.database, params.account_identity).await
    }
}

impl RpcHandler for AccountAssetAdd {
    async fn handle(state: &AppState, params: AccountAssetParams) -> Result<(), KoiError> {
        Account::add_asset(
            &state.database,
            params.account_identity,
            params.asset_identity,
        )
        .await
    }
}

impl RpcHandler for AccountAssetRemove {
    async fn handle(state: &AppState, params: AccountAssetParams) -> Result<(), KoiError> {
        Account::remove_asset(
            &state.database,
            params.account_identity,
            params.asset_identity,
        )
        .await
    }
}

impl RpcHandler for AccountAssetBalance {
    async fn handle(
        state: &AppState,
        params: AccountAssetBalanceParams,
    ) -> Result<AccountBalance, KoiError> {
        let account = Account::get_by_id(&state.database, params.account_identity).await?;
        account
            .fetch_asset_balance(state, &params.asset_identity, &params.display_currency)
            .await
    }
}

impl RpcHandler for AccountBalanceList {
    async fn handle(
        state: &AppState,
        params: AccountBalancesParams,
    ) -> Result<AccountBalances, KoiError> {
        let account = Account::get_by_id(&state.database, params.account_identity).await?;
        state
            .balances
            .get_balances(
                state,
                &account,
                &params.display_currency,
                params.fresh.unwrap_or(false),
            )
            .await
    }
}

impl RpcHandler for AccountLayoutGet {
    async fn handle(state: &AppState, _params: EmptyParams) -> Result<AccountLayout, KoiError> {
        AccountLayout::get(&state.database).await
    }
}

impl RpcHandler for AccountLayoutUpdate {
    async fn handle(
        state: &AppState,
        params: LayoutUpdateParams,
    ) -> Result<AccountLayout, KoiError> {
        AccountLayout::update(&state.database, params.input).await
    }
}

impl RpcHandler for AccountGroupCreate {
    async fn handle(state: &AppState, params: GroupCreateParams) -> Result<AccountGroup, KoiError> {
        AccountGroup::create(&state.database, params.input.name).await
    }
}

impl RpcHandler for AccountGroupUpdate {
    async fn handle(state: &AppState, params: GroupUpdateParams) -> Result<AccountGroup, KoiError> {
        AccountGroup::update(&state.database, params.group_identity, params.input).await
    }
}

impl RpcHandler for AccountGroupDelete {
    async fn handle(state: &AppState, params: GroupParams) -> Result<(), KoiError> {
        AccountGroup::delete(&state.database, params.group_identity).await
    }
}

impl RpcHandler for AccountTransactionList {
    async fn handle(state: &AppState, params: AccountParams) -> Result<Vec<Tx>, KoiError> {
        account_transactions(state, params.account_identity).await
    }
}

impl RpcHandler for AccountTransactionPending {
    async fn handle(state: &AppState, params: AccountParams) -> Result<Vec<Tx>, KoiError> {
        let account = Account::get_by_id(&state.database, params.account_identity).await?;
        account
            .metadata
            .unwrap_address()
            .ok_or_else(|| KoiError::InvalidInput("account has no address".to_string()))?;
        Ok(Vec::new())
    }
}

impl RpcHandler for AccountMnemonicGenerate {
    async fn handle(_state: &AppState, _params: EmptyParams) -> Result<String, KoiError> {
        generate_mnemonic()
    }
}

impl RpcHandler for AccountDerivationDefaultPath {
    async fn handle(_state: &AppState, _params: EmptyParams) -> Result<String, KoiError> {
        Ok(default_derivation_path().to_string())
    }
}

impl RpcHandler for AccountDerivationFromMnemonic {
    async fn handle(
        _state: &AppState,
        params: DeriveMnemonicParams,
    ) -> Result<Vec<DeriveMnemonicResult>, KoiError> {
        derive_addresses_from_mnemonic(&params.input.mnemonic, &params.input.paths).map(|values| {
            values
                .into_iter()
                .map(|(path, address)| DeriveMnemonicResult {
                    path,
                    address: address.to_checksum(None),
                })
                .collect::<Vec<_>>()
        })
    }
}

impl RpcHandler for AccountDerivationFromPrivateKey {
    async fn handle(_state: &AppState, params: DerivePrivateKeyParams) -> Result<String, KoiError> {
        derive_address_from_private_key(&params.input).map(|address| address.to_checksum(None))
    }
}

async fn account_transactions(
    state: &AppState,
    identity: AccountIdentity,
) -> Result<Vec<Tx>, KoiError> {
    let account = Account::get_by_id(&state.database, identity).await?;
    let bases = match account.metadata {
        WalletType::Safe(safe) => stream::iter(account.networks)
            .map(|network| async move {
                fetch_safewallet_tx(network, safe.evm_address.0)
                    .await
                    .map(|response| {
                        response
                            .results
                            .into_iter()
                            .filter_map(|tx| tx.try_into().ok())
                            .collect::<Vec<TxBase>>()
                    })
                    .unwrap_or_default()
            })
            .buffered(8)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    Ok(stream::iter(bases)
        .map(|tx| async move { tx.decode(state).await.ok() })
        .buffered(8)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect())
}
