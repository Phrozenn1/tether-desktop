pub use tether_core::storage::*;

pub fn pick_impulse_file() -> Option<std::path::PathBuf> {
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
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(std::path::PathBuf::from(path));
                }
            }
        }
    }

    None
}
