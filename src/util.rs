use axum::body::Bytes;
use minio::s3::segmented_bytes::SegmentedBytes;

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
