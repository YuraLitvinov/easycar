use easycar_server::{build_state_from_env, run_server};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[forbid(unsafe_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    dotenvy::dotenv().ok();
    let state = build_state_from_env().await?;

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    run_server(&addr, state).await?;

    Ok(())
}
