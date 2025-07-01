const ALLOWED_EXT: [&str; 3] = [".jpeg", ".jpg", ".png"];

pub fn is_image_ext(ext: &str) -> bool {
    ALLOWED_EXT.contains(&ext)
}

pub fn generated_unique_file_name(ext: &str) -> String {
    format!("{}.{}", uuid::Uuid::new_v4(), ext)
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
