#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod audio;
mod context_engine;
mod editor;
mod flowmodoro;
mod i18n;
mod impulse;
mod ramdump;
mod storage;
mod theme;
mod time_anchor;
mod tray;
mod ui;
pub mod speedbump;
pub mod quick_capture;
pub mod dopamine;

use libadwaita::prelude::*;
use libadwaita::Application;
use ui::TetherUi;

fn build_cli_request_json(cli_args: &[String]) -> Result<String, String> {
    let first = cli_args
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Usage: tether --cli <operation> [json_payload] | tether --cli '<json_request>'".to_string())?;

    if first.starts_with('{') {
        return Ok(first.to_string());
    }

    let payload = match cli_args.get(1).map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(raw) => serde_json::from_str::<serde_json::Value>(raw)
            .map_err(|e| format!("Invalid JSON payload: {e}"))?,
        None => serde_json::Value::Object(serde_json::Map::new()),
    };

    serde_json::to_string(&serde_json::json!({
        "operation": first,
        "payload": payload,
    }))
    .map_err(|e| e.to_string())
}

fn run_cli(cli_args: &[String]) -> i32 {
    let request_json = match build_cli_request_json(cli_args) {
        Ok(json) => json,
        Err(err) => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": false,
                    "error": err,
                })
            );
            return 1;
        }
    };

    let response = tether_core::ffi::handle_json_request(&request_json);
    println!("{response}");

    let is_ok = serde_json::from_str::<serde_json::Value>(&response)
        .ok()
        .and_then(|v| v.get("ok").and_then(|b| b.as_bool()))
        .unwrap_or(false);
    if is_ok { 0 } else { 1 }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--speedbump" => {
                speedbump::run_speedbump_dialog(&args[2..]);
                return;
            }
            "--quick-capture" | "--quick" | "--impulse" | "--ramdump" => {
                quick_capture::run_quick_capture_dialog(&args[1..]);
                return;
            }
            "--cli" => {
                let code = run_cli(&args[2..]);
                std::process::exit(code);
            }
            _ => {}
        }
    }

    let open_shutdown = args.iter().any(|a| a == "--shutdown");
    if open_shutdown {
        storage::request_shutdown_ritual();
    }

    let app = Application::builder()
        .application_id("org.tether.Tether")
        .build();

    app.connect_activate(|app| {
        let _ui = TetherUi::build(app);
    });

    if open_shutdown {
        app.run_with_args::<&str>(&[]);
    } else {
        app.run();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_builds_raw_json_or_operation_payload() {
        let raw = vec![r#"{"operation":"health"}"#.to_string()];
        assert_eq!(build_cli_request_json(&raw).unwrap(), raw[0]);

        let op_only = vec!["flow.status".to_string()];
        let v: serde_json::Value =
            serde_json::from_str(&build_cli_request_json(&op_only).unwrap()).unwrap();
        assert_eq!(v["operation"], "flow.status");
        assert_eq!(v["payload"], serde_json::json!({}));

        let op_payload = vec![
            "impulses.add".to_string(),
            r#"{"text":"Test CLI impulse"}"#.to_string(),
        ];
        let v2: serde_json::Value =
            serde_json::from_str(&build_cli_request_json(&op_payload).unwrap()).unwrap();
        assert_eq!(v2["operation"], "impulses.add");
        assert_eq!(v2["payload"]["text"], "Test CLI impulse");
    }
}
