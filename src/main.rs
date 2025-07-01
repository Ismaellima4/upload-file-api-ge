mod client;

use std::{sync::Arc, vec};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, State},
    response::IntoResponse,
    routing::post,
};
use minio::s3::{Client, segmented_bytes::SegmentedBytes, types::S3Api};
use tokio::net::TcpListener;

static BUCKET_NAME: &str = "products";

#[derive(Clone)]
struct AppState {
    client: Arc<Client>,
}

async fn upload_file(State(app_state): State<AppState>, mut file: Multipart) -> impl IntoResponse {
    let client = &app_state.client;
    let mut res = vec![];
    while let Some(field) = file.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let mut segmented_bytes = SegmentedBytes::new();
        segmented_bytes.append(data);

        client
            .put_object(BUCKET_NAME, name.clone(), segmented_bytes)
            .send()
            .await
            .unwrap();

        res.push(name);
    }
    Json(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = client::init_minio_client()?;
    client::create_bucket_if_not_exists(BUCKET_NAME, &client)
        .await
        .unwrap();
    let state = AppState {
        client: Arc::new(client),
    };

    let app: Router = Router::new()
        .route("/upload", post(upload_file))
        .with_state(state)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {} ", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
