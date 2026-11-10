use serde::{Deserialize, Serialize};
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
    #[serde(rename = "custom")]
    Custom,
}

impl Default for EditorType {
    fn default() -> Self {
        EditorType::Auto
    }
}

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

pub fn binary_running(bin: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("pgrep").args(["-x", bin]).output() {
            if !output.stdout.is_empty() {
                return true;
            }
        }
    }
    false
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
            EditorType::Custom => "Egendefinert",
        }
    }

    pub fn binary_name(&self) -> Option<&'static str> {
        match self {
            EditorType::Auto | EditorType::None | EditorType::Custom => None,
            EditorType::Neovim => Some("nvim"),
            EditorType::Neovide => Some("neovide"),
            EditorType::VSCode => Some("code"),
            EditorType::VSCodium => Some("codium"),
            EditorType::Cursor => Some("cursor"),
            EditorType::Helix => Some("hx"),
        }
    }

    pub fn is_installed(&self) -> bool {
        match self.binary_name() {
            Some(bin) => binary_exists(bin),
            None => true,
        }
    }

    pub fn detected_editors() -> Vec<EditorType> {
        let candidates = [
            EditorType::Neovim,
            EditorType::VSCode,
            EditorType::VSCodium,
            EditorType::Cursor,
            EditorType::Neovide,
            EditorType::Helix,
        ];
        candidates
            .into_iter()
            .filter(|e| e.binary_name().is_some() && e.is_installed())
            .collect()
    }

    /// Resolves `Auto` to the most relevant installed editor (or `None`), factoring in anchor & running processes.
    pub fn resolve_context(&self, anchor: Option<&ContextAnchor>) -> EditorType {
        match self {
            EditorType::Auto => {
                // 1. If anchor is available, prioritize the editor that created the anchor
                if let Some(a) = anchor {
                    let s = a.server_name.to_lowercase();
                    if s.contains("vscode") || s.contains("code") {
                        if binary_exists("code") {
                            return EditorType::VSCode;
                        }
                    } else if s.contains("codium") {
                        if binary_exists("codium") {
                            return EditorType::VSCodium;
                        }
                    } else if s.contains("cursor") {
                        if binary_exists("cursor") {
                            return EditorType::Cursor;
                        }
                    } else if s.contains("neovide") {
                        if binary_exists("neovide") {
                            return EditorType::Neovide;
                        }
                    } else if s.contains("nvim") || s.contains("neovim") {
                        if binary_exists("nvim") {
                            return EditorType::Neovim;
                        }
                    } else if s.contains("helix") || s.contains("hx") {
                        if binary_exists("hx") {
                            return EditorType::Helix;
                        }
                    }
                }

                // 2. Check which editor is currently running
                if let Some(running) = Self::detect_running_editor() {
                    return running;
                }

                // 3. Fallback based on installed binaries
                if binary_exists("nvim") {
                    EditorType::Neovim
                } else if binary_exists("code") {
                    EditorType::VSCode
                } else if binary_exists("neovide") {
                    EditorType::Neovide
                } else if binary_exists("cursor") {
                    EditorType::Cursor
                } else if binary_exists("codium") {
                    EditorType::VSCodium
                } else if binary_exists("hx") {
                    EditorType::Helix
                } else {
                    EditorType::None
                }
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
        None
    }

    pub fn is_running(&self) -> bool {
        let active = self.resolve();
        let bin = match active.binary_name() {
            Some(b) => b,
            None => return false,
        };
        binary_running(bin)
    }

    pub fn short_badge(&self) -> &'static str {
        match self.resolve() {
            EditorType::Neovim => "NVIM",
            EditorType::Neovide => "Neovide",
            EditorType::VSCode => "VS Code",
            EditorType::VSCodium => "Codium",
            EditorType::Cursor => "Cursor",
            EditorType::Helix => "Helix",
            EditorType::Custom => "Editor",
            _ => "",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self.resolve() {
            EditorType::Neovim | EditorType::Helix => "utilities-terminal-symbolic",
            _ => "text-editor-symbolic",
        }
    }

    pub fn resume_button_label(&self, lang: crate::storage::Language) -> String {
        let active = self.resolve();
        let editor_name = match active {
            EditorType::Neovim => "Neovim",
            EditorType::Neovide => "Neovide",
            EditorType::VSCode => "VS Code",
            EditorType::VSCodium => "VSCodium",
            EditorType::Cursor => "Cursor",
            EditorType::Helix => "Helix",
            EditorType::Custom => match lang {
                crate::storage::Language::Norwegian => "editor",
                _ => "editor",
            },
            EditorType::Auto | EditorType::None => match lang {
                crate::storage::Language::Norwegian => "fil",
                _ => "file",
            },
        };

        match lang {
            crate::storage::Language::Norwegian => format!("Gjenoppta i {} 🚀", editor_name),
            _ => format!("Resume in {} 🚀", editor_name),
        }
    }

    /// Return available choices for the Settings ComboRow.
    pub fn available_choices() -> Vec<EditorType> {
        let mut list = vec![EditorType::Auto, EditorType::None];
        let candidates = [
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
                let _ = std::process::Command::new("code")
                    .args(["-g", &target])
                    .spawn();
            }
            EditorType::VSCodium => {
                let target = format!("{}:{}", anchor.file_path, anchor.line);
                let _ = std::process::Command::new("codium")
                    .args(["-g", &target])
                    .spawn();
            }
            EditorType::Cursor => {
                let target = format!("{}:{}", anchor.file_path, anchor.line);
                let _ = std::process::Command::new("cursor")
                    .args(["-g", &target])
                    .spawn();
            }
            EditorType::Helix => {
                let parts: Vec<&str> = terminal_cmd.split_whitespace().collect();
                if !parts.is_empty() {
                    let mut args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
                    args.push(format!("{}:{}", anchor.file_path, anchor.line));
                    let _ = std::process::Command::new(parts[0]).args(&args).spawn();
                }
            }
            _ => {}
        }
    }
}

pub fn focus_or_spawn_window(cmd: &[&str], match_name: &str) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("hyprctl").args(["clients", "-j"]).output() {
            if let Ok(clients) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(arr) = clients.as_array() {
                    let client = arr.iter().find(|c| {
                        let class = c["class"].as_str().unwrap_or("").to_lowercase();
                        let title = c["title"].as_str().unwrap_or("").to_lowercase();
                        class.contains(match_name) || title.contains(match_name)
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
                (true, "🟢 Installert (i ~/.config/nvim/lua/plugins/tether.lua)".to_string())
            } else {
                (false, "⚠️ Ikke installert i plugins/tether.lua".to_string())
            }
        }
        NvimPluginManager::Packpath => {
            let pack_path = home.join(".local/share/nvim/site/pack/tether/start/tether.nvim");
            if pack_path.exists() {
                (true, "🟢 Installert i packpath (~/.local/share/nvim/site/pack/...)".to_string())
            } else {
                (false, "⚠️ Ikke installert i packpath".to_string())
            }
        }
        NvimPluginManager::VimPlug | NvimPluginManager::Manual => {
            let init_lua = home.join(".config/nvim/init.lua");
            let init_vim = home.join(".config/nvim/init.vim");
            let in_lua = std::fs::read_to_string(&init_lua).map(|s| s.contains("tether")).unwrap_or(false);
            let in_vim = std::fs::read_to_string(&init_vim).map(|s| s.contains("tether")).unwrap_or(false);
            if in_lua || in_vim {
                (true, "🟢 Referanse funnet i init-fil".to_string())
            } else {
                (false, "⚠️ Ingen referanse funnet i init-fil".to_string())
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
                    return (true, "🟢 Installert i editor".to_string());
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
                        return (true, "🟢 Installert i ~/.vscode/extensions/".to_string());
                    }
                }
            }
        }
    }

    (false, "⚠️ Ikke installert".to_string())
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

pub fn get_focused_editor() -> Option<EditorType> {
    // 1. Primary: Platform-agnostic focus reported directly by editor integration (Neovim, VS Code, etc.)
    // Works universally across all desktop environments (GNOME, KDE, Hyprland, Sway, Windows, macOS).
    if let Some(editor) = crate::storage::get_focused_editor_from_state() {
        return Some(editor);
    }

    // 2. Secondary fallback on Hyprland: query activewindow
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
