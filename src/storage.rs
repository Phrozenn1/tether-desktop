use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

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

pub fn append_impulse(text: &str) -> Result<(), std::io::Error> {
    let dir = ensure_data_dir()?;
    let file_path = dir.join("impulses.md");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M");
    writeln!(file, "- [{}] {}", timestamp, text)?;
    Ok(())
}

pub fn load_impulses() -> Vec<String> {
    let dir = get_data_dir();
    let file_path = dir.join("impulses.md");
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
