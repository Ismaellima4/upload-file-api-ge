mod client;
mod util;

use std::sync::Arc;

use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use minio::s3::{Client, types::S3Api};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use util::*;
static BUCKET_NAME: &str = "products";

#[derive(Clone)]
struct AppState {
    client: Arc<Client>,
}

async fn upload_image(State(app_state): State<AppState>, mut res: Multipart) -> impl IntoResponse {
    let client = &app_state.client;
    let mut file = File::new();
    while let Some(field) = res.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("");
        match field_name {
            "product_id" => file.file_name(field.text().await.unwrap()),
            "file" => {
                let name = field.file_name().clone().unwrap();
                let ext = std::path::Path::new(&name)
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap_or("");

                file.ext(ext.to_string());
                file.bytes(field.bytes().await.unwrap());
            }
            _ => {}
        }
    }
    if !file.file_name.is_empty() {
        return match client
            .put_object(BUCKET_NAME, file.file_name, file.bytes)
            .send()
            .await
        {
            Ok(_) => StatusCode::CREATED,
            Err(_err) => StatusCode::BAD_REQUEST,
        };
    }

    StatusCode::BAD_REQUEST
}

async fn find_image(
    Path(name): Path<String>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let client = &app_state.client;

    match client.get_object(BUCKET_NAME, &name).send().await {
        Ok(res) => {
            let bytes = res
                .content
                .to_segmented_bytes()
                .await
                .map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Erro ao ler bytes: {}", err),
                    )
                })
                .unwrap()
                .to_bytes();

            Ok((
                [(header::CONTENT_TYPE, determine_content_type(&name))],
                bytes,
            ))
        }
        Err(err) => Err((
            StatusCode::NOT_FOUND,
            format!("Imagem não encontrada: {}", err),
        )),
    }
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

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {} ", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
