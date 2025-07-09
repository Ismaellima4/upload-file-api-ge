mod client;
mod handlers;
mod util;
mod config;

use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use handlers::*;
use minio::s3::Client;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    client: Arc<Client>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let client = client::init_minio_client()?;
    client::create_bucket_if_not_exists(BUCKET_NAME, &client)
        .await
        .unwrap();

    let state = AppState {
        client: Arc::new(client),
    };

    let app: Router = Router::new()
        .route("/upload", post(upload_image))
        .route("/image/{name}", get(find_image))
        .with_state(state)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {} ", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
