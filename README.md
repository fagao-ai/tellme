# tellme

[![Release](https://github.com/fagao-ai/tellme/actions/workflows/release.yml/badge.svg)](https://github.com/fagao-ai/tellme/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**tellme** is a CLI tool that probes OpenAI-compatible model serving endpoints (vLLM, TGI, SGLang, etc.) to verify whether key features like **tool-calling**, **reasoning**, and **vision** are properly configured.

Forgetting `--tool-call-parser` or `--reasoning-parser` when starting a vLLM server? tellme catches it in seconds.

## Features

- **LLM check** (`tellme llm`) — tool-call and reasoning capability probing
- **VLM check** (`tellme vlm`) — vision/image-input capability probing
- **Tool-call check** — sends a request with `tools` parameter and checks if the model responds with `tool_calls`
- **Reasoning check** — sends a reasoning prompt and checks for `reasoning_content` / `reasoning` fields in the response
- **Vision check** — sends an image in OpenAI vision format and verifies the model responds with text
- **Human-readable output** — Markdown with ✓ / ✗ indicators
- **Performance metrics** — tok/s, token counts, and latency for each request
- **Works with any OpenAI-compatible API** — vLLM, TGI, SGLang, etc.
- **Optional API key** — supports `--api-key` for authenticated services, omit for open services

## Installation

### Install script (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/fagao-ai/tellme/main/scripts/install.sh | sudo sh
```

### Install script (Windows PowerShell)

```powershell
iex "& { $(Invoke-RestContent https://raw.githubusercontent.com/fagao-ai/tellme/main/scripts/install.ps1) }"
```

### From source

```bash
cargo install tellme
```

### Build from source

```bash
git clone https://github.com/fagao-ai/tellme.git
cd tellme
cargo build --release
./target/release/tellme --help
```

## Usage

```bash
# Check tool-call support
tellme llm --base-url http://localhost:8008/v1 --tool-call

# Check reasoning support
tellme llm --base-url http://localhost:8008/v1 --reasoning

# Check both
tellme llm --base-url http://localhost:8008/v1 --tool-call --reasoning

# Specify a model name explicitly
tellme llm --base-url http://localhost:8008/v1 --tool-call --model Qwen3.6-27B

# With API key for authenticated services
tellme llm --base-url https://api.openai.com/v1 --api-key sk-xxx --tool-call

# Check VLM (vision) support
tellme vlm --base-url http://localhost:8008/v1

# Check VLM with specific model
tellme vlm --base-url http://localhost:8008/v1 --model Qwen2.5-VL-7B
```

## Output

```markdown
# 检查报告

## 服务器状态
- **地址**: http://localhost:8008/v1
- **模型**: Qwen3.6-27B
- **响应**: ✓ 正常

## 功能检查
- **Tool Call**:  ✓ 已启用
- **Reasoning**:  ✗ 未检测到
  - **提示**: 响应中未发现 reasoning_content 或 reasoning 字段，可能模型不支持推理或未配置 --reasoning-parser
```

## How it works

| Check | Method | Pass criteria |
|-------|--------|---------------|
| Health | `GET /v1/models` | Server responds with model list |
| Tool-call | `POST /chat/completions` with `tools` | Response contains `tool_calls` |
| Reasoning | `POST /chat/completions` with reasoning prompt | Response contains `reasoning_content` or `reasoning` |
| Vision | `POST /chat/completions` with `image_url` content | Response contains non-empty text content |

## License

MIT
