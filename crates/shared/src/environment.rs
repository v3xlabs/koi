/// Environment Bindings (implemented differently for each platform)
pub struct Environment {
    pub database: String,
    pub http: reqwest::Client,
}
