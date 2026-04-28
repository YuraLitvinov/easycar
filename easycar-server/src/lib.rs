use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use info_car_api::{client::Client, types::AddEmployeeRequest};
use tokio::sync::Mutex;

mod vault;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Mutex<Client>>,
    pub employer_id: String,
}

async fn create_employee(
    State(state): State<AppState>,
    Json(body): Json<AddEmployeeRequest>,
) -> StatusCode {
    if std::env::var("DEV_MODE")
        .map(|v| v == "true")
        .unwrap_or(false)
    {
        return StatusCode::OK;
    }
    let mut client = Client::new();
    let username = dotenvy::var("USER_HANDLER").expect("USER_HANDLER not set in .env");
    let password = dotenvy::var("PASS").expect("PASS not set in .env");
    if let Err(e) = client.login(&username, &password).await {
        panic!("Login failed: {:?}", e);
    }

    let result = client.add_employee(state.employer_id, body).await;
    tracing::info!("Add employee result: {:?}", result);
    StatusCode::CREATED
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/add-employee", post(create_employee))
        .with_state(state)
}

pub async fn build_state(
    username: String,
    password: String,
    employer_id: String,
    dev_mode: bool,
) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
    let mut client = Client::new();
    if !dev_mode {
        client.login(&username, &password).await?;
    }

    Ok(AppState {
        client: Arc::new(Mutex::new(client)),
        employer_id,
    })
}

pub async fn build_state_from_env() -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
    let dev_mode = std::env::var("DEV_MODE")
        .map(|v| v == "true")
        .unwrap_or(false);

    let (username, password, employer_id) = if dev_mode {
        (
            "dev".to_string(),
            "dev".to_string(),
            "dev_employer".to_string(),
        )
    } else {
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
        }
    };

    build_state(username, password, employer_id, dev_mode).await
}

pub async fn run_server(
    addr: &str,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
