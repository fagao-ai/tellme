use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub fn build_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap()
}

pub async fn check_server(client: &Client, base_url: &str, api_key: Option<&str>) -> (Option<String>, bool) {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let mut req = client.get(&url);
    if let Some(key) = api_key {
        req = req.bearer_auth(key);
    }
    match req.send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<Value>().await {
                    if let Some(models) = body.get("data").and_then(|d| d.as_array()) {
                        if let Some(first) = models.first() {
                            let id = first
                                .get("id")
                                .and_then(|id| id.as_str())
                                .map(|s| s.to_string());
                            return (id, true);
                        }
                    }
                }
                (None, true)
            } else {
                (None, false)
            }
        }
        Err(_) => (None, false),
    }
}
