use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use info_car_api::{client::Client, error::GenericClientError, types::AddEmployeeRequest};
use serde_json::{Value, json};
use tokio::signal;

#[derive(Clone)]
pub struct AppState {
    pub employer_id: String,
    pub username: String,
    pub password: String,
}

async fn create_employee(
    State(state): State<AppState>,
    Json(body): Json<AddEmployeeRequest>,
) -> (StatusCode, Json<Value>) {
    if std::env::var("DEV_STATE").unwrap_or("false".to_string()) == "true" {
        let jbody = json!(body);
        println!("{}\n{:#?}", jbody, body);
        return (StatusCode::ACCEPTED, Json(json!({"detail": "dev mode accepted"})));
    }

    let mut client = Client::new();
    if let Err(e) = client.login(&state.username, &state.password).await {
        tracing::error!("easycar login failed: {e:?}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"detail": "Login to InfoCar failed"})),
        );
    }

    let result = client.add_employee(state.employer_id, body).await;

    if let Err(e) = client.logout().await {
        tracing::error!("easycar logout failed: {e:?}");
    }

    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({"detail": "Employee submitted"}))),
        Err(GenericClientError::ValidationError(msg)) => {
            tracing::warn!("add_employee validation error: {msg}");
            (StatusCode::BAD_REQUEST, Json(json!({"detail": msg})))
        }
        Err(GenericClientError::ApiError { status, body }) => {
            tracing::warn!("InfoCar API error {status}: {body}");
            let code = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
            (code, Json(json!({"detail": body})))
        }
        Err(e) => {
            tracing::error!("add_employee failed: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"detail": "Failed to submit employee to InfoCar"})),
            )
        }
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/add-employee", post(create_employee))
        .with_state(state)
}

pub async fn run_server(
    addr: &str,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    terminate.await
}
