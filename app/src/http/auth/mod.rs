use poem::{Error, Route, Server, error::Unauthorized, listener::TcpListener};
use poem_openapi::{OpenApi, OpenApiService, SecurityScheme, auth::Bearer, payload::PlainText};
use reqwest::StatusCode;
use tracing::{debug, info};

#[derive(SecurityScheme)]
#[oai(ty = "bearer", bearer_format = "JWT")]
pub struct Auth(Bearer);

pub struct AuthData {
    pub user_id: String,
}

impl Auth {
    pub fn validate(&self) -> Result<(), poem::Error> {
        info!("Validating auth {:?}", self.0);

        Ok(())
    }

    pub fn unwrap(self) -> Result<AuthData, poem::Error> {
        self.validate()?;

        Ok(AuthData {
            user_id: "123".to_string(),
        })
    }
}
