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
pub mod updater;

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

fn format_waybar_payload(
    shared: &storage::SharedFlowState,
    anchor: Option<&storage::ContextAnchor>,
    open_impulses: usize,
    is_no: bool,
) -> serde_json::Value {
    let now_epoch = chrono::Utc::now().timestamp().max(0) as u64;
    let focus_target_secs = (shared.focus_ratio.max(1) as u64) * 60;

    let (text, alt, class_name, percentage, status_line) = match shared.state.as_str() {
        "focus" => {
            let elapsed = if shared.start_epoch > 0 {
                now_epoch.saturating_sub(shared.start_epoch)
            } else {
                shared.elapsed_focus_sec
            };
            let pct = ((elapsed * 100) / focus_target_secs).min(100);
            let time_str = flowmodoro::format_time(elapsed);
            let line = if is_no {
                format!("🔥 Fokus: {time_str}")
            } else {
                format!("🔥 Focus: {time_str}")
            };
            (format!("🔥 {time_str}"), "focus", "focus", pct, line)
        }
        "micro_start" => {
            let elapsed = if shared.start_epoch > 0 {
                now_epoch.saturating_sub(shared.start_epoch)
            } else {
                0
            };
            let rem = 300_u64.saturating_sub(elapsed);
            let pct = ((elapsed * 100) / 300).min(100);
            let time_str = flowmodoro::format_time(rem);
            let line = if is_no {
                format!("🌱 Mikrostart: {time_str} igjen")
            } else {
                format!("🌱 Micro-Start: {time_str} left")
            };
            (format!("🌱 {time_str}"), "micro-start", "micro-start", pct, line)
        }
        "break" => {
            let total = shared.break_duration_sec.max(shared.remaining_break_sec);
            let elapsed = if shared.start_epoch > 0 {
                now_epoch.saturating_sub(shared.start_epoch)
            } else {
                0
            };
            let rem = total.saturating_sub(elapsed);
            let pct = if total > 0 {
                ((rem * 100) / total).min(100)
            } else {
                0
            };
            let time_str = flowmodoro::format_time(rem);
            let line = if is_no {
                format!("☕ Opptjent pause: {time_str}")
            } else {
                format!("☕ Earned break: {time_str}")
            };
            (format!("☕ {time_str}"), "break", "break", pct, line)
        }
        _ => {
            let label = if is_no { "⚓ Klar" } else { "⚓ Ready" };
            let line = if is_no {
                "⚓ Flowmodoro: Klar til ny økt"
            } else {
                "⚓ Flowmodoro: Ready for session"
            };
            (label.to_string(), "idle", "idle", 0, line.to_string())
        }
    };

    let mut tooltip_lines = vec![status_line];
    if let Some(a) = anchor {
        if !a.project_name.is_empty() || !a.relative_path.is_empty() {
            let ctx = if a.relative_path.is_empty() {
                a.project_name.clone()
            } else {
                format!("{} — {}", a.project_name, a.relative_path)
            };
            tooltip_lines.push(if is_no {
                format!("⚓ Kontekst: {ctx}")
            } else {
                format!("⚓ Context: {ctx}")
            });
        }
    }
    tooltip_lines.push(if is_no {
        format!("⚡ Åpne impulser: {open_impulses}")
    } else {
        format!("⚡ Open impulses: {open_impulses}")
    });
    tooltip_lines.push(if is_no {
        "Venstreklikk: Start/Pause • Midtklikk: 5m Mikrostart • Høyreklikk: Hurtigfangst".to_string()
    } else {
        "Left-click: Toggle • Middle-click: 5m Micro-Start • Right-click: Quick-Capture".to_string()
    });

    serde_json::json!({
        "text": text,
        "alt": alt,
        "tooltip": tooltip_lines.join("\n"),
        "class": class_name,
        "percentage": percentage,
    })
}

fn build_waybar_json() -> String {
    let settings = storage::load_settings();
    let is_no = matches!(settings.language.resolve(), storage::Language::Norwegian);
    let shared = flowmodoro::apply_shared_flow_action("status", "waybar")
        .unwrap_or_else(|_| storage::SharedFlowState {
            state: "idle".to_string(),
            start_epoch: 0,
            break_duration_sec: 0,
            elapsed_focus_sec: 0,
            remaining_break_sec: 0,
            banked_break_sec: 0,
            focus_ratio: settings.ratio_focus.max(1),
            break_ratio: settings.ratio_break.max(1),
            writer: "waybar".to_string(),
            seq: 0,
            timestamp: 0,
        });
    let anchor = storage::load_context_anchor();
    let open_impulses = storage::load_structured_impulses_with_cleanup(&settings)
        .into_iter()
        .filter(|i| !i.done)
        .count();

    let payload = format_waybar_payload(&shared, anchor.as_ref(), open_impulses, is_no);
    serde_json::to_string(&payload)
        .unwrap_or_else(|_| r#"{"text":"⚓ Tether","alt":"idle","class":"idle","percentage":0}"#.into())
}

fn run_waybar() -> i32 {
    println!("{}", build_waybar_json());
    0
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
            "--waybar" => {
                let code = run_waybar();
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

    #[test]
    fn waybar_json_contains_expected_fields() {
        let shared = storage::SharedFlowState {
            state: "idle".to_string(),
            start_epoch: 0,
            break_duration_sec: 0,
            elapsed_focus_sec: 0,
            remaining_break_sec: 0,
            banked_break_sec: 0,
            focus_ratio: 45,
            break_ratio: 15,
            writer: "test".to_string(),
            seq: 1,
            timestamp: 0,
        };
        let v = format_waybar_payload(&shared, None, 2, false);
        assert_eq!(v["text"], "⚓ Ready");
        assert_eq!(v["alt"], "idle");
        assert_eq!(v["class"], "idle");
        assert_eq!(v["percentage"], 0);
        assert!(v["tooltip"].as_str().unwrap().contains("Open impulses: 2"));
    }
}
