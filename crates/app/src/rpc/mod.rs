//! Definition-site RPC registration.
//!
//! `rpc_method!` declares a method next to the model it serves: it expands the
//! marker type, its `RpcMethod` contract, and a [`MethodRecord`] entry in the
//! [`RPC_METHODS`] distributed slice. The dispatcher and the bindings
//! generator both walk that slice, so the declaration plus an `RpcHandler`
//! impl in the same file is the entire integration — there is no registry to
//! update. A declaration without a handler impl fails to compile.

pub mod bindings;

use std::future::Future;

use futures::future::BoxFuture;
use linkme::distributed_slice;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use ts_rs::TS;

use crate::{error::KoiError, state::AppState};

#[distributed_slice]
pub static RPC_METHODS: [MethodRecord];

pub trait RpcMethod {
    type Params: Serialize + DeserializeOwned + TS + 'static;
    type Output: Serialize + DeserializeOwned + TS + 'static;
    const NAME: &'static str;
}

pub trait RpcHandler: RpcMethod {
    fn handle(
        state: &AppState,
        params: Self::Params,
    ) -> impl Future<Output = Result<Self::Output, KoiError>> + Send;
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[serde(deny_unknown_fields)]
pub struct EmptyParams {}

pub enum RpcCallError {
    InvalidParams,
    Domain(KoiError),
    Encode(serde_json::Error),
}

pub struct MethodRecord {
    pub name: &'static str,
    pub marker: &'static str,
    pub dispatch: for<'a> fn(&'a AppState, Value) -> BoxFuture<'a, Result<Value, RpcCallError>>,
    pub collect_types: fn(&mut bindings::DeclarationCollector<'_>),
    pub params_ts_name: fn(&ts_rs::Config) -> String,
    pub output_ts_name: fn(&ts_rs::Config) -> String,
    pub takes_params: fn() -> bool,
}

#[macro_export]
macro_rules! rpc_method {
    ($marker:ident, $name:literal, $params:ty => $output:ty) => {
        pub struct $marker;

        impl $crate::rpc::RpcMethod for $marker {
            type Params = $params;
            type Output = $output;
            const NAME: &'static str = $name;
        }

        const _: () = {
            fn dispatch<'a>(
                state: &'a $crate::state::AppState,
                params: ::serde_json::Value,
            ) -> ::futures::future::BoxFuture<
                'a,
                Result<::serde_json::Value, $crate::rpc::RpcCallError>,
            > {
                Box::pin(async move {
                    let params = ::serde_json::from_value::<$params>(params)
                        .map_err(|_| $crate::rpc::RpcCallError::InvalidParams)?;
                    let output = <$marker as $crate::rpc::RpcHandler>::handle(state, params)
                        .await
                        .map_err($crate::rpc::RpcCallError::Domain)?;
                    ::serde_json::to_value(output).map_err($crate::rpc::RpcCallError::Encode)
                })
            }

            fn collect_types(collector: &mut $crate::rpc::bindings::DeclarationCollector<'_>) {
                collector.collect::<$params>();
                collector.collect::<$output>();
            }

            fn params_ts_name(config: &::ts_rs::Config) -> String {
                <$params as ::ts_rs::TS>::name(config)
            }

            fn output_ts_name(config: &::ts_rs::Config) -> String {
                <$output as ::ts_rs::TS>::name(config)
            }

            fn takes_params() -> bool {
                ::std::any::TypeId::of::<$params>()
                    != ::std::any::TypeId::of::<$crate::rpc::EmptyParams>()
            }

            #[::linkme::distributed_slice($crate::rpc::RPC_METHODS)]
            static RECORD: $crate::rpc::MethodRecord = $crate::rpc::MethodRecord {
                name: $name,
                marker: ::std::stringify!($marker),
                dispatch,
                collect_types,
                params_ts_name,
                output_ts_name,
                takes_params,
            };
        };
    };
}

crate::rpc_method!(SystemPing, "system.ping", EmptyParams => String);

impl RpcHandler for SystemPing {
    async fn handle(_state: &AppState, _params: EmptyParams) -> Result<String, KoiError> {
        Ok("OK".to_string())
    }
}
