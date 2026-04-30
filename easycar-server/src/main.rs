use easycar_server::{build_state_from_env, run_server};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[forbid(unsafe_code)]
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let state = match build_state_from_env().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("startup failed: {e}");
            std::process::exit(1);
        }
    };

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");

    if let Err(e) = run_server(&addr, state).await {
        tracing::error!("server error: {e}");
        std::process::exit(1);
    }
}
