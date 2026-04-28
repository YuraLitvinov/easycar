use serde::Deserialize;
use snafu::{ResultExt, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
pub enum VaultError {
    #[snafu(display("HTTP request failed: {source}"))]
    HttpError { source: reqwest::Error },
    #[snafu(display("Vault returned error status: {status}"))]
    VaultError { status: String },
    #[snafu(display("Secret key not found: {key}"))]
    KeyNotFound { key: String },
    #[snafu(display("JSON parse error: {source}"))]
    JsonError { source: reqwest::Error },
}

#[derive(Debug, Deserialize)]
struct VaultResponse {
    data: SecretData,
}

#[derive(Debug, Deserialize)]
struct SecretData {
    data: HashMap<String, String>,
}

pub struct VaultClient {
    addr: String,
    token: String,
    http: reqwest::Client,
}

impl VaultClient {
    pub fn new(addr: String, token: String) -> Self {
        Self {
            addr,
            token,
            http: reqwest::Client::new(),
        }
    }

    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>, VaultError> {
        let url = format!("{}/v1/secret/data/{}", self.addr, path);

        let mut retries = 5u32;
        let mut delay = std::time::Duration::from_secs(1);

        loop {
            let resp = self
                .http
                .get(&url)
                .header("X-Vault-Token", &self.token)
                .send()
                .await
                .context(HttpSnafu)?;

            tracing::info!("Vault response for {}: {}", path, resp.status());

            if resp.status().is_success() {
                let vault_resp: VaultResponse = resp.json().await.context(JsonSnafu)?;
                return Ok(vault_resp.data.data);
            }

            if (resp.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE
                || resp.status() == reqwest::StatusCode::NOT_FOUND)
                && retries > 0
            {
                tracing::warn!(
                    "Vault not ready (status {}), retrying in {:?}...",
                    resp.status(),
                    delay
                );
                tokio::time::sleep(delay).await;
                retries -= 1;
                delay *= 2;
                continue;
            }

            return Err(VaultError::VaultError {
                status: resp.status().to_string(),
            });
        }
    }

    pub async fn get_infocar_credentials(&self) -> Result<(String, String, String), VaultError> {
        let path = std::env::var("VAULT_SECRET_PATH").unwrap_or_else(|_| "easycar".to_string());
        let secret = self.get_secret(&path).await?;

        let username = secret
            .get("username")
            .ok_or_else(|| VaultError::KeyNotFound {
                key: "username".to_string(),
            })?
            .clone();
        let password = secret
            .get("password")
            .ok_or_else(|| VaultError::KeyNotFound {
                key: "password".to_string(),
            })?
            .clone();
        let employer_id = secret
            .get("employer_id")
            .ok_or_else(|| VaultError::KeyNotFound {
                key: "employer_id".to_string(),
            })?
            .clone();

        Ok((username, password, employer_id))
    }
}
