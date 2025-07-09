use std::{env::var, sync::LazyLock};
use tracing::{error, info};

#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub minio_url: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
}

pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| AppConfig {
    jwt_secret: load_secret("JWT_SECRET"),
    minio_url: load_secret("MINIO_ENDPOINT"),
    minio_access_key: load_secret("MINIO_ACCESS_KEY"),
    minio_secret_key: load_secret("MINIO_SECRET_KEY"),
});

fn load_secret(secret_name: &str) -> String {
    match var(secret_name) {
        Ok(s) => {
            info!("Success load env: {}", secret_name);
            s
        }
        Err(e) => {
            error!("Error ao obter {}: {}", secret_name, e);
            std::process::exit(1)
        }
    }
}

