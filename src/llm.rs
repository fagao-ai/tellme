use reqwest::Client;
use serde_json::json;
use std::time::Instant;

use crate::dashboard;
use crate::utils;

pub async fn run(
    base_url: &str,
    api_key: Option<String>,
    check_tool_call: bool,
    check_reasoning: bool,
    model_override: Option<String>,
) {
    let client = utils::build_client();

    dashboard::banner("LLM");

    let (auto_model, server_ok) = utils::check_server(&client, base_url, api_key.as_deref()).await;
    let model = model_override.or(auto_model);

    dashboard::server_panel(base_url, model.as_deref(), server_ok);

    if !server_ok {
        return;
    }

    if !check_tool_call && !check_reasoning {
        dashboard::no_checks_hint();
        return;
    }

    dashboard::section_header("Feature Checks");

    if check_tool_call {
        check_tool_call_feature(&client, base_url, api_key.as_deref(), model.as_deref()).await;
    }

    if check_reasoning {
        check_reasoning_feature(&client, base_url, api_key.as_deref(), model.as_deref()).await;
    }
}

async fn check_tool_call_feature(client: &Client, base_url: &str, api_key: Option<&str>, model: Option<&str>) {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let model_name = model.unwrap_or("default");

    let tools = json!([{
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "获取指定城市的天气信息",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "城市名"
                    }
                },
                "required": ["location"]
            }
        }
    }]);

    let body = json!({
        "model": model_name,
        "messages": [{"role": "user", "content": "北京现在的天气怎么样?"}],
        "tools": tools,
        "tool_choice": "auto",
    });

    let mut req = client.post(&url).json(&body);
    if let Some(key) = api_key {
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
                        dashboard::feature_panel("Tool Call", false, Some(msg), None, None);
                        return;
                    }

                    let has_tool_calls = result["choices"][0]["message"]["tool_calls"]
                        .as_array()
                        .map(|c| !c.is_empty())
                        .unwrap_or(false);

                    let usage = result.get("usage");
                    let detail = if !has_tool_calls {
                        Some("模型未返回 tool_calls，可能未配置 tool-call-parser")
                    } else {
                        None
                    };

                    dashboard::feature_panel(
                        "Tool Call",
                        has_tool_calls,
                        detail,
                        usage.map(|u| (u, elapsed)),
                        None,
                    );
                }
                Err(e) => {
                    dashboard::feature_panel(
                        "Tool Call",
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
                "Tool Call",
                false,
                Some(&format!("请求失败: {}", e)),
                None,
                None,
            );
        }
    }
}

async fn check_reasoning_feature(client: &Client, base_url: &str, api_key: Option<&str>, model: Option<&str>) {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let model_name = model.unwrap_or("default");

    let body = json!({
        "model": model_name,
        "messages": [
            {"role": "user", "content": "What is 9.11 and 9.9 which is bigger? Let's think step by step."}
        ],
    });

    let mut req = client.post(&url).json(&body);
    if let Some(key) = api_key {
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
                        dashboard::feature_panel("Reasoning", false, Some(msg), None, None);
                        return;
                    }

                    let has_reasoning = result["choices"][0]["message"]["reasoning_content"]
                        .as_str()
                        .or_else(|| result["choices"][0]["message"]["reasoning"].as_str())
                        .is_some();

                    let usage = result.get("usage");
                    let detail = if !has_reasoning {
                        Some("响应中未发现 reasoning_content 或 reasoning 字段，可能模型不支持推理或未配置 --reasoning-parser")
                    } else {
                        None
                    };

                    dashboard::feature_panel(
                        "Reasoning",
                        has_reasoning,
                        detail,
                        usage.map(|u| (u, elapsed)),
                        None,
                    );
                }
                Err(e) => {
                    dashboard::feature_panel(
                        "Reasoning",
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
                "Reasoning",
                false,
                Some(&format!("请求失败: {}", e)),
                None,
                None,
            );
        }
    }
}
