use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use info_car_api::{client::Client, types::AddEmployeeRequest};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Mutex<Client>>,
    pub employer_id: String,
}

fn require_env(key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    std::env::var(key).map_err(|_| format!("required environment variable `{key}` is not set").into())
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

    let username = match require_env("USER_HANDLER") {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    let password = match require_env("PASS") {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let mut client = Client::new();
    if let Err(e) = client.login(&username, &password).await {
        tracing::error!("easycar login failed: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR;
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
) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
    let mut client = Client::new();
    client.login(&username, &password).await?;

    Ok(AppState {
        client: Arc::new(Mutex::new(client)),
        employer_id,
    })
}

pub async fn build_state_from_env() -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
    let username = require_env("EASYCAR_USERNAME")?;
    let password = require_env("EASYCAR_PASSWORD")?;
    let employer_id = require_env("EASYCAR_EMPLOYER_ID")?;

    build_state(username, password, employer_id).await
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
