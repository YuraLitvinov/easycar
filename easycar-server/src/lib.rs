use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use info_car_api::{client::Client, types::AddEmployeeRequest};

#[derive(Clone)]
pub struct AppState {
    pub employer_id: String,
    pub username: String,
    pub password: String,
}

async fn create_employee(
    State(state): State<AppState>,
    Json(body): Json<AddEmployeeRequest>,
) -> StatusCode {
    let mut client = Client::new();
    if let Err(e) = client.login(&state.username, &state.password).await {
        tracing::error!("easycar login failed: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let result = client.add_employee(state.employer_id, body).await;
    tracing::info!("Add employee result: {:?}", result);

    if let Err(e) = client.logout().await {
        tracing::error!("easycar logout failed: {e:?}");
    }

    StatusCode::CREATED
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
    axum::serve(listener, app).await?;
    Ok(())
}
