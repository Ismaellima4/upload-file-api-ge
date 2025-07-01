use std::{env, error::Error};

use minio::s3::{
    Client, ClientBuilder, creds::StaticProvider, http::BaseUrl, response::BucketExistsResponse,
    types::S3Api,
};

#[allow(dead_code)]
pub fn init_minio_client() -> Result<Client, Box<dyn Error + Send + Sync>> {
    dotenv::dotenv().ok();

    let endpoint = env::var("MINIO_ENDPOINT")?.parse::<BaseUrl>()?;
    let access_key = env::var("MINIO_ACCESS_KEY")?;
    let secret_key = env::var("MINIO_SECRET_KEY")?;

    let static_provider = StaticProvider::new(access_key.as_str(), secret_key.as_str(), None);
    let minio = ClientBuilder::new(endpoint)
        .provider(Some(Box::new(static_provider)))
        .build()?;

    Ok(minio)
}

#[allow(dead_code)]
pub async fn create_bucket_if_not_exists(
    bucket_name: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check 'bucket_name' bucket exist or not.
    let resp: BucketExistsResponse = client.bucket_exists(bucket_name).send().await?;

    // Make 'bucket_name' bucket if not exist.
    if !resp.exists {
        client.create_bucket(bucket_name).send().await?;
    };
    Ok(())
}
