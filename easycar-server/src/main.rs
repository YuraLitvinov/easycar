mod vault;

use std::sync::Arc;

use easycar_server::{AppState, build_router};
use info_car_api::client::Client;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[forbid(unsafe_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (username, password, employer_id) =
        match (std::env::var("VAULT_ADDR"), std::env::var("VAULT_TOKEN")) {
            (Ok(addr), Ok(token)) => {
                vault::VaultClient::new(addr, token)
                    .get_infocar_credentials()
                    .await?
            }
            _ => {
                dotenvy::dotenv().ok();
                (
                    std::env::var("USERNAME")?,
                    std::env::var("PASSWORD")?,
                    std::env::var("EMPLOYER_ID")?,
                )
            }
        };

    let mut client = Client::new();
    client.login(&username, &password).await?;

    let state = AppState {
        client: Arc::new(Mutex::new(client)),
        employer_id,
    };
    let app = build_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
