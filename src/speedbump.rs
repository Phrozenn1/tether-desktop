use std::time::{Duration, Instant};
use crate::storage::TetherSettings;

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
        settings: &TetherSettings,
    ) -> Option<String> {
        if !settings.speedbump_enabled {
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

        let matched = check_speedbump_trigger(window, &settings.speedbump_triggers)?;

        // Cooldown check
        let now = Instant::now();
        if let Some(last) = self.last_trigger_instant {
            let cooldown = Duration::from_secs(settings.speedbump_cooldown_secs.max(5));
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let res1 = state.should_trigger(&win, &settings);
        assert_eq!(res1, Some("youtube".into()));

        // Immediate subsequent check should be throttled by cooldown
        let res2 = state.should_trigger(&win, &settings);
        assert_eq!(res2, None);

        // Snooze window address
        state.snooze(&win.address);
        state.last_trigger_instant = None; // bypass cooldown to test snooze
        let res3 = state.should_trigger(&win, &settings);
        assert_eq!(res3, None);

        // Another window should still trigger
        let win2 = ActiveWindowInfo {
            address: "0xdef".into(),
            class: "firefox".into(),
            title: "YouTube - Something else".into(),
            workspace_name: "1".into(),
        };
        let res4 = state.should_trigger(&win2, &settings);
        assert_eq!(res4, Some("youtube".into()));
    }
}
