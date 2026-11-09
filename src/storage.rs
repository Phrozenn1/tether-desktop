use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "no")]
    Norwegian,
    #[serde(rename = "en")]
    English,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageSetting {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "no")]
    Norwegian,
    #[serde(rename = "en")]
    English,
}

impl Default for Language {
    fn default() -> Self {
        detect_system_locale()
    }
}

impl Default for LanguageSetting {
    fn default() -> Self {
        LanguageSetting::Auto
    }
}

impl LanguageSetting {
    pub fn resolve(&self) -> Language {
        match self {
            LanguageSetting::Auto => detect_system_locale(),
            LanguageSetting::Norwegian => Language::Norwegian,
            LanguageSetting::English => Language::English,
        }
    }
}

pub fn detect_system_locale() -> Language {
    for name in gtk4::glib::language_names() {
        let lower = name.to_lowercase();
        if lower.starts_with("nb") || lower.starts_with("nn") || lower.starts_with("no") {
            return Language::Norwegian;
        }
    }

    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let lower = val.to_lowercase();
            if lower.starts_with("nb") || lower.starts_with("nn") || lower.starts_with("no") {
                return Language::Norwegian;
            }
        }
    }

    Language::English
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideQuestFormat {
    #[serde(rename = "markdown_checkbox")]
    MarkdownCheckbox,
    #[serde(rename = "markdown_bullet")]
    MarkdownBullet,
    #[serde(rename = "todo_txt")]
    TodoTxt,
    #[serde(rename = "json")]
    Json,
}

impl Default for SideQuestFormat {
    fn default() -> Self {
        SideQuestFormat::MarkdownCheckbox
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniHudPlacement {
    #[serde(rename = "floating")]
    Floating,
    #[serde(rename = "top_right")]
    TopRight,
    #[serde(rename = "bottom_right")]
    BottomRight,
    #[serde(rename = "top_left")]
    TopLeft,
    #[serde(rename = "bottom_left")]
    BottomLeft,
    #[serde(rename = "top_center")]
    TopCenter,
}

impl Default for MiniHudPlacement {
    fn default() -> Self {
        MiniHudPlacement::Floating
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMinimizeAction {
    #[serde(rename = "tray")]
    Tray,
    #[serde(rename = "mini_hud")]
    MiniHud,
    #[serde(rename = "keep_open")]
    KeepOpen,
}

impl Default for EditorMinimizeAction {
    fn default() -> Self {
        EditorMinimizeAction::Tray
    }
}

impl EditorMinimizeAction {
    pub fn display_name(&self) -> &'static str {
        match self {
            EditorMinimizeAction::Tray => "Minimer til system-tray (skjul vindu)",
            EditorMinimizeAction::MiniHud => "Minimer til Mini-HUD (lite hjørnevindu)",
            EditorMinimizeAction::KeepOpen => "Ikke minimer (behold fullt vindu)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetherSettings {
    #[serde(default)]
    pub language: LanguageSetting,
    #[serde(default = "default_focus_ratio")]
    pub ratio_focus: u32,
    #[serde(default = "default_break_ratio")]
    pub ratio_break: u32,
    #[serde(default = "default_side_quests_file")]
    pub side_quests_file: String,
    #[serde(default)]
    pub side_quests_format: SideQuestFormat,
    #[serde(default)]
    pub editor_type: crate::editor::EditorType,
    #[serde(default)]
    pub auto_launch_desktop: bool,
    #[serde(default)]
    pub editor_minimize_action: Option<EditorMinimizeAction>,
    #[serde(default)]
    pub nvim_plugin_manager: crate::editor::NvimPluginManager,
    #[serde(default = "default_terminal_command")]
    pub terminal_command: String,
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    #[serde(default = "default_true")]
    pub remind_ramdump: bool,
    #[serde(default = "default_locked_monitor")]
    pub locked_monitor: String,
    #[serde(default)]
    pub custom_pos_mini: Option<(i32, i32)>,
    #[serde(default)]
    pub custom_pos_full: Option<(i32, i32)>,
    #[serde(default)]
    pub mini_hud_placement: MiniHudPlacement,
    #[serde(default = "default_soundscape")]
    pub soundscape: crate::audio::SoundscapeType,
    #[serde(default = "default_soundscape_volume")]
    pub soundscape_volume: f32,
    #[serde(default = "default_true")]
    pub soundscape_auto_play: bool,
}

fn default_focus_ratio() -> u32 { 45 }
fn default_break_ratio() -> u32 { 15 }
fn default_true() -> bool { true }
fn default_locked_monitor() -> String { "auto".to_string() }
fn default_terminal_command() -> String { "xdg-terminal-exec nvim".to_string() }
fn default_soundscape() -> crate::audio::SoundscapeType { crate::audio::SoundscapeType::Off }
fn default_soundscape_volume() -> f32 { 0.45 }
fn default_side_quests_file() -> String {
    get_data_dir().join("impulses.md").to_string_lossy().to_string()
}

impl Default for TetherSettings {
    fn default() -> Self {
        Self {
            language: LanguageSetting::Auto,
            ratio_focus: 45,
            ratio_break: 15,
            side_quests_file: default_side_quests_file(),
            side_quests_format: SideQuestFormat::MarkdownCheckbox,
            editor_type: crate::editor::EditorType::Auto,
            auto_launch_desktop: false,
            editor_minimize_action: None,
            nvim_plugin_manager: crate::editor::NvimPluginManager::Lazy,
            terminal_command: default_terminal_command(),
            notifications_enabled: true,
            remind_ramdump: true,
            locked_monitor: default_locked_monitor(),
            custom_pos_mini: None,
            custom_pos_full: None,
            mini_hud_placement: MiniHudPlacement::Floating,
            soundscape: default_soundscape(),
            soundscape_volume: default_soundscape_volume(),
            soundscape_auto_play: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFlowState {
    pub state: String, // "idle" | "focus" | "break"
    pub start_epoch: u64,
    pub break_duration_sec: u64,
    pub elapsed_focus_sec: u64,
    pub remaining_break_sec: u64,
    pub focus_ratio: u32,
    pub break_ratio: u32,
    pub writer: String, // "desktop" | "nvim"
    pub seq: u64,
    pub timestamp: u64,
}

pub fn get_data_dir() -> PathBuf {
    if let Some(state_dir) = dirs::state_dir() {
        state_dir.join("tether")
    } else if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("tether")
    } else {
        PathBuf::from(".tether")
    }
}

pub fn ensure_data_dir() -> Result<PathBuf, std::io::Error> {
    let dir = get_data_dir();
    create_dir_all(&dir)?;
    Ok(dir)
}

pub fn get_config_dir() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("tether")
    } else {
        PathBuf::from(".config/tether")
    }
}

pub fn resolve_path(path_str: &str) -> PathBuf {
    let trimmed = path_str.trim();
    if trimmed.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(trimmed.trim_start_matches("~/"));
        }
    } else if trimmed == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(trimmed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
    pub focused: bool,
}

impl MonitorInfo {
    pub fn logical_width(&self) -> f64 {
        if self.scale > 0.0 {
            self.width / self.scale
        } else {
            self.width
        }
    }

    pub fn logical_height(&self) -> f64 {
        if self.scale > 0.0 {
            self.height / self.scale
        } else {
            self.height
        }
    }

    pub fn mini_hud_pos(&self) -> (i32, i32) {
        let x = self.x + self.logical_width() - 284.0;
        let y = self.y + 48.0;
        (x as i32, y as i32)
    }

    pub fn expanded_center_pos(&self) -> (i32, i32) {
        let x = self.x + (self.logical_width() / 2.0) - 230.0;
        let y = self.y + (self.logical_height() / 2.0) - 290.0;
        (x as i32, y as i32)
    }

    pub fn display_label(&self) -> String {
        let desc = if !self.description.is_empty() {
            format!(" ({})", self.description)
        } else {
            String::new()
        };
        format!(
            "{} — {}x{}{}",
            self.name,
            self.logical_width() as i32,
            self.logical_height() as i32,
            desc
        )
    }
}

pub fn get_available_monitors() -> Vec<MonitorInfo> {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            if let Ok(out) = std::process::Command::new("hyprctl")
                .args(["monitors", "-j"])
                .output()
            {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                    if let Some(arr) = val.as_array() {
                        return arr
                            .iter()
                            .filter_map(|m| {
                                let name = m["name"].as_str()?.to_string();
                                let description = m["description"].as_str().unwrap_or("").to_string();
                                let id = m["id"].as_i64().unwrap_or(0);
                                let x = m["x"].as_f64().unwrap_or(0.0);
                                let y = m["y"].as_f64().unwrap_or(0.0);
                                let width = m["width"].as_f64().unwrap_or(1920.0);
                                let height = m["height"].as_f64().unwrap_or(1080.0);
                                let scale = m["scale"].as_f64().unwrap_or(1.0);
                                let focused = m["focused"].as_bool().unwrap_or(false);
                                Some(MonitorInfo {
                                    id,
                                    name,
                                    description,
                                    x,
                                    y,
                                    width,
                                    height,
                                    scale,
                                    focused,
                                })
                            })
                            .collect();
                    }
                }
            }
        }
    }
    Vec::new()
}

pub fn get_target_monitor(locked_mon: &str) -> Option<MonitorInfo> {
    let monitors = get_available_monitors();
    if monitors.is_empty() {
        return None;
    }
    if locked_mon == "auto" || locked_mon.is_empty() {
        if let Some(focused) = monitors.iter().find(|m| m.focused) {
            return Some(focused.clone());
        }
        return monitors.first().cloned();
    }
    if let Some(m) = monitors.iter().find(|m| m.name == locked_mon) {
        return Some(m.clone());
    }
    monitors.first().cloned()
}

impl MiniHudPlacement {
    pub fn calculate_pos(&self, settings: &TetherSettings) -> Option<(i32, i32)> {
        match self {
            MiniHudPlacement::Floating => {
                if let Some(pos) = settings.custom_pos_mini {
                    return Some(pos);
                }
                let mon = get_target_monitor(&settings.locked_monitor)?;
                let x = mon.x + mon.logical_width() - 320.0 - 24.0;
                let y = mon.y + 48.0;
                Some((x as i32, y as i32))
            }
            MiniHudPlacement::TopRight => {
                let mon = get_target_monitor(&settings.locked_monitor)?;
                let x = mon.x + mon.logical_width() - 320.0 - 24.0;
                let y = mon.y + 48.0;
                Some((x as i32, y as i32))
            }
            MiniHudPlacement::BottomRight => {
                let mon = get_target_monitor(&settings.locked_monitor)?;
                let x = mon.x + mon.logical_width() - 320.0 - 24.0;
                let y = mon.y + mon.logical_height() - 44.0 - 24.0;
                Some((x as i32, y as i32))
            }
            MiniHudPlacement::TopLeft => {
                let mon = get_target_monitor(&settings.locked_monitor)?;
                let x = mon.x + 24.0;
                let y = mon.y + 48.0;
                Some((x as i32, y as i32))
            }
            MiniHudPlacement::BottomLeft => {
                let mon = get_target_monitor(&settings.locked_monitor)?;
                let x = mon.x + 24.0;
                let y = mon.y + mon.logical_height() - 44.0 - 24.0;
                Some((x as i32, y as i32))
            }
            MiniHudPlacement::TopCenter => {
                let mon = get_target_monitor(&settings.locked_monitor)?;
                let x = mon.x + (mon.logical_width() - 320.0) / 2.0;
                let y = mon.y + 48.0;
                Some((x as i32, y as i32))
            }
        }
    }
}

pub fn ensure_pinned(pinned: bool) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("hyprctl").args(["clients", "-j"]).output() {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(arr) = val.as_array() {
                    for client in arr {
                        let class = client.get("class").and_then(|c| c.as_str()).unwrap_or("");
                        if class == "org.tether.Tether" || class == "tether" {
                            let is_currently_pinned = client.get("pinned").and_then(|p| p.as_bool()).unwrap_or(false);
                            if is_currently_pinned != pinned {
                                let _ = std::process::Command::new("hyprctl")
                                    .args(["dispatch", "hl.dsp.window.pin({ window = \"class:org.tether.Tether\" })"])
                                    .output();
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub fn load_settings() -> TetherSettings {
    let file = get_config_dir().join("config.json");
    if let Ok(bytes) = std::fs::read(&file) {
        if let Ok(settings) = serde_json::from_slice::<TetherSettings>(&bytes) {
            return settings;
        }
    }
    let default = TetherSettings::default();
    let _ = save_settings(&default);
    default
}

pub fn save_settings(settings: &TetherSettings) -> Result<(), std::io::Error> {
    let dir = get_config_dir();
    create_dir_all(&dir)?;
    let file = dir.join("config.json");
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(file, json)?;
    Ok(())
}

pub fn save_shared_state(state: &SharedFlowState) -> Result<(), std::io::Error> {
    let dir = ensure_data_dir()?;
    let file_path = dir.join("flow_state.json");
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(file_path, json)?;
    Ok(())
}

pub fn load_shared_state() -> Option<SharedFlowState> {
    let dir = get_data_dir();
    let file_path = dir.join("flow_state.json");
    if let Ok(bytes) = std::fs::read(file_path) {
        if let Ok(state) = serde_json::from_slice::<SharedFlowState>(&bytes) {
            return Some(state);
        }
    }
    None
}

// Side-Quests med format-støtte
#[derive(Serialize, Deserialize)]
struct JsonTask {
    timestamp: String,
    text: String,
    done: bool,
}

pub fn pick_impulse_file() -> Option<PathBuf> {
    // 1. System file dialog via rfd
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Velg fil for side-quests / impulser")
        .add_filter("Markdown (*.md)", &["md"])
        .add_filter("Tekstfiler (*.txt)", &["txt"])
        .add_filter("JSON (*.json)", &["json"])
        .add_filter("Alle filer (*.*)", &["*"])
        .save_file()
    {
        return Some(path);
    }

    // 2. Fallback til zenity på Linux hvis tilgjengelig
    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("zenity")
            .args([
                "--file-selection",
                "--save",
                "--confirm-overwrite",
                "--title=Velg fil for side-quests / impulser",
            ])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(PathBuf::from(s));
                }
            }
        }
    }

    None
}

pub fn append_impulse_with_settings(settings: &TetherSettings, text: &str) -> Result<(), std::io::Error> {
    let file_path = resolve_path(&settings.side_quests_file);
    if let Some(parent) = file_path.parent() {
        create_dir_all(parent)?;
    }

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();

    match settings.side_quests_format {
        SideQuestFormat::MarkdownCheckbox => {
            let mut file = OpenOptions::new().create(true).append(true).open(file_path)?;
            writeln!(file, "- [ ] [{}] {}", timestamp, text)?;
        }
        SideQuestFormat::MarkdownBullet => {
            let mut file = OpenOptions::new().create(true).append(true).open(file_path)?;
            writeln!(file, "- [{}] {}", timestamp, text)?;
        }
        SideQuestFormat::TodoTxt => {
            let mut file = OpenOptions::new().create(true).append(true).open(file_path)?;
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            writeln!(file, "(B) {} {} +sidequest", date, text)?;
        }
        SideQuestFormat::Json => {
            let mut list = load_json_tasks(&file_path);
            list.push(JsonTask {
                timestamp,
                text: text.to_string(),
                done: false,
            });
            save_json_tasks(&file_path, &list)?;
        }
    }
    Ok(())
}

fn load_json_tasks(path: &PathBuf) -> Vec<JsonTask> {
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(tasks) = serde_json::from_slice::<Vec<JsonTask>>(&bytes) {
            return tasks;
        }
    }
    Vec::new()
}

fn save_json_tasks(path: &PathBuf, tasks: &[JsonTask]) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(tasks)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_impulses_with_settings(settings: &TetherSettings) -> Vec<String> {
    let file_path = resolve_path(&settings.side_quests_file);
    if settings.side_quests_format == SideQuestFormat::Json {
        let tasks = load_json_tasks(&file_path);
        return tasks.into_iter().filter(|t| !t.done).map(|t| format!("- [{}] {}", t.timestamp, t.text)).collect();
    }

    if let Ok(mut file) = std::fs::File::open(file_path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            return content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|s| s.to_string())
                .collect();
        }
    }
    Vec::new()
}

pub fn save_all_impulses_with_settings(settings: &TetherSettings, items: &[String]) -> Result<(), std::io::Error> {
    let file_path = resolve_path(&settings.side_quests_file);
    if let Some(parent) = file_path.parent() {
        create_dir_all(parent)?;
    }

    if settings.side_quests_format == SideQuestFormat::Json {
        let tasks: Vec<JsonTask> = items
            .iter()
            .map(|s| JsonTask {
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                text: s.clone(),
                done: false,
            })
            .collect();
        return save_json_tasks(&file_path, &tasks);
    }

    let mut file = std::fs::File::create(file_path)?;
    for item in items {
        writeln!(file, "{}", item)?;
    }
    Ok(())
}

// Bakoverkompatibilitet
pub fn append_impulse(text: &str) -> Result<(), std::io::Error> {
    let settings = load_settings();
    append_impulse_with_settings(&settings, text)
}

pub fn load_impulses() -> Vec<String> {
    let settings = load_settings();
    load_impulses_with_settings(&settings)
}

pub fn save_all_impulses(items: &[String]) -> Result<(), std::io::Error> {
    let settings = load_settings();
    save_all_impulses_with_settings(&settings, items)
}

// RAM-Dump
pub fn append_ramdump(now: &str, status: &str, next_step: &str) -> Result<(), std::io::Error> {
    let dir = ensure_data_dir()?;
    let file_path = dir.join("ramdumps.md");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "--- PAUSE / SAVE-STATE ({}) ---", timestamp)?;
    writeln!(file, "[NÅ]:         {}", now)?;
    writeln!(file, "[STATUS]:     {}", status)?;
    writeln!(file, "[NESTE STEG]: {}", next_step)?;
    writeln!(file, "--------------------------------------------\n")?;
    Ok(())
}

pub fn load_last_ramdump() -> Option<(String, String, String, String)> {
    let dir = get_data_dir();
    let file_path = dir.join("ramdumps.md");
    if let Ok(mut file) = std::fs::File::open(file_path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            let blocks: Vec<&str> = content.split("--- PAUSE / SAVE-STATE").collect();
            if let Some(last) = blocks.last() {
                if !last.trim().is_empty() {
                    let mut time = String::new();
                    let mut now = String::new();
                    let mut status = String::new();
                    let mut next = String::new();

                    for line in last.lines() {
                        if line.contains('(') && line.contains(')') {
                            time = line.trim_matches(|c| c != '(' && c != ')').trim_matches(|c| c == '(' || c == ')').to_string();
                        } else if let Some(stripped) = line.strip_prefix("[NÅ]:") {
                            now = stripped.trim().to_string();
                        } else if let Some(stripped) = line.strip_prefix("[STATUS]:") {
                            status = stripped.trim().to_string();
                        } else if let Some(stripped) = line.strip_prefix("[NESTE STEG]:") {
                            next = stripped.trim().to_string();
                        }
                    }

                    if !now.is_empty() || !status.is_empty() {
                        return Some((time, now, status, next));
                    }
                }
            }
        }
    }
    None
}

pub fn get_tether_window_pos() -> Option<(i32, i32)> {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            if let Ok(out) = std::process::Command::new("hyprctl")
                .args(["clients", "-j"])
                .output()
            {
                if out.status.success() {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                        if let Some(arr) = val.as_array() {
                            for client in arr {
                                let class = client.get("class").and_then(|c| c.as_str()).unwrap_or("");
                                if class == "org.tether.Tether" || class == "tether" {
                                    if let Some(at) = client.get("at").and_then(|a| a.as_array()) {
                                        if at.len() == 2 {
                                            if let (Some(x), Some(y)) = (at[0].as_i64(), at[1].as_i64()) {
                                                return Some((x as i32, y as i32));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn move_window_to_pos(x: i32, y: i32) {
    #[cfg(target_os = "linux")]
    {
        let move_cmd = format!(
            "hl.dsp.window.move({{ x = {}, y = {}, window = \"class:org.tether.Tether\" }})",
            x, y
        );
        let _ = std::process::Command::new("hyprctl")
            .args(["dispatch", &move_cmd])
            .output();
    }
}

pub fn resize_window(w: i32, h: i32) {
    #[cfg(target_os = "linux")]
    {
        let cmd = format!(
            "hl.dsp.window.resize({{ x = {}, y = {}, relative = false, window = \"class:org.tether.Tether\" }})",
            w, h
        );
        let _ = std::process::Command::new("hyprctl")
            .args(["dispatch", &cmd])
            .output();
    }
}

pub fn clamp_full_window_pos(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    locked_mon: &str,
) -> (i32, i32) {
    if let Some(mon) = get_target_monitor(locked_mon) {
        let min_x = mon.x as i32 + 16;
        let max_x = (mon.x + mon.logical_width()) as i32 - width - 16;
        let min_y = mon.y as i32 + 36; // Keep clear of top bar
        let max_y = (mon.y + mon.logical_height()) as i32 - height - 16;

        let clamped_x = if max_x > min_x {
            x.clamp(min_x, max_x)
        } else {
            (mon.x as i32) + ((mon.logical_width() as i32 - width) / 2)
        };

        let clamped_y = if max_y > min_y {
            y.clamp(min_y, max_y)
        } else {
            (mon.y as i32) + ((mon.logical_height() as i32 - height) / 2)
        };

        (clamped_x, clamped_y)
    } else {
        (x.max(16), y.max(36))
    }
}

pub fn get_centered_full_pos(width: i32, height: i32, locked_mon: &str) -> (i32, i32) {
    if let Some(mon) = get_target_monitor(locked_mon) {
        let x = (mon.x as i32) + ((mon.logical_width() as i32 - width) / 2);
        let y = (mon.y as i32) + ((mon.logical_height() as i32 - height) / 2);
        (x, y)
    } else {
        (400, 180)
    }
}

// Neovim Deteksjon og Fokus
pub fn is_neovim_running() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("pgrep").args(["-x", "nvim"]).output() {
            if !output.stdout.is_empty() {
                return true;
            }
        }
    }
    false
}

pub fn open_or_focus_neovim(terminal_cmd: &str) {
    #[cfg(target_os = "linux")]
    {
        // Hvis nvim kjører under Hyprland, prøv å fokusere vinduet
        if let Ok(output) = std::process::Command::new("hyprctl").args(["clients", "-j"]).output() {
            if let Ok(clients) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(arr) = clients.as_array() {
                    let nvim_client = arr.iter().find(|c| {
                        let class = c["class"].as_str().unwrap_or("").to_lowercase();
                        let title = c["title"].as_str().unwrap_or("").to_lowercase();
                        class.contains("nvim") || title.contains("nvim") || class.contains("neovim")
                    });
                    if let Some(c) = nvim_client {
                        let address = c["address"].as_str().unwrap_or("");
                        let dispatch_arg = format!("hl.dsp.focus({{ window = \"address:{}\" }})", address);
                        let _ = std::process::Command::new("hyprctl")
                            .args(["dispatch", &dispatch_arg])
                            .output();
                        return;
                    }
                }
            }
        }
    }

    // Hvis ikke åpen, start ny terminal med nvim
    let parts: Vec<&str> = terminal_cmd.split_whitespace().collect();
    if !parts.is_empty() {
        let _ = std::process::Command::new(parts[0])
            .args(&parts[1..])
            .spawn();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextAnchor {
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default)]
    pub project_name: String,
    #[serde(default)]
    pub project_path: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub relative_path: String,
    #[serde(default)]
    pub line: usize,
    #[serde(default)]
    pub col: usize,
    #[serde(default)]
    pub last_note: String,
    #[serde(default)]
    pub server_name: String,
}

pub fn get_context_anchor_file() -> PathBuf {
    get_data_dir().join("context_anchor.json")
}

pub fn load_context_anchor() -> Option<ContextAnchor> {
    let path = get_context_anchor_file();
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(anchor) = serde_json::from_slice::<ContextAnchor>(&bytes) {
            if !anchor.file_path.is_empty() {
                return Some(anchor);
            }
        }
    }
    None
}

pub fn resume_in_neovim(anchor: &ContextAnchor, terminal_cmd: &str) {
    let socket_alive = !anchor.server_name.is_empty() && std::path::Path::new(&anchor.server_name).exists();
    if socket_alive {
        let cmd = format!("\x1c\x0e:e +{} {}\r", anchor.line, anchor.file_path);
        let _ = std::process::Command::new("nvim")
            .args(["--server", &anchor.server_name, "--remote-send", &cmd])
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("hyprctl").args(["clients", "-j"]).output() {
            if let Ok(clients) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(arr) = clients.as_array() {
                    let nvim_client = arr.iter().find(|c| {
                        let class = c["class"].as_str().unwrap_or("").to_lowercase();
                        let title = c["title"].as_str().unwrap_or("").to_lowercase();
                        class.contains("nvim") || title.contains("nvim") || class.contains("ghostty") || class.contains("kitty") || class.contains("alacritty")
                    });
                    if let Some(c) = nvim_client {
                        if let Some(address) = c["address"].as_str() {
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

    if !socket_alive {
        let parts: Vec<&str> = terminal_cmd.split_whitespace().collect();
        if !parts.is_empty() {
            let mut args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            args.push(format!("+{}", anchor.line));
            args.push(anchor.file_path.clone());
            let _ = std::process::Command::new(parts[0])
                .args(&args)
                .spawn();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumTier {
    Spark,       // 0 - 499 XP
    FlowRider,   // 500 - 1999 XP
    DeepDiver,   // 2000 - 4999 XP
    CodeWizard,  // 5000+ XP
}

impl MomentumTier {
    pub fn title_no(&self) -> &'static str {
        match self {
            MomentumTier::Spark => "⚡ Gnist",
            MomentumTier::FlowRider => "🌊 Flyt-rytter",
            MomentumTier::DeepDiver => "🚀 Dypdykker",
            MomentumTier::CodeWizard => "🧙‍♂️ Flyt-mester",
        }
    }

    pub fn title_en(&self) -> &'static str {
        match self {
            MomentumTier::Spark => "⚡ Spark",
            MomentumTier::FlowRider => "🌊 Flow Rider",
            MomentumTier::DeepDiver => "🚀 Deep Diver",
            MomentumTier::CodeWizard => "🧙‍♂️ Flow Master",
        }
    }

    pub fn current_threshold(&self) -> u64 {
        match self {
            MomentumTier::Spark => 0,
            MomentumTier::FlowRider => 500,
            MomentumTier::DeepDiver => 2000,
            MomentumTier::CodeWizard => 5000,
        }
    }

    pub fn next_threshold(&self) -> u64 {
        match self {
            MomentumTier::Spark => 500,
            MomentumTier::FlowRider => 2000,
            MomentumTier::DeepDiver => 5000,
            MomentumTier::CodeWizard => 10000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    #[serde(default)]
    pub total_focus_mins: u64,
    #[serde(default)]
    pub current_week_mins: u64,
    #[serde(default)]
    pub current_week_sessions: u32,
    #[serde(default)]
    pub current_week_impulses_captured: u32,
    #[serde(default = "default_week_identifier")]
    pub week_identifier: String,
    #[serde(default)]
    pub daily_mins: std::collections::HashMap<String, u32>,
    #[serde(default)]
    pub xp: u64,
}

fn default_week_identifier() -> String {
    chrono::Local::now().format("%G-W%V").to_string()
}

impl Default for UserStats {
    fn default() -> Self {
        Self {
            total_focus_mins: 0,
            current_week_mins: 0,
            current_week_sessions: 0,
            current_week_impulses_captured: 0,
            week_identifier: default_week_identifier(),
            daily_mins: std::collections::HashMap::new(),
            xp: 0,
        }
    }
}

impl UserStats {
    pub fn tier(&self) -> MomentumTier {
        if self.xp < 500 {
            MomentumTier::Spark
        } else if self.xp < 2000 {
            MomentumTier::FlowRider
        } else if self.xp < 5000 {
            MomentumTier::DeepDiver
        } else {
            MomentumTier::CodeWizard
        }
    }

    pub fn check_week_rollover(&mut self) {
        let current_week = default_week_identifier();
        if self.week_identifier != current_week {
            self.week_identifier = current_week;
            self.current_week_mins = 0;
            self.current_week_sessions = 0;
            self.current_week_impulses_captured = 0;
        }
    }

    pub fn record_focus_minutes(&mut self, minutes: u64) {
        if minutes == 0 {
            return;
        }
        self.check_week_rollover();
        self.total_focus_mins += minutes;
        self.current_week_mins += minutes;
        self.current_week_sessions += 1;
        self.xp += minutes * 10;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let current = self.daily_mins.entry(today).or_insert(0);
        *current += minutes as u32;
    }

    pub fn record_micro_start(&mut self) {
        self.check_week_rollover();
        self.xp += 100;
    }

    pub fn record_impulse(&mut self) {
        self.check_week_rollover();
        self.current_week_impulses_captured += 1;
        self.xp += 25;
    }
}

pub fn get_stats_file() -> PathBuf {
    get_data_dir().join("stats.json")
}

pub fn load_user_stats() -> UserStats {
    let path = get_stats_file();
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(mut stats) = serde_json::from_slice::<UserStats>(&bytes) {
            stats.check_week_rollover();
            return stats;
        }
    }
    UserStats::default()
}

pub fn save_user_stats(stats: &UserStats) -> Result<(), std::io::Error> {
    let path = get_stats_file();
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(stats)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)?;
    Ok(())
}
