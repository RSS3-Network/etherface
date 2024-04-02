use crate::config::{Config, VaultConfig};
use derive_builder::Builder;
use rustify_derive::Endpoint;
use serde::Deserialize;
use std::{collections::HashMap, error::Error, fs::read_to_string, result::Result};
use tokio;
use vaultrs::{
    api,
    auth::kubernetes::login,
    client::{Client, VaultClient, VaultClientSettingsBuilder},
};

const SERVICE_ACCOUNT_TOKEN_PATH: &str = "/var/run/secrets/kubernetes.io/serviceaccount/token";

#[derive(Debug, Builder, Endpoint)]
#[endpoint(
    path = "{self.mount}/{self.path}",
    method = "GET",
    response = "GithubResponse",
    builder = "true"
)]
#[builder(setter(into))]
struct GithubRequest {
    #[endpoint(skip)]
    mount: String,
    path: String,
    #[endpoint(query)]
    org_name: String,
}

#[derive(Deserialize, Debug)]
struct GithubResponse {
    pub request_id: String,
    pub lease_id: String,
    pub renewable: bool,
    pub lease_duration: i32,
    pub data: GithubResponseData,
    pub wrap_info: Option<String>,
    pub warnings: Option<String>,
    pub auth: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GithubResponseData {
    pub expires_at: String,
    pub installation_id: i32,
    pub org_name: String,
    pub permissions: HashMap<String, String>,
    pub repository_selection: String,
    pub token: String,
}

pub(crate) struct VaultManager {
    client: VaultClient,
    config: VaultConfig,
    token: Option<GithubResponse>,
}

impl VaultManager {
    /// Returns a new token manager.
    #[tokio::main]
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let config = Config::new()?.vault;

        let client =
            VaultClient::new(VaultClientSettingsBuilder::default().address(config.address.clone()).build()?)?;

        let mut manager = VaultManager {
            client,
            config,
            token: None,
        };
        manager.auth()?;

        Ok(manager)
    }
    // set vault auth method
    #[tokio::main]
    pub async fn auth(&mut self) -> Result<(), Box<dyn Error>> {
        match self.config.auth.method.as_str() {
            "kubernetes" => {
                let jwt = read_to_string(SERVICE_ACCOUNT_TOKEN_PATH)?;
                let auth = login(&self.client, &self.config.auth.path, &self.config.auth.role, &jwt).await?;
                self.client.set_token(&auth.client_token);
            }
            "token" => match &self.config.auth.token {
                Some(token) => {
                    self.client.set_token(token.as_str());
                }
                None => {
                    return Err("Token auth method requires a token".into());
                }
            },
            _ => {
                return Err(format!("Unsupported auth method: {}", self.config.auth.method).into());
            }
        }
        Ok(())
    }

    #[tokio::main]
    pub async fn get_token(&mut self) -> Result<String, Box<dyn Error>> {
        // if token is expired or not exist, renew it
        if let Some(token) = &self.token {
            if token.data.expires_at < chrono::Utc::now().to_rfc3339() {
                self.renew_token().await?;
            }
        } else {
            self.renew_token().await?;
        }

        return match &self.token {
            Some(token) => Ok(token.data.token.clone()),
            None => Err("Token not found".into()),
        };
    }

    pub async fn renew_token(&mut self) -> Result<(), Box<dyn Error>> {
        self.auth()?;
        let endpoint = GithubRequestBuilder::default()
            .mount(self.config.secret.mount.clone())
            .path(self.config.secret.path.clone())
            .org_name(self.config.secret.org_name.clone())
            .build()?;

        match api::exec_with_no_result(&self.client, endpoint).await {
            Ok(response) => {
                self.token = Some(response);
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}
