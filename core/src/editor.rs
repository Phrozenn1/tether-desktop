use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::storage::ContextAnchor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorType {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "nvim")]
    Neovim,
    #[serde(rename = "neovide")]
    Neovide,
    #[serde(rename = "vscode")]
    VSCode,
    #[serde(rename = "vscodium")]
    VSCodium,
    #[serde(rename = "cursor")]
    Cursor,
    #[serde(rename = "helix")]
    Helix,
    #[serde(rename = "obsidian")]
    Obsidian,
    #[serde(rename = "typora")]
    Typora,
    #[serde(rename = "libreoffice")]
    LibreOffice,
    #[serde(rename = "pdf_reader")]
    PdfReader,
    #[serde(rename = "image_editor")]
    ImageEditor,
    #[serde(rename = "video_audio")]
    VideoAudio,
    #[serde(rename = "custom")]
    Custom,
}

impl Default for EditorType {
    fn default() -> Self {
        EditorType::Auto
    }
}

const PDF_BINS: &[&str] = &["sioyek", "evince", "zathura", "okular", "xournalpp"];
const IMAGE_BINS: &[&str] = &["gimp", "krita", "inkscape", "darktable"];
const VIDEO_BINS: &[&str] = &["kdenlive", "omacut", "blender", "audacity", "obs"];
const LIBREOFFICE_BINS: &[&str] = &["libreoffice", "soffice", "soffice.bin"];

pub fn binary_exists(bin: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for p in std::env::split_paths(&path) {
            let full = p.join(bin);
            if full.is_file() {
                return true;
            }
        }
    }
    false
}

fn any_binary_exists(bins: &[&str]) -> bool {
    bins.iter().any(|b| binary_exists(b))
}

pub fn binary_running(bin: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("pgrep").args(["-x", bin]).output() {
            if !output.stdout.is_empty() {
                return true;
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = bin;
    false
}

fn any_binary_running(bins: &[&str]) -> bool {
    bins.iter().any(|b| binary_running(b))
}

fn hyprland_client_matches(keywords: &[&str]) -> bool {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            if let Ok(output) = std::process::Command::new("hyprctl").args(["clients", "-j"]).output() {
                if let Ok(clients) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    if let Some(arr) = clients.as_array() {
                        return arr.iter().any(|c| {
                            let class = c["class"].as_str().unwrap_or("").to_lowercase();
                            let title = c["title"].as_str().unwrap_or("").to_lowercase();
                            keywords.iter().any(|k| class.contains(k) || title.contains(k))
                        });
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = keywords;
    false
}

pub fn find_obsidian_vaults() -> Vec<PathBuf> {
    let mut vaults = Vec::new();
    let Some(home) = dirs::home_dir() else {
        return vaults;
    };

    for root in [home.join("Documents"), home.join("Obsidian"), home.clone()] {
        if root.join(".obsidian").is_dir() && !vaults.contains(&root) {
            vaults.push(root.clone());
        }
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.join(".obsidian").is_dir() && !vaults.contains(&p) {
                    vaults.push(p);
                }
            }
        }
    }
    vaults.sort();
    vaults
}

fn open_with_system_handler(target: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(target).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(target).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", target]).spawn();
}

fn percent_encode_uri(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        let ch = *byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.' | '~' | ':') {
            out.push(ch);
        } else {
            out.push_str(&format!("%{:02X}", byte));
        }
    }
    out
}

fn percent_decode_uri(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(val) = u8::from_str_radix(hex, 16) {
                    out.push(val);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

impl EditorType {
    pub fn id(&self) -> &'static str {
        match self {
            EditorType::Auto => "auto",
            EditorType::None => "none",
            EditorType::Neovim => "nvim",
            EditorType::Neovide => "neovide",
            EditorType::VSCode => "code",
            EditorType::VSCodium => "codium",
            EditorType::Cursor => "cursor",
            EditorType::Helix => "helix",
            EditorType::Obsidian => "obsidian",
            EditorType::Typora => "typora",
            EditorType::LibreOffice => "libreoffice",
            EditorType::PdfReader => "pdf_reader",
            EditorType::ImageEditor => "image_editor",
            EditorType::VideoAudio => "video_audio",
            EditorType::Custom => "custom",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EditorType::Auto => "Automatisk (Oppdag aktiv)",
            EditorType::None => "Ingen / Frittstående",
            EditorType::Neovim => "Neovim (Terminal)",
            EditorType::Neovide => "Neovide (GUI)",
            EditorType::VSCode => "Visual Studio Code",
            EditorType::VSCodium => "VSCodium",
            EditorType::Cursor => "Cursor",
            EditorType::Helix => "Helix",
            EditorType::Obsidian => "Obsidian (Notater & Skriving)",
            EditorType::Typora => "Typora (Skriving)",
            EditorType::LibreOffice => "LibreOffice (Dokumenter / Kontor)",
            EditorType::PdfReader => "PDF & Studier (Sioyek / Evince / Zathura)",
            EditorType::ImageEditor => "Bilde & Design (GIMP / Krita / Inkscape)",
            EditorType::VideoAudio => "Video, 3D & Lyd (Kdenlive / Omacut / Blender / Audacity)",
            EditorType::Custom => "Egendefinert",
        }
    }

    pub fn binary_name(&self) -> Option<&'static str> {
        match self {
            EditorType::Auto
            | EditorType::None
            | EditorType::Custom
            | EditorType::PdfReader
            | EditorType::ImageEditor
            | EditorType::VideoAudio => None,
            EditorType::Neovim => Some("nvim"),
            EditorType::Neovide => Some("neovide"),
            EditorType::VSCode => Some("code"),
            EditorType::VSCodium => Some("codium"),
            EditorType::Cursor => Some("cursor"),
            EditorType::Helix => Some("hx"),
            EditorType::Obsidian => Some("obsidian"),
            EditorType::Typora => Some("typora"),
            EditorType::LibreOffice => Some("libreoffice"),
        }
    }

    pub fn is_installed(&self) -> bool {
        match self {
            EditorType::Obsidian => binary_exists("obsidian") || !find_obsidian_vaults().is_empty(),
            EditorType::Typora => binary_exists("typora"),
            EditorType::LibreOffice => any_binary_exists(&["libreoffice", "soffice"]),
            EditorType::PdfReader => any_binary_exists(PDF_BINS),
            EditorType::ImageEditor => any_binary_exists(IMAGE_BINS),
            EditorType::VideoAudio => any_binary_exists(VIDEO_BINS),
            _ => match self.binary_name() {
                Some(bin) => binary_exists(bin),
                None => true,
            },
        }
    }

    pub fn detected_editors() -> Vec<EditorType> {
        let candidates = [
            EditorType::Obsidian,
            EditorType::LibreOffice,
            EditorType::Typora,
            EditorType::PdfReader,
            EditorType::ImageEditor,
            EditorType::VideoAudio,
            EditorType::Neovim,
            EditorType::VSCode,
            EditorType::VSCodium,
            EditorType::Cursor,
            EditorType::Neovide,
            EditorType::Helix,
        ];
        candidates.into_iter().filter(|e| e.is_installed()).collect()
    }

    /// Resolves `Auto` to the most relevant installed app (or `None`), factoring in anchor & running processes.
    pub fn resolve_context(&self, anchor: Option<&ContextAnchor>) -> EditorType {
        match self {
            EditorType::Auto => {
                if let Some(a) = anchor {
                    let s = a.server_name.to_lowercase();
                    let resolved = if s.contains("obsidian") {
                        Some(EditorType::Obsidian)
                    } else if s.contains("typora") {
                        Some(EditorType::Typora)
                    } else if s.contains("libreoffice") || s.contains("soffice") {
                        Some(EditorType::LibreOffice)
                    } else if ["pdf_reader", "sioyek", "evince", "zathura", "okular", "xournal"]
                        .iter()
                        .any(|k| s.contains(k))
                    {
                        Some(EditorType::PdfReader)
                    } else if ["image_editor", "gimp", "krita", "inkscape", "darktable"]
                        .iter()
                        .any(|k| s.contains(k))
                    {
                        Some(EditorType::ImageEditor)
                    } else if ["video_audio", "kdenlive", "omacut", "blender", "audacity", "obs"]
                        .iter()
                        .any(|k| s.contains(k))
                    {
                        Some(EditorType::VideoAudio)
                    } else if s.contains("vscode") || s.contains("code") {
                        Some(EditorType::VSCode)
                    } else if s.contains("codium") {
                        Some(EditorType::VSCodium)
                    } else if s.contains("cursor") {
                        Some(EditorType::Cursor)
                    } else if s.contains("neovide") {
                        Some(EditorType::Neovide)
                    } else if s.contains("nvim") || s.contains("neovim") {
                        Some(EditorType::Neovim)
                    } else if s.contains("helix") || s.contains("hx") {
                        Some(EditorType::Helix)
                    } else {
                        None
                    };
                    if let Some(app) = resolved {
                        if app.is_installed() {
                            return app;
                        }
                    }
                }

                if let Some(running) = Self::detect_running_editor() {
                    return running;
                }

                for cand in [
                    EditorType::Neovim,
                    EditorType::Obsidian,
                    EditorType::VSCode,
                    EditorType::LibreOffice,
                    EditorType::Typora,
                    EditorType::PdfReader,
                    EditorType::ImageEditor,
                    EditorType::VideoAudio,
                    EditorType::Neovide,
                    EditorType::Cursor,
                    EditorType::VSCodium,
                    EditorType::Helix,
                ] {
                    if cand.is_installed() {
                        return cand;
                    }
                }
                EditorType::None
            }
            other => *other,
        }
    }

    pub fn resolve(&self) -> EditorType {
        self.resolve_context(None)
    }

    pub fn detect_running_editor() -> Option<EditorType> {
        let code_running = binary_running("code");
        let nvim_running = binary_running("nvim");
        let neovide_running = binary_running("neovide");
        let cursor_running = binary_running("cursor");

        if code_running && !nvim_running && !neovide_running {
            return Some(EditorType::VSCode);
        }
        if (nvim_running || neovide_running) && !code_running {
            return if neovide_running { Some(EditorType::Neovide) } else { Some(EditorType::Neovim) };
        }
        if cursor_running && !code_running && !nvim_running {
            return Some(EditorType::Cursor);
        }
        if binary_running("obsidian") || hyprland_client_matches(&["obsidian"]) {
            return Some(EditorType::Obsidian);
        }
        if any_binary_running(LIBREOFFICE_BINS) || hyprland_client_matches(&["libreoffice"]) {
            return Some(EditorType::LibreOffice);
        }
        if binary_running("typora") {
            return Some(EditorType::Typora);
        }
        if any_binary_running(PDF_BINS) {
            return Some(EditorType::PdfReader);
        }
        if any_binary_running(VIDEO_BINS) {
            return Some(EditorType::VideoAudio);
        }
        if any_binary_running(IMAGE_BINS) {
            return Some(EditorType::ImageEditor);
        }
        None
    }

    pub fn is_running(&self) -> bool {
        let active = self.resolve();
        match active {
            EditorType::Obsidian => {
                binary_running("obsidian") || hyprland_client_matches(&["obsidian"])
            }
            EditorType::LibreOffice => {
                any_binary_running(LIBREOFFICE_BINS) || hyprland_client_matches(&["libreoffice"])
            }
            EditorType::PdfReader => {
                any_binary_running(PDF_BINS) || hyprland_client_matches(PDF_BINS)
            }
            EditorType::ImageEditor => {
                any_binary_running(IMAGE_BINS) || hyprland_client_matches(IMAGE_BINS)
            }
            EditorType::VideoAudio => {
                any_binary_running(VIDEO_BINS) || hyprland_client_matches(VIDEO_BINS)
            }
            _ => match active.binary_name() {
                Some(bin) => binary_running(bin),
                None => false,
            },
        }
    }

    pub fn short_badge(&self) -> &'static str {
        match self.resolve() {
            EditorType::Neovim => "NVIM",
            EditorType::Neovide => "Neovide",
            EditorType::VSCode => "VS Code",
            EditorType::VSCodium => "Codium",
            EditorType::Cursor => "Cursor",
            EditorType::Helix => "Helix",
            EditorType::Obsidian => "Obsidian",
            EditorType::Typora => "Typora",
            EditorType::LibreOffice => "LibreOffice",
            EditorType::PdfReader => "PDF/Studier",
            EditorType::ImageEditor => "Design/Bilde",
            EditorType::VideoAudio => "Media/Video",
            EditorType::Custom => "App",
            _ => "",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self.resolve() {
            EditorType::Neovim | EditorType::Helix => "utilities-terminal-symbolic",
            EditorType::Obsidian | EditorType::Typora | EditorType::LibreOffice => {
                "x-office-document-symbolic"
            }
            EditorType::PdfReader => "application-pdf-symbolic",
            EditorType::ImageEditor => "applications-graphics-symbolic",
            EditorType::VideoAudio => "applications-multimedia-symbolic",
            _ => "text-editor-symbolic",
        }
    }

    pub fn resume_button_label(&self, lang: crate::storage::Language) -> String {
        let active = self.resolve();
        let app_name = match active {
            EditorType::Neovim => "Neovim",
            EditorType::Neovide => "Neovide",
            EditorType::VSCode => "VS Code",
            EditorType::VSCodium => "VSCodium",
            EditorType::Cursor => "Cursor",
            EditorType::Helix => "Helix",
            EditorType::Obsidian => "Obsidian",
            EditorType::Typora => "Typora",
            EditorType::LibreOffice => "LibreOffice",
            EditorType::PdfReader => {
                if lang == crate::storage::Language::Norwegian {
                    "PDF / studie"
                } else {
                    "PDF / study"
                }
            }
            EditorType::ImageEditor => {
                if lang == crate::storage::Language::Norwegian {
                    "bildeprosjekt"
                } else {
                    "design project"
                }
            }
            EditorType::VideoAudio => {
                if lang == crate::storage::Language::Norwegian {
                    "mediaprosjekt"
                } else {
                    "media project"
                }
            }
            EditorType::Custom => "app",
            EditorType::Auto | EditorType::None => {
                if lang == crate::storage::Language::Norwegian {
                    "dokument / fil"
                } else {
                    "document / file"
                }
            }
        };

        match lang {
            crate::storage::Language::Norwegian => format!("Gjenoppta i {}", app_name),
            _ => format!("Resume in {}", app_name),
        }
    }

    /// Return available choices for the Settings ComboRow.
    pub fn available_choices() -> Vec<EditorType> {
        let mut list = vec![EditorType::Auto, EditorType::None];
        let candidates = [
            EditorType::Obsidian,
            EditorType::LibreOffice,
            EditorType::Typora,
            EditorType::PdfReader,
            EditorType::ImageEditor,
            EditorType::VideoAudio,
            EditorType::VSCode,
            EditorType::Neovim,
            EditorType::Neovide,
            EditorType::VSCodium,
            EditorType::Cursor,
            EditorType::Helix,
        ];
        for cand in candidates {
            if cand.is_installed() {
                list.push(cand);
            }
        }
        list
    }

    pub fn open_or_focus(&self, terminal_cmd: &str) {
        let active = self.resolve();
        match active {
            EditorType::Neovim => {
                crate::storage::open_or_focus_neovim(terminal_cmd);
            }
            EditorType::Neovide => {
                focus_or_spawn_window(&["neovide"], "neovide");
            }
            EditorType::VSCode => {
                focus_or_spawn_window(&["code"], "code");
            }
            EditorType::VSCodium => {
                focus_or_spawn_window(&["codium"], "codium");
            }
            EditorType::Cursor => {
                focus_or_spawn_window(&["cursor"], "cursor");
            }
            EditorType::Helix => {
                let parts: Vec<&str> = terminal_cmd.split_whitespace().collect();
                if !parts.is_empty() {
                    let mut args: Vec<&str> = parts[1..].to_vec();
                    args.push("hx");
                    let _ = std::process::Command::new(parts[0]).args(&args).spawn();
                }
            }
            EditorType::Obsidian => {
                if binary_exists("obsidian") {
                    focus_or_spawn_window(&["obsidian"], "obsidian");
                } else {
                    focus_or_spawn_window(&["xdg-open", "obsidian://"], "obsidian");
                }
            }
            EditorType::Typora => {
                focus_or_spawn_window(&["typora"], "typora");
            }
            EditorType::LibreOffice => {
                focus_or_spawn_window(&["libreoffice"], "libreoffice");
            }
            EditorType::PdfReader => {
                for b in PDF_BINS {
                    if binary_exists(b) {
                        focus_or_spawn_window(&[b], b);
                        break;
                    }
                }
            }
            EditorType::ImageEditor => {
                for b in IMAGE_BINS {
                    if binary_exists(b) {
                        focus_or_spawn_window(&[b], b);
                        break;
                    }
                }
            }
            EditorType::VideoAudio => {
                for b in VIDEO_BINS {
                    if binary_exists(b) {
                        focus_or_spawn_window(&[b], b);
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn resume(&self, anchor: &ContextAnchor, terminal_cmd: &str) {
        let active = self.resolve_context(Some(anchor));
        match active {
            EditorType::Neovim => {
                crate::storage::resume_in_neovim(anchor, terminal_cmd);
            }
            EditorType::Neovide => {
                let _ = std::process::Command::new("neovide")
                    .args([&anchor.file_path, &format!("+{}", anchor.line)])
                    .spawn();
            }
            EditorType::VSCode => {
                let target = format!("{}:{}", anchor.file_path, anchor.line);
                let _ = std::process::Command::new("code").args(["-g", &target]).spawn();
            }
            EditorType::VSCodium => {
                let target = format!("{}:{}", anchor.file_path, anchor.line);
                let _ = std::process::Command::new("codium").args(["-g", &target]).spawn();
            }
            EditorType::Cursor => {
                let target = format!("{}:{}", anchor.file_path, anchor.line);
                let _ = std::process::Command::new("cursor").args(["-g", &target]).spawn();
            }
            EditorType::Helix => {
                let parts: Vec<&str> = terminal_cmd.split_whitespace().collect();
                if !parts.is_empty() {
                    let mut args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
                    args.push(format!("{}:{}", anchor.file_path, anchor.line));
                    let _ = std::process::Command::new(parts[0]).args(&args).spawn();
                }
            }
            EditorType::Obsidian => {
                if !anchor.file_path.is_empty() {
                    let uri = format!("obsidian://open?path={}", percent_encode_uri(&anchor.file_path));
                    open_with_system_handler(&uri);
                }
                if hyprland_client_matches(&["obsidian"]) {
                    focus_or_spawn_window(&["obsidian"], "obsidian");
                }
            }
            EditorType::Typora
            | EditorType::LibreOffice
            | EditorType::PdfReader
            | EditorType::ImageEditor
            | EditorType::VideoAudio => {
                let file_exists = !anchor.file_path.is_empty() && Path::new(&anchor.file_path).exists();
                let target_keyword = if !anchor.server_name.is_empty() {
                    anchor.server_name.to_lowercase()
                } else {
                    active.id().to_string()
                };
                let doc_name = Path::new(&anchor.file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if !doc_name.is_empty() && hyprland_client_matches(&[&doc_name]) {
                    focus_or_spawn_window(&[], &doc_name);
                } else if file_exists {
                    open_with_system_handler(&anchor.file_path);
                    if hyprland_client_matches(&[&target_keyword]) {
                        focus_or_spawn_window(&[], &target_keyword);
                    }
                } else if !anchor.server_name.is_empty() {
                    focus_or_spawn_window(&[], &target_keyword);
                }
            }
            _ => {
                if !anchor.file_path.is_empty() && Path::new(&anchor.file_path).exists() {
                    open_with_system_handler(&anchor.file_path);
                }
            }
        }
    }
}

pub fn focus_or_spawn_window(cmd: &[&str], match_name: &str) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("hyprctl").args(["clients", "-j"]).output() {
            if let Ok(clients) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(arr) = clients.as_array() {
                    let needle = match_name.to_lowercase();
                    let client = arr.iter().find(|c| {
                        let class = c["class"].as_str().unwrap_or("").to_lowercase();
                        let title = c["title"].as_str().unwrap_or("").to_lowercase();
                        class.contains(&needle) || title.contains(&needle)
                    });
                    if let Some(c) = client {
                        if let Some(address) = c["address"].as_str() {
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
    }

    if !cmd.is_empty() {
        let _ = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .spawn();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NvimPluginManager {
    #[serde(rename = "lazy")]
    Lazy,
    #[serde(rename = "packpath")]
    Packpath,
    #[serde(rename = "vimplug")]
    VimPlug,
    #[serde(rename = "manual")]
    Manual,
}

impl Default for NvimPluginManager {
    fn default() -> Self {
        NvimPluginManager::Lazy
    }
}

impl NvimPluginManager {
    pub fn display_name(&self) -> &'static str {
        match self {
            NvimPluginManager::Lazy => "lazy.nvim (~/.config/nvim/lua/plugins/)",
            NvimPluginManager::Packpath => "Innebygd packpath (Krever null oppsett)",
            NvimPluginManager::VimPlug => "vim-plug (init.vim / init.lua)",
            NvimPluginManager::Manual => "Manuell (Vis konfigurasjon)",
        }
    }
}

pub fn check_nvim_plugin_status(manager: NvimPluginManager) -> (bool, String) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return (false, "Finner ikke hjemmemappe".to_string()),
    };

    match manager {
        NvimPluginManager::Lazy => {
            let spec_path = home.join(".config/nvim/lua/plugins/tether.lua");
            if spec_path.exists() {
                (true, "Installert (i ~/.config/nvim/lua/plugins/tether.lua)".to_string())
            } else {
                (false, "Ikke installert i plugins/tether.lua".to_string())
            }
        }
        NvimPluginManager::Packpath => {
            let pack_path = home.join(".local/share/nvim/site/pack/tether/start/tether.nvim");
            if pack_path.exists() {
                (true, "Installert i packpath (~/.local/share/nvim/site/pack/...)".to_string())
            } else {
                (false, "Ikke installert i packpath".to_string())
            }
        }
        NvimPluginManager::VimPlug | NvimPluginManager::Manual => {
            let init_lua = home.join(".config/nvim/init.lua");
            let init_vim = home.join(".config/nvim/init.vim");
            let in_lua = std::fs::read_to_string(&init_lua).map(|s| s.contains("tether")).unwrap_or(false);
            let in_vim = std::fs::read_to_string(&init_vim).map(|s| s.contains("tether")).unwrap_or(false);
            if in_lua || in_vim {
                (true, "Referanse funnet i init-fil".to_string())
            } else {
                (false, "Ingen referanse funnet i init-fil".to_string())
            }
        }
    }
}

pub fn install_nvim_plugin(manager: NvimPluginManager) -> Result<String, String> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Err("Finner ikke hjemmemappe".to_string()),
    };

    let local_tether_repo = home.join("Projects/tether.nvim");
    let repo_path_str = if local_tether_repo.exists() {
        local_tether_repo.to_string_lossy().to_string()
    } else {
        "https://github.com/kim/tether.nvim".to_string()
    };

    match manager {
        NvimPluginManager::Lazy => {
            let plugins_dir = home.join(".config/nvim/lua/plugins");
            if let Err(e) = std::fs::create_dir_all(&plugins_dir) {
                return Err(format!("Kunne ikke opprette mappe: {}", e));
            }
            let spec_path = plugins_dir.join("tether.lua");
            let content = format!(
                "return {{\n  {{\n    dir = vim.fn.expand(\"{}\"),\n    name = \"tether.nvim\",\n    lazy = false,\n    opts = {{\n      ratio = {{\n        focus = 45,\n        rest = 15,\n      }},\n    }},\n    config = function(_, opts)\n      require(\"tether\").setup(opts)\n    end,\n  }},\n}}\n",
                repo_path_str
            );
            if let Err(e) = std::fs::write(&spec_path, content) {
                return Err(format!("Kunne ikke skrive tether.lua: {}", e));
            }
            Ok("tether.lua ble opprettet i ~/.config/nvim/lua/plugins/!".to_string())
        }
        NvimPluginManager::Packpath => {
            let pack_start = home.join(".local/share/nvim/site/pack/tether/start");
            if let Err(e) = std::fs::create_dir_all(&pack_start) {
                return Err(format!("Kunne ikke opprette pack-mappe: {}", e));
            }
            let target_link = pack_start.join("tether.nvim");
            if target_link.exists() || target_link.is_symlink() {
                let _ = std::fs::remove_file(&target_link);
                let _ = std::fs::remove_dir_all(&target_link);
            }

            #[cfg(unix)]
            {
                if std::os::unix::fs::symlink(&local_tether_repo, &target_link).is_ok() {
                    return Ok("Symlink opprettet i ~/.local/share/nvim/site/pack/tether/start/".to_string());
                }
            }
            Err("Kunne ikke opprette symlink til tether.nvim".to_string())
        }
        NvimPluginManager::VimPlug => {
            Ok("Kopiert: Legg til `Plug 'kim/tether.nvim'` i din init.vim/init.lua".to_string())
        }
        NvimPluginManager::Manual => {
            Ok("Konfigurer din pakkemanager til å peke mot ~/Projects/tether.nvim".to_string())
        }
    }
}

pub fn check_obsidian_plugin_status() -> (bool, String) {
    let vaults = find_obsidian_vaults();
    if vaults.is_empty() {
        return (false, "Fant ingen Obsidian-vault i ~/Documents".to_string());
    }
    let installed_names: Vec<String> = vaults
        .iter()
        .filter(|v| v.join(".obsidian/plugins/tether-flow/main.js").is_file())
        .filter_map(|v| v.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();

    if !installed_names.is_empty() {
        (true, format!("Installert i {}/{} vaults ({})", installed_names.len(), vaults.len(), installed_names.join(", ")))
    } else {
        (false, format!("Ikke installert i {} Obsidian-vault(s)", vaults.len()))
    }
}

pub fn install_obsidian_plugin() -> Result<String, String> {
    let Some(home) = dirs::home_dir() else {
        return Err("Finner ikke hjemmemappe".to_string());
    };
    let source_dir = home.join("Projects/tether-obsidian");
    if !source_dir.join("main.js").is_file() || !source_dir.join("manifest.json").is_file() {
        return Err("Finner ikke ~/Projects/tether-obsidian (main.js / manifest.json)".to_string());
    }

    let vaults = find_obsidian_vaults();
    if vaults.is_empty() {
        return Err("Fant ingen Obsidian-vault (.obsidian-mappe)".to_string());
    }

    let mut installed_in = Vec::new();
    for vault in &vaults {
        for legacy in ["tether-obsidian", "tether"] {
            let legacy_dir = vault.join(".obsidian/plugins").join(legacy);
            if legacy_dir.is_dir() {
                let _ = std::fs::remove_dir_all(&legacy_dir);
            }
        }

        let plugin_dir = vault.join(".obsidian/plugins/tether-flow");
        std::fs::create_dir_all(&plugin_dir).map_err(|e| e.to_string())?;
        for fname in ["manifest.json", "main.js", "styles.css"] {
            let src = source_dir.join(fname);
            if src.is_file() {
                std::fs::copy(&src, plugin_dir.join(fname)).map_err(|e| e.to_string())?;
            }
        }

        let community_path = vault.join(".obsidian/community-plugins.json");
        let mut plugins: Vec<String> = std::fs::read_to_string(&community_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        plugins.retain(|p| p != "tether-obsidian" && p != "tether");
        if !plugins.iter().any(|p| p == "tether-flow") {
            plugins.push("tether-flow".to_string());
        }
        if let Ok(json) = serde_json::to_string_pretty(&plugins) {
            let _ = std::fs::write(&community_path, json);
        }

        if let Some(name) = vault.file_name() {
            installed_in.push(name.to_string_lossy().to_string());
        }
    }

    Ok(format!("Installert Tether Flowmodoro (tether-flow) i: {}. Start Obsidian på nytt (eller reload plugins).", installed_in.join(", ")))
}

pub fn check_vscode_extension_status(editor: EditorType) -> (bool, String) {
    let bin = match editor.resolve().binary_name() {
        Some(b) => b,
        None => "code",
    };

    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new(bin).arg("--list-extensions").output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
                if text.contains("tether") {
                    return (true, "Installert i editor".to_string());
                }
            }
        }
    }

    let home = dirs::home_dir();
    if let Some(h) = home {
        let ext_dir = h.join(".vscode/extensions");
        if ext_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(ext_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if name.contains("tether") {
                        return (true, "Installert i ~/.vscode/extensions/".to_string());
                    }
                }
            }
        }
    }

    (false, "Ikke installert".to_string())
}

pub fn install_vscode_extension(editor: EditorType) -> Result<String, String> {
    let bin = match editor.resolve().binary_name() {
        Some(b) => b,
        None => "code",
    };

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Err("Finner ikke hjemmemappe".to_string()),
    };
    let vsix_cand = home.join("Projects/tether-vscode/tether.vsix");
    if vsix_cand.exists() {
        if let Ok(status) = std::process::Command::new(bin)
            .args(["--install-extension", &vsix_cand.to_string_lossy()])
            .status()
        {
            if status.success() {
                return Ok(format!("Installert Tether-utvidelse i {}!", editor.display_name()));
            }
        }
    }

    Ok(format!("Klargjort for integrasjon med {}.", editor.display_name()))
}

struct UniversalAppDef {
    app_id: &'static str,
    friendly_name: &'static str,
    class_needles: &'static [&'static str],
    extensions: &'static [&'static str],
}

const UNIVERSAL_APPS: &[UniversalAppDef] = &[
    UniversalAppDef {
        app_id: "obsidian",
        friendly_name: "Obsidian",
        class_needles: &["obsidian"],
        extensions: &[".md", ".canvas"],
    },
    UniversalAppDef {
        app_id: "typora",
        friendly_name: "Typora",
        class_needles: &["typora"],
        extensions: &[".md", ".txt"],
    },
    UniversalAppDef {
        app_id: "libreoffice",
        friendly_name: "LibreOffice",
        class_needles: &["libreoffice", "soffice"],
        extensions: &[".odt", ".ods", ".odp", ".docx", ".xlsx", ".pptx", ".rtf"],
    },
    UniversalAppDef {
        app_id: "sioyek",
        friendly_name: "Sioyek PDF",
        class_needles: &["sioyek"],
        extensions: &[".pdf", ".epub"],
    },
    UniversalAppDef {
        app_id: "evince",
        friendly_name: "Evince",
        class_needles: &["evince", "org.gnome.evince"],
        extensions: &[".pdf"],
    },
    UniversalAppDef {
        app_id: "zathura",
        friendly_name: "Zathura",
        class_needles: &["zathura"],
        extensions: &[".pdf", ".epub", ".djvu"],
    },
    UniversalAppDef {
        app_id: "okular",
        friendly_name: "Okular",
        class_needles: &["okular"],
        extensions: &[".pdf", ".epub"],
    },
    UniversalAppDef {
        app_id: "xournalpp",
        friendly_name: "Xournal++",
        class_needles: &["xournalpp"],
        extensions: &[".xopp", ".pdf"],
    },
    UniversalAppDef {
        app_id: "kdenlive",
        friendly_name: "Kdenlive",
        class_needles: &["kdenlive"],
        extensions: &[".kdenlive", ".mp4", ".mkv", ".mov"],
    },
    UniversalAppDef {
        app_id: "omacut",
        friendly_name: "Omacut",
        class_needles: &["omacut"],
        extensions: &[".mp4", ".mkv", ".mov", ".webm"],
    },
    UniversalAppDef {
        app_id: "gimp",
        friendly_name: "GIMP",
        class_needles: &["gimp"],
        extensions: &[".xcf", ".png", ".jpg", ".jpeg", ".webp"],
    },
    UniversalAppDef {
        app_id: "krita",
        friendly_name: "Krita",
        class_needles: &["krita"],
        extensions: &[".kra", ".png", ".psd"],
    },
    UniversalAppDef {
        app_id: "inkscape",
        friendly_name: "Inkscape",
        class_needles: &["inkscape"],
        extensions: &[".svg", ".pdf"],
    },
    UniversalAppDef {
        app_id: "blender",
        friendly_name: "Blender",
        class_needles: &["blender"],
        extensions: &[".blend"],
    },
    UniversalAppDef {
        app_id: "audacity",
        friendly_name: "Audacity",
        class_needles: &["audacity"],
        extensions: &[".aup3", ".wav", ".mp3", ".flac", ".ogg"],
    },
];

fn clean_window_doc_title(title: &str) -> String {
    let mut cleaned = title.trim().to_string();
    for sep in [" — ", " - ", " – "] {
        if let Some((head, _)) = cleaned.rsplit_once(sep) {
            let h = head.trim();
            if !h.is_empty() && h.len() >= 2 {
                // For "Note - Vault - Obsidian 1.15.1", keep Note or strip only final app suffix
                cleaned = h.to_string();
            }
        }
    }
    cleaned
}

fn find_file_in_vault_by_stem(vault: &Path, stem: &str, max_entries: usize) -> Option<PathBuf> {
    let target_md = format!("{}.md", stem).to_lowercase();
    let mut stack = vec![vault.to_path_buf()];
    let mut seen = 0usize;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            seen += 1;
            if seen > max_entries {
                return None;
            }
            let p = entry.path();
            if p.is_dir() {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') {
                    stack.push(p);
                }
            } else if let Some(fname) = p.file_name().and_then(|n| n.to_str()) {
                if fname.to_lowercase() == target_md || fname.eq_ignore_ascii_case(stem) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn resolve_doc_path_from_proc_or_recent(pid: Option<i64>, doc_title: &str, extensions: &[&str]) -> Option<PathBuf> {
    let needle = doc_title.to_lowercase();
    let needle_stem = Path::new(doc_title)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(doc_title)
        .to_lowercase();

    #[cfg(target_os = "linux")]
    if let Some(p) = pid {
        if let Ok(entries) = std::fs::read_dir(format!("/proc/{}/fd", p)) {
            for entry in entries.flatten() {
                if let Ok(target) = std::fs::read_link(entry.path()) {
                    if !target.is_file() {
                        continue;
                    }
                    let s = target.to_string_lossy().to_lowercase();
                    let matches_ext = extensions.iter().any(|ext| s.ends_with(ext));
                    let matches_name = !needle_stem.is_empty() && s.contains(&needle_stem);
                    if matches_ext && (matches_name || needle.is_empty()) {
                        return Some(target);
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = pid;

    if let Some(home) = dirs::home_dir() {
        let xbel = home.join(".local/share/recently-used.xbel");
        if let Ok(content) = std::fs::read_to_string(xbel) {
            let mut best: Option<PathBuf> = None;
            for line in content.lines() {
                if let Some(start) = line.find("href=\"file://") {
                    let rest = &line[start + "href=\"file://".len()..];
                    if let Some(end) = rest.find('"') {
                        let decoded = percent_decode_uri(&rest[..end]);
                        let path = PathBuf::from(&decoded);
                        if !path.is_file() {
                            continue;
                        }
                        let lower = decoded.to_lowercase();
                        if !extensions.iter().any(|ext| lower.ends_with(ext)) {
                            continue;
                        }
                        if !needle_stem.is_empty() && lower.contains(&needle_stem) {
                            best = Some(path);
                        }
                    }
                }
            }
            if best.is_some() {
                return best;
            }
        }
    }

    None
}

/// Captures active document/media/study context from non-editor creative/writing OS windows.
pub fn sync_universal_context_anchor() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
            return;
        }
        let Ok(out) = std::process::Command::new("hyprctl").args(["activewindow", "-j"]).output() else {
            return;
        };
        let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
            return;
        };
        let class = val.get("class").and_then(|c| c.as_str()).unwrap_or("");
        let title = val.get("title").and_then(|t| t.as_str()).unwrap_or("");
        if class.is_empty() && title.is_empty() {
            return;
        }
        let pid = val.get("pid").and_then(|p| p.as_i64());
        let class_l = class.to_lowercase();
        let title_l = title.to_lowercase();

        let Some(app) = UNIVERSAL_APPS.iter().find(|a| {
            a.class_needles
                .iter()
                .any(|n| class_l.contains(n) || title_l.contains(n))
        }) else {
            return;
        };

        let now = chrono::Utc::now().timestamp().max(0) as u64;
        let existing = crate::storage::load_context_anchor();
        if app.app_id == "obsidian" {
            if let Some(ref ex) = existing {
                if ex.server_name == "obsidian" && now.saturating_sub(ex.timestamp) < 5 {
                    return;
                }
            }
        }

        let doc_title = clean_window_doc_title(title);
        if doc_title.is_empty() || doc_title.eq_ignore_ascii_case(app.friendly_name) {
            return;
        }

        let mut resolved_file = resolve_doc_path_from_proc_or_recent(pid, &doc_title, app.extensions);
        let mut vault_name: Option<String> = None;
        let mut vault_path: Option<PathBuf> = None;

        if app.app_id == "obsidian" && resolved_file.is_none() {
            for v in find_obsidian_vaults() {
                if let Some(found) = find_file_in_vault_by_stem(&v, &doc_title, 400) {
                    vault_name = v.file_name().map(|n| n.to_string_lossy().to_string());
                    vault_path = Some(v);
                    resolved_file = Some(found);
                    break;
                }
            }
        }

        let (file_path, project_path, project_name, relative_path) = if let Some(fp) = resolved_file {
            let abs = fp.to_string_lossy().to_string();
            let p_dir = vault_path
                .clone()
                .or_else(|| fp.parent().map(|p| p.to_path_buf()))
                .unwrap_or_default();
            let rel = if let Ok(r) = fp.strip_prefix(&p_dir) {
                r.to_string_lossy().to_string()
            } else {
                fp.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| abs.clone())
            };
            let proj = vault_name
                .or_else(|| Some(app.friendly_name.to_string()))
                .unwrap_or_default();
            (abs, p_dir.to_string_lossy().to_string(), proj, rel)
        } else {
            (
                doc_title.clone(),
                String::new(),
                app.friendly_name.to_string(),
                doc_title,
            )
        };

        let anchor = ContextAnchor {
            timestamp: now,
            project_name,
            project_path,
            branch: String::new(),
            file_path,
            relative_path,
            line: 1,
            col: 0,
            last_note: format!("Aktiv i {}", app.friendly_name),
            server_name: app.app_id.to_string(),
        };

        let should_write = match existing {
            Some(ex) => {
                ex.file_path != anchor.file_path
                    || ex.server_name != anchor.server_name
                    || now.saturating_sub(ex.timestamp) >= 4
            }
            None => true,
        };
        if should_write {
            let _ = crate::storage::save_context_anchor(&anchor);
        }
    }
}

pub fn get_focused_editor() -> Option<EditorType> {
    if let Some(editor) = crate::storage::get_focused_editor_from_state() {
        return Some(editor);
    }

    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            if let Ok(out) = std::process::Command::new("hyprctl").args(["activewindow", "-j"]).output() {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                    let class = val.get("class").and_then(|c| c.as_str()).unwrap_or("").to_lowercase();
                    let title = val.get("title").and_then(|t| t.as_str()).unwrap_or("").to_lowercase();
                    let init_title = val.get("initialTitle").and_then(|t| t.as_str()).unwrap_or("").to_lowercase();

                    if class.contains("code") {
                        return Some(EditorType::VSCode);
                    }
                    if class.contains("vscodium") {
                        return Some(EditorType::VSCodium);
                    }
                    if class.contains("cursor") {
                        return Some(EditorType::Cursor);
                    }
                    if class.contains("neovide") {
                        return Some(EditorType::Neovide);
                    }
                    if class.contains("obsidian") {
                        return Some(EditorType::Obsidian);
                    }
                    if class.contains("typora") {
                        return Some(EditorType::Typora);
                    }
                    if class.contains("libreoffice") || class.contains("soffice") {
                        return Some(EditorType::LibreOffice);
                    }
                    if PDF_BINS.iter().any(|k| class.contains(k)) {
                        return Some(EditorType::PdfReader);
                    }
                    if IMAGE_BINS.iter().any(|k| class.contains(k)) {
                        return Some(EditorType::ImageEditor);
                    }
                    if VIDEO_BINS.iter().any(|k| class.contains(k)) {
                        return Some(EditorType::VideoAudio);
                    }
                    if title.contains("helix") || class.contains("helix") || title.contains("hx") {
                        return Some(EditorType::Helix);
                    }
                    if title.contains("nvim") || title.contains("neovim") || init_title.contains("nvim") {
                        return Some(EditorType::Neovim);
                    }
                    if let Some(pid) = val.get("pid").and_then(|p| p.as_i64()) {
                        if let Ok(pgrep) = std::process::Command::new("pgrep").args(["-P", &pid.to_string(), "nvim"]).output() {
                            if !pgrep.stdout.is_empty() {
                                return Some(EditorType::Neovim);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn is_editor_focused() -> bool {
    get_focused_editor().is_some()
}
