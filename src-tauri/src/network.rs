use reqwest::{header::CONTENT_LENGTH, Client};
use url::Url;

pub async fn fetch_total_bytes(client: &Client, url: &Url) -> Option<u64> {
    let response = client.head(url.as_str()).send().await.ok()?;
    response
        .headers()
        .get(CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

pub fn get_extension_from_url_path(url: &Url) -> Option<String> {
    let filename = url.path().rsplit('/').next()?;
    let ext = filename.rsplit('.').next()?;
    if ext == filename {
        None
    } else {
        Some(ext.to_lowercase())
    }
}
