use clap::Parser;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "tellme", version, about = "验证 OpenAI 兼容模型部署配置")]
enum Cli {
    /// 检查部署配置
    Check {
        /// API 基础 URL，如 http://localhost:8008/v1
        #[arg(long)]
        base_url: String,

        /// 检查 Tool Call 功能
        #[arg(long)]
        tool_call: bool,

        /// 检查 Reasoning 功能
        #[arg(long)]
        reasoning: bool,

        /// 指定模型名称（可选，默认使用服务返回的第一个模型）
        #[arg(long)]
        model: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::Check {
            base_url,
            tool_call,
            reasoning,
            model,
        } => run_check(&base_url, tool_call, reasoning, model).await,
    }
}

async fn run_check(
    base_url: &str,
    check_tool_call: bool,
    check_reasoning: bool,
    model_override: Option<String>,
) {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    println!("# 检查报告\n");

    let (auto_model, server_ok) = check_server(&client, base_url).await;
    let model = model_override.or(auto_model);

    // 服务器状态
    println!("## 服务器状态");
    println!("- **地址**: {}", base_url);
    if server_ok {
        if let Some(ref m) = model {
            println!("- **模型**: {}", m);
        }
        println!("- **响应**: ✓ 正常\n");
    } else {
        println!("- **响应**: ✗ 连接失败\n");
        return;
    }

    if !check_tool_call && !check_reasoning {
        println!("> 未指定检查项。使用 `--tool-call` 和/或 `--reasoning` 指定要检查的功能。\n");
        return;
    }

    println!("## 功能检查\n");

    if check_tool_call {
        check_tool_call_feature(&client, base_url, model.as_deref()).await;
    }

    if check_reasoning {
        check_reasoning_feature(&client, base_url, model.as_deref()).await;
    }
}

/// 检查服务器连通性并获取模型列表
async fn check_server(client: &Client, base_url: &str) -> (Option<String>, bool) {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    match client.get(&url).send().await {
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

/// 检查 Tool Call 功能
async fn check_tool_call_feature(client: &Client, base_url: &str, model: Option<&str>) {
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

    match client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(result) => {
                // 检查服务端返回的错误
                if let Some(error) = result.get("error") {
                    let msg = error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("未知错误");
                    println!("- **Tool Call**: ✗ 未检测到");
                    println!("  - **原因**: {}\n", msg);
                    return;
                }

                // 检查是否包含 tool_calls
                let has_tool_calls = result["choices"][0]["message"]["tool_calls"]
                    .as_array()
                    .map(|c| !c.is_empty())
                    .unwrap_or(false);

                if has_tool_calls {
                    println!("- **Tool Call**: ✓ 已启用\n");
                } else {
                    println!("- **Tool Call**: ✗ 未检测到");
                    println!(
                        "  - **提示**: 模型未返回 tool_calls，可能未配置 tool-call-parser\n"
                    );
                }
            }
            Err(e) => {
                println!("- **Tool Call**: ✗ 响应解析失败 ({})\n", e);
            }
        },
        Err(e) => {
            println!("- **Tool Call**: ✗ 请求失败 ({})\n", e);
        }
    }
}

/// 检查 Reasoning 功能
async fn check_reasoning_feature(client: &Client, base_url: &str, model: Option<&str>) {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let model_name = model.unwrap_or("default");

    let body = json!({
        "model": model_name,
        "messages": [
            {"role": "user", "content": "What is 9.11 and 9.9 which is bigger? Let's think step by step."}
        ],
    });

    match client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(result) => {
                if let Some(error) = result.get("error") {
                    let msg = error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("未知错误");
                    println!("- **Reasoning**: ✗ 未检测到");
                    println!("  - **原因**: {}\n", msg);
                    return;
                }

                let reasoning = result["choices"][0]["message"]["reasoning_content"]
                    .as_str()
                    .or_else(|| result["choices"][0]["message"]["reasoning"].as_str());

                if reasoning.is_some() {
                    println!("- **Reasoning**: ✓ 已启用\n");
                } else {
                    println!("- **Reasoning**: ✗ 未检测到");
                    println!(
                        "  - **提示**: 响应中未发现 reasoning_content 或 reasoning 字段，可能模型不支持推理或未配置 --reasoning-parser\n"
                    );
                }
            }
            Err(e) => {
                println!("- **Reasoning**: ✗ 响应解析失败 ({})\n", e);
            }
        },
        Err(e) => {
            println!("- **Reasoning**: ✗ 请求失败 ({})\n", e);
        }
    }
}
