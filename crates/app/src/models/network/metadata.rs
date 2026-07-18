use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::KoiError,
    models::{network::identity::NetworkIdentity, vendor::flags::VendorFlag},
    state::AppState,
    vendor::{safe_wallet, smoldapp},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMetadataOption {
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMetadataDiscovery {
    pub network_identity: NetworkIdentity,
    pub options: HashMap<String, NetworkMetadataOption>,
}

impl super::Network {
    pub async fn fetch_metadata(
        state: &AppState,
        network_identity: &NetworkIdentity,
    ) -> Result<NetworkMetadataDiscovery, KoiError> {
        let icon_options: Vec<Option<(Result<String, KoiError>, String)>> = vec![
            match state.vendors.has_flag(VendorFlag::SmoldappNetworkIcons) {
                true => Some((
                    smoldapp::fetch_network_icon(network_identity.0).await,
                    "smoldapp".to_string(),
                )),
                false => None,
            },
            match state.vendors.has_flag(VendorFlag::SafewalletNetworkIcons) {
                true => Some((
                    safe_wallet::fetch_network_icon(network_identity.0).await,
                    "safe".to_string(),
                )),
                false => None,
            },
        ];

        let mut options = HashMap::new();

        for option in icon_options {
            if let Some((Ok(url), name)) = option {
                options.insert(
                    name.to_string(),
                    NetworkMetadataOption {
                        icon_url: Some(url),
                    },
                );
            }
        }

        Ok(NetworkMetadataDiscovery {
            network_identity: network_identity.clone(),
            options,
        })
    }
}
