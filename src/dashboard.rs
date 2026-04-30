use colored::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use serde_json::Value;
use std::time::Duration;

/// 顶部横幅
pub fn banner(subtitle: &str) {
    let text = format!(" tellme · {} ", subtitle);
    let width = text.chars().count();
    let bar = "─".repeat(width);
    println!();
    println!("{}", format!("╭{}╮", bar).bright_black());
    println!("{}", text.bold().white());
    println!("{}", format!("╰{}╯", bar).bright_black());
    println!();
}

/// 服务器状态面板
pub fn server_panel(base_url: &str, model: Option<&str>, ok: bool) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![Cell::new(" Server ").add_attribute(Attribute::Bold)]);

    table.add_row(vec![
        Cell::new("Address").add_attribute(Attribute::Bold),
        Cell::new(base_url),
    ]);

    if let Some(m) = model {
        table.add_row(vec![
            Cell::new("Model").add_attribute(Attribute::Bold),
            Cell::new(m),
        ]);
    }

    let status_cell = if ok {
        Cell::new("✓ Connected")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold)
    } else {
        Cell::new("✗ Connection Failed")
            .fg(Color::Red)
            .add_attribute(Attribute::Bold)
    };
    table.add_row(vec![
        Cell::new("Status").add_attribute(Attribute::Bold),
        status_cell,
    ]);

    println!("{}\n", table);
}

/// 区段标题
pub fn section_header(title: &str) {
    println!("{}", format!("▸ {}", title).bold().cyan());
    println!();
}

/// 功能检查结果面板
pub fn feature_panel(
    name: &str,
    ok: bool,
    detail: Option<&str>,
    usage: Option<(&Value, Duration)>,
    model_reply: Option<&str>,
) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![Cell::new(format!(" {} ", name))
            .add_attribute(Attribute::Bold)]);

    let status_cell = if ok {
        Cell::new("✓ Enabled")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold)
    } else {
        Cell::new("✗ Not Detected")
            .fg(Color::Red)
            .add_attribute(Attribute::Bold)
    };
    table.add_row(vec![
        Cell::new("Status").add_attribute(Attribute::Bold),
        status_cell,
    ]);

    if let Some(reply) = model_reply {
        table.add_row(vec![
            Cell::new("Reply").add_attribute(Attribute::Bold),
            Cell::new(reply),
        ]);
    }

    if let Some(d) = detail {
        let detail_cell = Cell::new(d).fg(Color::Yellow);
        table.add_row(vec![
            Cell::new("Hint").add_attribute(Attribute::Bold),
            detail_cell,
        ]);
    }

    if let Some((usage_data, elapsed)) = usage {
        let completion_tokens = usage_data
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let prompt_tokens = usage_data
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total_tokens = usage_data
            .get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let elapsed_secs = elapsed.as_secs_f64();
        let tok_s = if elapsed_secs > 0.0 {
            completion_tokens as f64 / elapsed_secs
        } else {
            0.0
        };

        table.add_row(vec![
            Cell::new("Latency").add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2}s", elapsed_secs)),
        ]);
        table.add_row(vec![
            Cell::new("Throughput").add_attribute(Attribute::Bold),
            Cell::new(format!("{:.1} tok/s", tok_s))
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
        ]);
        table.add_row(vec![
            Cell::new("Tokens").add_attribute(Attribute::Bold),
            Cell::new(format!(
                "Prompt: {}  Completion: {}  Total: {}",
                prompt_tokens, completion_tokens, total_tokens
            )),
        ]);
    }

    println!("{}\n", table);
}

/// 未指定检查项的提示
pub fn no_checks_hint() {
    println!(
        "{}",
        "▸ Use --tool-call and/or --reasoning to specify checks.\n"
            .yellow()
    );
}
