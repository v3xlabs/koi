#[derive(Debug, Serialize, Deserialize, FromRow, Object)]
pub struct NetworkEndpoint {
    pub endpoint_identity: String,
    pub endpoint_label: Option<String>,
    pub endpoint_type: String,
    pub endpoint_url: String,
    pub endpoint_priority: u32,
    pub endpoint_disabled: bool,
    pub network_identity: NetworkIdentity,
}
