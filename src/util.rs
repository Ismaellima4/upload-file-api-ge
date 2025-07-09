use axum::{
    Json, RequestPartsExt,
    body::Bytes,
    extract::FromRequestParts,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use minio::s3::segmented_bytes::SegmentedBytes;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::CONFIG;

#[allow(dead_code)]
const ALLOWED_EXT: [&str; 3] = [".jpeg", ".jpg", ".png"];

#[allow(dead_code)]
pub fn is_image_ext(ext: &str) -> bool {
    ALLOWED_EXT.contains(&ext)
}

pub fn determine_content_type(filename: &str) -> &'static str {
    match filename
        .split('.')
        .last()
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream", // tipo genérico se não reconhecer
    }
}
pub struct File {
    pub file_name: String,
    pub bytes: SegmentedBytes,
    pub ext: String,
}

impl File {
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            bytes: SegmentedBytes::new(),
            ext: String::new(),
        }
    }

    pub fn file_name(&mut self, file_name: String) {
        self.file_name = file_name
    }
    pub fn bytes(&mut self, bytes: Bytes) {
        self.bytes.append(bytes);
    }
    pub fn ext(&mut self, ext: String) {
        self.ext = ext;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "admin-stock")]
    AdminStock,
    #[serde(rename = "admin-cashier")]
    AdminCashier,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_err| AuthError::InvalidToken)?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(CONFIG.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
