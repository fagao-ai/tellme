use base64::Engine;
use serde_json::json;
use std::time::Instant;

use crate::dashboard;
use crate::utils;

fn test_image_base64() -> String {
    let bytes = include_bytes!("../assets/demo.png");
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub async fn run(base_url: &str, api_key: Option<String>, model_override: Option<String>) {
    let client = utils::build_client();

    dashboard::banner("VLM");

    let (auto_model, server_ok) = utils::check_server(&client, base_url, api_key.as_deref()).await;
    let model = model_override.or(auto_model);

    dashboard::server_panel(base_url, model.as_deref(), server_ok);

    if !server_ok {
        return;
    }

    dashboard::section_header("Feature Checks");

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let model_name = model.as_deref().unwrap_or("default");

    let body = json!({
        "model": model_name,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "What do you see in this image? Reply in one short sentence."},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", test_image_base64())
                        }
                    }
                ]
            }
        ],
        "max_tokens": 128,
    });

    let mut req = client.post(&url).json(&body);
    if let Some(key) = api_key.as_deref() {
        req = req.bearer_auth(key);
    }
    let start = Instant::now();
    match req.send().await {
        Ok(resp) => {
            let elapsed = start.elapsed();
            match resp.json::<serde_json::Value>().await {
                Ok(result) => {
                    if let Some(error) = result.get("error") {
                        let msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("未知错误");
                        dashboard::feature_panel("Vision", false, Some(msg), None, None);
                        return;
                    }

                    let content = result["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("");

                    let has_vision = !content.is_empty();
                    let usage = result.get("usage");
                    let detail = if !has_vision {
                        Some("模型返回了空响应，可能不支持视觉输入")
                    } else {
                        None
                    };

                    dashboard::feature_panel(
                        "Vision",
                        has_vision,
                        detail,
                        usage.map(|u| (u, elapsed)),
                        if has_vision { Some(content) } else { None },
                    );
                }
                Err(e) => {
                    dashboard::feature_panel(
                        "Vision",
                        false,
                        Some(&format!("响应解析失败: {}", e)),
                        None,
                        None,
                    );
                }
            }
        }
        Err(e) => {
            dashboard::feature_panel(
                "Vision",
                false,
                Some(&format!("请求失败: {}", e)),
                None,
                None,
            );
        }
    }
}
