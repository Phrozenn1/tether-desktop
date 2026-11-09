use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct OmarchyColors {
    pub accent: String,
    pub background: String,
    pub lighter_background: String,
    pub foreground: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub is_dark: bool,
}

impl Default for OmarchyColors {
    fn default() -> Self {
        Self {
            accent: "#7d82d9".to_string(),
            background: "#1e1e2e".to_string(),
            lighter_background: "#28283d".to_string(),
            foreground: "#cdd6f4".to_string(),
            red: "#f38ba8".to_string(),
            green: "#a6e3a1".to_string(),
            yellow: "#f9e2af".to_string(),
            is_dark: true,
        }
    }
}

pub fn colors_file_path() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".local/state/omarchy/current/theme/colors.toml");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_colors_mtime() -> Option<SystemTime> {
    let path = colors_file_path()?;
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

pub fn load_omarchy_colors() -> OmarchyColors {
    let mut colors = OmarchyColors::default();
    let path = match colors_file_path() {
        Some(p) => p,
        None => return colors,
    };

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return colors,
    };

    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'');
            map.insert(key.to_string(), val.to_string());
        }
    }

    if let Some(accent) = map.get("accent") {
        colors.accent = accent.clone();
    }
    if let Some(bg) = map.get("background") {
        colors.background = bg.clone();
    }
    if let Some(lbg) = map.get("lighter_background") {
        colors.lighter_background = lbg.clone();
    } else if let Some(bg) = map.get("background") {
        colors.lighter_background = bg.clone();
    }
    if let Some(fg) = map.get("foreground") {
        colors.foreground = fg.clone();
    }
    if let Some(red) = map.get("red") {
        colors.red = red.clone();
    }
    if let Some(green) = map.get("green") {
        colors.green = green.clone();
    }
    if let Some(yellow) = map.get("yellow") {
        colors.yellow = yellow.clone();
    }
    if let Some(mode) = map.get("mode") {
        colors.is_dark = mode != "light";
    }

    colors
}

pub fn generate_css(colors: &OmarchyColors) -> String {
    format!(
        r#"
        @define-color accent_color {accent};
        @define-color accent_bg_color {accent};
        @define-color window_bg_color {bg};
        @define-color window_fg_color {fg};
        @define-color view_bg_color {lbg};
        @define-color card_bg_color alpha({lbg}, 0.7);
        @define-color headerbar_bg_color {bg};
        @define-color headerbar_fg_color {fg};
        @define-color popover_bg_color {lbg};
        @define-color popover_fg_color {fg};

        .timer-display {{
            font-size: 48px;
            font-weight: 800;
            font-family: 'JetBrains Mono', 'Fira Code', monospace;
            letter-spacing: 2px;
            margin: 2px 0;
        }}
        .badge-micro {{
            background-color: alpha({yellow}, 0.22);
            color: {yellow};
            border-radius: 9999px;
            padding: 4px 14px;
            font-weight: bold;
            font-size: 12px;
        }}
        .badge-focus {{
            background-color: alpha({green}, 0.22);
            color: {green};
            border-radius: 9999px;
            padding: 4px 14px;
            font-weight: bold;
            font-size: 12px;
        }}
        .badge-break {{
            background-color: alpha({red}, 0.22);
            color: {red};
            border-radius: 9999px;
            padding: 4px 14px;
            font-weight: bold;
            font-size: 12px;
        }}
        .badge-idle {{
            background-color: alpha(currentColor, 0.08);
            color: alpha(currentColor, 0.65);
            border-radius: 9999px;
            padding: 4px 14px;
            font-weight: bold;
            font-size: 12px;
        }}
        .hero-card {{
            padding: 16px 16px;
            margin: 0 0 6px 0;
            border-radius: 14px;
        }}
        .stat-pill {{
            background-color: alpha(currentColor, 0.05);
            border-radius: 10px;
            padding: 10px 8px;
        }}
        .mini-hud-bar {{
            background-color: {bg};
            padding: 2px 6px;
        }}
        .mini-hud-bar button {{
            min-height: 28px;
            min-width: 28px;
            padding: 3px;
            margin: 0;
            border-radius: 4px;
        }}
        .mini-hud-bar label.badge-idle,
        .mini-hud-bar label.badge-focus,
        .mini-hud-bar label.badge-micro,
        .mini-hud-bar label.badge-break {{
            padding: 2px 8px;
            font-size: 11px;
            border-radius: 4px;
            margin: 0 2px;
        }}
        .mini-hud-timer {{
            font-size: 15px;
            font-weight: 700;
            font-family: 'JetBrains Mono', monospace;
            letter-spacing: 0.5px;
            margin: 0 4px;
        }}
        "#,
        accent = colors.accent,
        bg = colors.background,
        lbg = colors.lighter_background,
        fg = colors.foreground,
        red = colors.red,
        green = colors.green,
        yellow = colors.yellow,
    )
}
