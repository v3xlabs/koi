use poem::{
    Error, Route, Server, error::Unauthorized, listener::TcpListener
};
use poem_openapi::{
    auth::Bearer,
    payload::PlainText,
    OpenApi, OpenApiService, SecurityScheme,
};
use reqwest::StatusCode;
use tracing::{debug, info};

#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    bearer_format = "JWT"
)]
pub struct Auth(Bearer);

pub struct AuthData {
    pub user_id: String,
}

impl Auth {
    pub fn validate(&self) -> Result<(), poem::Error> {
        debug!("Validating auth {:?}", self.0);

        // Ok(())
        Err(Unauthorized(Error::from_status(StatusCode::UNAUTHORIZED)))
    }

    pub fn unwrap(self) -> Result<AuthData, poem::Error> {
        self.validate()?;

        Ok(AuthData {
            user_id: "123".to_string(),
        })
    }
}
