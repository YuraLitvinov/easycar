use easycar_server::{AppState, run_server};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[forbid(unsafe_code)]
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let employer_id =
        std::env::var("EASYCAR_EMPLOYER_ID").expect("EASYCAR_EMPLOYER_ID must be set");
    let username = std::env::var("EASYCAR_USER").expect("EASYCAR_USER must be set");
    let password = std::env::var("EASYCAR_PASSWORD").expect("EASYCAR_PASSWORD must be set");
    let state = AppState {
        employer_id,
        username,
        password,
    };

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");

    if let Err(e) = run_server(&addr, state).await {
        tracing::error!("server error: {e}");
        std::process::exit(1);
    }
}
