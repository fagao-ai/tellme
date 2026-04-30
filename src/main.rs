mod dashboard;
mod llm;
mod utils;
mod vlm;

use clap::Parser;

#[derive(Parser)]
#[command(name = "tellme", version, about = "验证 OpenAI 兼容模型部署配置")]
enum Cli {
    /// 检查大语言模型（LLM）部署配置
    Llm {
        /// API 基础 URL，如 http://localhost:8008/v1
        #[arg(long)]
        base_url: String,

        /// API Key（可选，不需要鉴权的服务可省略）
        #[arg(long)]
        api_key: Option<String>,

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

    /// 检查视觉语言模型（VLM）部署配置
    Vlm {
        /// API 基础 URL，如 http://localhost:8008/v1
        #[arg(long)]
        base_url: String,

        /// API Key（可选，不需要鉴权的服务可省略）
        #[arg(long)]
        api_key: Option<String>,

        /// 指定模型名称（可选，默认使用服务返回的第一个模型）
        #[arg(long)]
        model: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::Llm {
            base_url,
            api_key,
            tool_call,
            reasoning,
            model,
        } => llm::run(&base_url, api_key, tool_call, reasoning, model).await,

        Cli::Vlm {
            base_url,
            api_key,
            model,
        } => vlm::run(&base_url, api_key, model).await,
    }
}
