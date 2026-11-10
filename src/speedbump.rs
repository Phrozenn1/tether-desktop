use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveWindowInfo {
    pub address: String,
    pub class: String,
    pub title: String,
    pub workspace_name: String,
}

#[derive(Debug)]
pub struct SpeedbumpState {
    pub last_trigger_instant: Option<Instant>,
    pub snoozed_address: Option<String>,
}

impl SpeedbumpState {
    pub fn new() -> Self {
        Self {
            last_trigger_instant: None,
            snoozed_address: None,
        }
    }

    pub fn should_trigger(
        &mut self,
        window: &ActiveWindowInfo,
        enabled: bool,
        cooldown_secs: u64,
        triggers: &[String],
    ) -> Option<String> {
        if !enabled {
            return None;
        }

        // Don't trigger on Tether itself
        let class_lower = window.class.to_lowercase();
        if class_lower.contains("tether") || class_lower == "org.tether.tether" {
            return None;
        }

        // If this window address is already snoozed by the user ("Dette er relevant")
        if let Some(snoozed) = &self.snoozed_address {
            if snoozed == &window.address {
                return None;
            }
        }

        let matched = check_speedbump_trigger(window, triggers)?;

        // Cooldown check
        let now = Instant::now();
        if let Some(last) = self.last_trigger_instant {
            let cooldown = Duration::from_secs(cooldown_secs.max(5));
            if now.duration_since(last) < cooldown {
                return None;
            }
        }

        self.last_trigger_instant = Some(now);
        Some(matched)
    }

    pub fn snooze(&mut self, address: &str) {
        self.snoozed_address = Some(address.to_string());
    }

    pub fn reset_snooze(&mut self) {
        self.snoozed_address = None;
    }
}

pub fn check_speedbump_trigger(
    window: &ActiveWindowInfo,
    triggers: &[String],
) -> Option<String> {
    let class_lower = window.class.to_lowercase();
    let title_lower = window.title.to_lowercase();

    for trigger in triggers {
        let t = trigger.trim().to_lowercase();
        if t.is_empty() {
            continue;
        }

        if class_lower.contains(&t) || title_lower.contains(&t) {
            return Some(trigger.clone());
        }
    }

    None
}

/// Query the currently focused window on the system.
/// Supports Hyprland IPC on Linux. Gracefully degrades to None on other platforms or compositors.
pub fn get_active_window() -> Option<ActiveWindowInfo> {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            if let Ok(out) = std::process::Command::new("hyprctl")
                .args(["activewindow", "-j"])
                .output()
            {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                    let address = val.get("address").and_then(|a| a.as_str()).unwrap_or("").to_string();
                    let class = val.get("class").and_then(|c| c.as_str()).unwrap_or("").to_string();
                    let title = val.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    let workspace_name = val
                        .get("workspace")
                        .and_then(|w| w.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();

                    if !address.is_empty() || !class.is_empty() || !title.is_empty() {
                        return Some(ActiveWindowInfo {
                            address,
                            class,
                            title,
                            workspace_name,
                        });
                    }
                }
            }
        }
    }

    None
}

/// Focus the previous editor or target window via compositor.
pub fn refocus_editor() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            // Check if there is an active editor client to focus
            if let Ok(output) = std::process::Command::new("hyprctl").args(["clients", "-j"]).output() {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    if let Some(arr) = val.as_array() {
                        for client in arr {
                            let class = client.get("class").and_then(|c| c.as_str()).unwrap_or("").to_lowercase();
                            let title = client.get("title").and_then(|t| t.as_str()).unwrap_or("").to_lowercase();
                            let address = client.get("address").and_then(|a| a.as_str()).unwrap_or("");

                            if class.contains("code")
                                || class.contains("vscodium")
                                || class.contains("cursor")
                                || class.contains("neovide")
                                || title.contains("nvim")
                                || title.contains("helix")
                            {
                                let _ = std::process::Command::new("hyprctl")
                                    .args(["dispatch", "focuswindow", &format!("address:{}", address)])
                                    .output();
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Run dedicated Speedbump dialog window (called via `tether --speedbump ...`).
/// Presents a focused, centered modal alert directly in the middle of the screen.
pub fn run_speedbump_dialog(args: &[String]) {
    use gtk4::prelude::*;

    let elapsed_mins = args.get(0).cloned().unwrap_or_else(|| "0".to_string());
    let title = args.get(1).cloned().unwrap_or_else(|| "".to_string());
    let trigger = args.get(2).cloned().unwrap_or_else(|| "".to_string());
    let _address = args.get(3).cloned().unwrap_or_else(|| "".to_string());
    let lang = args.get(4).map(|s| s.as_str()).unwrap_or("no");

    let is_no = lang == "no";

    let heading = if is_no { "Fartsdump" } else { "Speedbump" };

    let title_display = if title.trim().is_empty() {
        trigger.clone()
    } else {
        title.clone()
    };

    let body_text = if is_no {
        format!(
            "Du har vært i dyp flyt i {} minutter.\n\nOppdaget aktivt vindu:\n\"{}\" (matcher \"{}\")\n\nEr dette et kaninhull, eller er det relevant for oppgaven du jobber med?",
            elapsed_mins, title_display, trigger
        )
    } else {
        format!(
            "You have been in deep flow for {} minutes.\n\nDetected active window:\n\"{}\" (matches \"{}\")\n\nIs this a rabbit hole, or is it relevant to your current task?",
            elapsed_mins, title_display, trigger
        )
    };

    let btn_impulse = if is_no { "Parkér som impuls" } else { "Park as impulse" };
    let btn_relevant = if is_no { "Dette er relevant" } else { "This is relevant" };
    let btn_break = if is_no { "Ta pause nå" } else { "Take break now" };

    let app = libadwaita::Application::builder()
        .application_id("org.tether.Speedbump")
        .build();

    app.connect_activate(move |app| {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title(heading)
            .default_width(480)
            .default_height(240)
            .resizable(false)
            .modal(true)
            .build();

        let toolbar_view = libadwaita::ToolbarView::new();
        let header_bar = libadwaita::HeaderBar::new();
        header_bar.set_show_end_title_buttons(true);
        let title_widget = libadwaita::WindowTitle::new(
            heading,
            if is_no { "Avsporingsvern" } else { "Distraction Interceptor" },
        );
        header_bar.set_title_widget(Some(&title_widget));
        toolbar_view.add_top_bar(&header_bar);

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content_box.set_margin_top(16);
        content_box.set_margin_bottom(20);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);

        let label = gtk4::Label::builder()
            .label(&body_text)
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::WordChar)
            .justify(gtk4::Justification::Center)
            .xalign(0.5)
            .build();
        content_box.append(&label);

        let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        btn_box.set_halign(gtk4::Align::Center);
        btn_box.set_margin_top(8);

        let impulse_button = gtk4::Button::with_label(btn_impulse);
        impulse_button.add_css_class("suggested-action");
        impulse_button.add_css_class("pill");

        let relevant_button = gtk4::Button::with_label(btn_relevant);
        relevant_button.add_css_class("flat");

        let break_button = gtk4::Button::with_label(btn_break);
        break_button.add_css_class("flat");

        btn_box.append(&impulse_button);
        btn_box.append(&relevant_button);
        btn_box.append(&break_button);

        content_box.append(&btn_box);
        toolbar_view.set_content(Some(&content_box));
        window.set_child(Some(&toolbar_view));

        let app_clone1 = app.clone();
        impulse_button.connect_clicked(move |_| {
            println!("impulse");
            app_clone1.quit();
        });

        let app_clone2 = app.clone();
        relevant_button.connect_clicked(move |_| {
            println!("relevant");
            app_clone2.quit();
        });

        let app_clone3 = app.clone();
        break_button.connect_clicked(move |_| {
            println!("break");
            app_clone3.quit();
        });

        let key_controller = gtk4::EventControllerKey::new();
        let app_esc = app.clone();
        let app_enter = app.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                println!("relevant");
                app_esc.quit();
                gtk4::glib::Propagation::Stop
            } else if key == gtk4::gdk::Key::Return || key == gtk4::gdk::Key::KP_Enter {
                println!("impulse");
                app_enter.quit();
                gtk4::glib::Propagation::Stop
            } else {
                gtk4::glib::Propagation::Proceed
            }
        });
        window.add_controller(key_controller);

        window.connect_close_request(|_| {
            println!("relevant");
            gtk4::glib::Propagation::Proceed
        });

        window.present();
        impulse_button.grab_focus();

        #[cfg(target_os = "linux")]
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            let _ = std::process::Command::new("hyprctl")
                .args(["dispatch", "centerwindow"])
                .output();
            let _ = std::process::Command::new("hyprctl")
                .args(["dispatch", "focuswindow", "class:org.tether.Speedbump"])
                .output();
        }
    });

    let empty_args: [&str; 0] = [];
    app.run_with_args(&empty_args);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::TetherSettings;

    #[test]
    fn test_check_speedbump_trigger_matches_title() {
        let win = ActiveWindowInfo {
            address: "0x123".into(),
            class: "brave-browser".into(),
            title: "r/ADHD: Tips for staying focused - Reddit".into(),
            workspace_name: "1".into(),
        };
        let triggers = vec!["reddit".into(), "youtube".into()];
        assert_eq!(check_speedbump_trigger(&win, &triggers), Some("reddit".into()));
    }

    #[test]
    fn test_check_speedbump_trigger_matches_class() {
        let win = ActiveWindowInfo {
            address: "0x456".into(),
            class: "discord".into(),
            title: "#general".into(),
            workspace_name: "2".into(),
        };
        let triggers = vec!["discord".into(), "steam".into()];
        assert_eq!(check_speedbump_trigger(&win, &triggers), Some("discord".into()));
    }

    #[test]
    fn test_check_speedbump_trigger_ignores_unrelated() {
        let win = ActiveWindowInfo {
            address: "0x789".into(),
            class: "com.mitchellh.ghostty".into(),
            title: "nvim src/ui.rs".into(),
            workspace_name: "3".into(),
        };
        let triggers = vec!["reddit".into(), "youtube".into(), "discord".into()];
        assert_eq!(check_speedbump_trigger(&win, &triggers), None);
    }

    #[test]
    fn test_speedbump_cooldown_and_snooze() {
        let mut state = SpeedbumpState::new();
        let settings = TetherSettings {
            speedbump_enabled: true,
            speedbump_cooldown_secs: 10,
            speedbump_triggers: vec!["youtube".into()],
            ..Default::default()
        };

        let win = ActiveWindowInfo {
            address: "0xabc".into(),
            class: "firefox".into(),
            title: "YouTube - LoFi Girl".into(),
            workspace_name: "1".into(),
        };

        // First trigger should fire
        let res1 = state.should_trigger(&win, settings.speedbump_enabled, settings.speedbump_cooldown_secs, &settings.speedbump_triggers);
        assert_eq!(res1, Some("youtube".into()));

        // Immediate subsequent check should be throttled by cooldown
        let res2 = state.should_trigger(&win, settings.speedbump_enabled, settings.speedbump_cooldown_secs, &settings.speedbump_triggers);
        assert_eq!(res2, None);

        // Snooze window address
        state.snooze(&win.address);
        state.last_trigger_instant = None; // bypass cooldown to test snooze
        let res3 = state.should_trigger(&win, settings.speedbump_enabled, settings.speedbump_cooldown_secs, &settings.speedbump_triggers);
        assert_eq!(res3, None);

        // Another window should still trigger
        let win2 = ActiveWindowInfo {
            address: "0xdef".into(),
            class: "firefox".into(),
            title: "YouTube - Something else".into(),
            workspace_name: "1".into(),
        };
        let res4 = state.should_trigger(&win2, settings.speedbump_enabled, settings.speedbump_cooldown_secs, &settings.speedbump_triggers);
        assert_eq!(res4, Some("youtube".into()));
    }
}
