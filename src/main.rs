mod client;
mod util;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use minio::s3::{Client, types::S3Api};
use serde_json::json;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use util::*;
static BUCKET_NAME: &str = "products";

#[derive(Clone)]
struct AppState {
    client: Arc<Client>,
}

async fn upload_image(
    State(app_state): State<AppState>,
    mut res: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let client = &app_state.client;
    let mut file = File::new();

    while let Some(field_result) = res.next_field().await.transpose() {
        let field = match field_result {
            Ok(f) => f,
            Err(err) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "message": format!("Erro no multipart: {}", err) })),
                ));
            }
        };

        let field_name = field.name().unwrap_or("");
        match field_name {
            "product_id" => {
                let unique_file_name_with_product_id_and_ext = match field.text().await {
                    Ok(text) => text,
                    Err(err) => {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "message": err.to_string() })),
                        ));
                    }
                };

                let parts: Vec<&str> = unique_file_name_with_product_id_and_ext
                    .split('.')
                    .collect();
                let ext = parts.last().unwrap_or(&"");
                let file_name = parts[0];

                file.file_name(file_name.to_string());
                file.ext(format!(".{ext}"));
            }
            "file" => {
                match field.bytes().await {
                    Ok(bytes) => file.bytes(bytes),
                    Err(err) => {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "message": err.to_string() })),
                        ));
                    }
                };
            }
            _ => {}
        }
    }

    if !file.file_name.is_empty() && !file.ext.is_empty() {
        let file_name_with_ext = file.file_name.clone() + &file.ext;
        return match client
            .put_object(BUCKET_NAME, file_name_with_ext.clone(), file.bytes)
            .send()
            .await
        {
            Ok(_) => Ok((
                StatusCode::CREATED,
                Json(json!({ "message": file_name_with_ext })),
            )),
            Err(err) => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "message": err.to_string() })),
            )),
        };
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({
            "message": "É necessario o id do product + o ext da imagem. (ex: ce8fa930-fe2d-4ad1-9966-8eb3c3826444.jpeg)"
        })),
    ))
}

async fn find_image(
    Path(name): Path<String>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
            Json(json!({ "message": err.to_string() })),
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

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {} ", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
