//! Config manager, reading the content of the `.env` file.
//!
//! Reads all content from `.env` into [`Config`] for all sub-modules to use.

use crate::error::Error;

pub struct Config {
    /// Database URL with the following structure `postgres://username:password@host/database_name`.
    pub database_url: String,

    /// Etherscan API token.
    pub token_etherscan: String,

    /// GitHub API tokens.
    pub tokens_github: Vec<String>,

    /// Etherface REST API address, e.g. <https://api.etherface.io>
    pub rest_address: String,

    pub vault: VaultConfig,
}

pub struct VaultConfig {
    /// Vault address.
    pub address: String,

    /// Vault auth
    pub auth: VaultAuth,

    /// Vault secret
    pub secret: VaultSecret,
}

pub struct VaultAuth {
    /// Vault auth method
    pub method: String,

    /// Vault auth path
    pub path: String,

    /// Vault auth role
    pub role: String,

    /// Vault token (optional)
    pub token: Option<String>,
}

pub struct VaultSecret {
    /// Vault mount
    pub mount: String,

    /// Vault path
    pub path: String,

    /// Vault param org_name
    pub org_name: String,
}

const ENV_VAR_DATABASE_URL: &str = "ETHERFACE_DATABASE_URL";
const ENV_VAR_TOKEN_ETHERSCAN: &str = "ETHERFACE_TOKEN_ETHERSCAN";
const ENV_VAR_TOKENS_GITHUB: &str = "ETHERFACE_TOKENS_GITHUB";
const ENV_VAR_REST_ADDRESS: &str = "ETHERFACE_REST_ADDRESS";

const ENV_VAR_VAULT_ADDR: &str = "VAULT_ADDR";
const ENV_VAR_VAULT_AUTH_METHOD: &str = "VAULT_AUTH_METHOD";
const ENV_VAR_VAULT_AUTH_PATH: &str = "VAULT_AUTH_PATH";
const ENV_VAR_VAULT_AUTH_ROLE: &str = "VAULT_AUTH_ROLE";
const ENV_VAR_VAULT_AUTH_TOKEN: &str = "VAULT_TOKEN";
const ENV_VAR_VAULT_SECRET_MOUNT: &str = "VAULT_SECRET_MOUNT";
const ENV_VAR_VAULT_SECRET_PATH: &str = "VAULT_SECRET_PATH";
const ENV_VAR_VAULT_SECRET_ORG_NAME: &str = "VAULT_SECRET_ORG_NAME";

#[inline]
fn read_and_return_env_var(env_var: &'static str) -> Result<String, Error> {
    let res = std::env::var(env_var)
        .map_err(|err| Error::ConfigReadNonExistantEnvironmentVariable(env_var, err))?;

    match res.is_empty() {
        true => Err(Error::ConfigReadEmptyEnvironmentVariable(env_var)),
        false => Ok(res),
    }
}

impl Config {
    /// Returns a new config manager, reading the content of `.env`.
    pub fn new() -> Result<Self, Error> {
        // match Path::new(".env").exists() {
        //     true => dotenv()?,
        //     false => dotenv::from_filename("../.env")?, // If executed within a sub-directory
        // };

        let database_url = read_and_return_env_var(ENV_VAR_DATABASE_URL)?;
        let token_etherscan = read_and_return_env_var(ENV_VAR_TOKEN_ETHERSCAN)?;
        let rest_address = read_and_return_env_var(ENV_VAR_REST_ADDRESS)?;

        let tokens_github = std::env::var(ENV_VAR_TOKENS_GITHUB)
            .map_err(|err| Error::ConfigReadNonExistantEnvironmentVariable(ENV_VAR_TOKENS_GITHUB, err))?
            .split(',')
            .map(str::to_string)
            .collect::<Vec<String>>();

        // if tokens_github.is_empty() {
        //     return Err(Error::ConfigReadEmptyEnvironmentVariable(ENV_VAR_TOKENS_GITHUB));
        // }

        let vault = VaultConfig {
            address: read_and_return_env_var(ENV_VAR_VAULT_ADDR)
                .unwrap_or("http://127.0.0.1:8200".to_string()),
            auth: VaultAuth {
                method: read_and_return_env_var(ENV_VAR_VAULT_AUTH_METHOD)
                    .unwrap_or("kubernetes".to_string()),
                path: read_and_return_env_var(ENV_VAR_VAULT_AUTH_PATH).unwrap_or("kubernetes".to_string()),
                role: read_and_return_env_var(ENV_VAR_VAULT_AUTH_ROLE).unwrap_or("etherface".to_string()),
                token: None,
            },
            secret: VaultSecret {
                mount: read_and_return_env_var(ENV_VAR_VAULT_SECRET_MOUNT).unwrap_or("etherface".to_string()),
                path: read_and_return_env_var(ENV_VAR_VAULT_SECRET_PATH).unwrap_or("token".to_string()),
                org_name: read_and_return_env_var(ENV_VAR_VAULT_SECRET_ORG_NAME)
                    .unwrap_or("RSS3-Network".to_string()),
            },
        };

        Ok(Config {
            database_url,
            tokens_github,
            token_etherscan,
            rest_address,
            vault,
        })
    }
}
