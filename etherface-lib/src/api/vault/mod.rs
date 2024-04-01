use crate::config::Config;
use derive_builder::Builder;
use futures::executor::block_on;
use log::debug;
use rustify_derive::Endpoint;
use serde::Deserialize;
use std::{collections::HashMap, error::Error, fs::read_to_string, result::Result};
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
    token: Option<GithubResponse>,
    mount: String,
    path: String,
    org_name: String,
}

impl VaultManager {
    /// Returns a new token manager.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let vault_config = Config::new()?.vault;

        let mut client =
            VaultClient::new(VaultClientSettingsBuilder::default().address(vault_config.address).build()?)?;

        match vault_config.auth.method.as_str() {
            "kubernetes" => {
                let jwt = read_to_string(SERVICE_ACCOUNT_TOKEN_PATH)?;
                let auth = block_on(login(&client, &vault_config.auth.path, &vault_config.auth.role, &jwt))?;
                debug!("Authenticated to Vault with Kubernetes auth method {:?}", auth);
                client.set_token(&auth.client_token);
            }
            "token" => match vault_config.auth.token {
                Some(token) => {
                    client.set_token(token.as_str());
                }
                None => {
                    return Err("Token auth method requires a token".into());
                }
            },
            _ => {
                return Err(format!("Unsupported auth method: {}", vault_config.auth.method).into());
            }
        }

        let manager = VaultManager {
            client,
            mount: vault_config.secret.mount,
            path: vault_config.secret.path,
            org_name: vault_config.secret.org_name,
            token: None,
        };

        Ok(manager)
    }

    pub fn get_token(&mut self) -> Result<String, Box<dyn Error>> {
        // if token is expired or not exist, renew it
        if let Some(token) = &self.token {
            if token.data.expires_at < chrono::Utc::now().to_rfc3339() {
                block_on(self.renew_token())?
            }
        } else {
            block_on(self.renew_token())?
        }

        return match &self.token {
            Some(token) => Ok(token.data.token.clone()),
            None => Err("Token not found".into()),
        };
    }

    pub async fn renew_token(&mut self) -> Result<(), Box<dyn Error>> {
        let endpoint = GithubRequestBuilder::default()
            .mount(self.mount.clone())
            .path(self.path.clone())
            .org_name(self.org_name.clone())
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
