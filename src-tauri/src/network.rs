use mime_guess::get_mime_extensions_str;
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Client,
};
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

fn get_extension_from_url_path(url: &Url) -> Option<String> {
    let filename = url.path().rsplit('/').next()?;
    let ext = filename.rsplit('.').next()?;
    if ext == filename {
        None
    } else {
        Some(ext.to_lowercase())
    }
}

fn get_extension_from_mime(mime: &str) -> Option<String> {
    get_mime_extensions_str(mime)
        .and_then(|exts| exts.first())
        .map(|ext| ext.to_string())
}

async fn fetch_extension_from_mime(client: &Client, url: &Url) -> Option<String> {
    let response = client.head(url.as_str()).send().await.ok()?;
    let content_type = response.headers().get(CONTENT_TYPE)?.to_str().ok()?;
    let mime = content_type.split(';').next()?.trim();
    get_extension_from_mime(mime)
}

pub async fn get_best_extension(client: &Client, url: &Url) -> Option<String> {
    if let Some(ext) = get_extension_from_url_path(url) {
        return Some(ext);
    }
    fetch_extension_from_mime(client, url).await
}
