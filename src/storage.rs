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

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetherSettings {
    #[serde(default)]
    pub language: Language,
    #[serde(default = "default_focus_ratio")]
    pub ratio_focus: u32,
    #[serde(default = "default_break_ratio")]
    pub ratio_break: u32,
    #[serde(default = "default_side_quests_file")]
    pub side_quests_file: String,
    #[serde(default)]
    pub side_quests_format: SideQuestFormat,
    #[serde(default)]
    pub auto_launch_desktop: bool,
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
            language: Language::English,
            ratio_focus: 45,
            ratio_break: 15,
            side_quests_file: default_side_quests_file(),
            side_quests_format: SideQuestFormat::MarkdownCheckbox,
            auto_launch_desktop: false,
            terminal_command: default_terminal_command(),
            notifications_enabled: true,
            remind_ramdump: true,
            locked_monitor: default_locked_monitor(),
            custom_pos_mini: None,
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
