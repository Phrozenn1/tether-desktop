use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_API_LATEST_RELEASE: &str =
    "https://api.github.com/repos/Phrozenn1/tether-desktop/releases/latest";
pub const GITHUB_RELEASES_PAGE: &str =
    "https://github.com/Phrozenn1/tether-desktop/releases/latest";
pub const UPDATE_CHECK_INTERVAL_SECS: u64 = 6 * 3600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateCheckInfo {
    pub current_version: String,
    pub latest_version: String,
    pub tag_name: String,
    pub release_name: String,
    pub html_url: String,
    pub published_at: String,
    pub update_available: bool,
    pub dismissed: bool,
    pub checked_at_epoch: u64,
}

impl Default for UpdateCheckInfo {
    fn default() -> Self {
        Self {
            current_version: CURRENT_VERSION.to_string(),
            latest_version: String::new(),
            tag_name: String::new(),
            release_name: String::new(),
            html_url: GITHUB_RELEASES_PAGE.to_string(),
            published_at: String::new(),
            update_available: false,
            dismissed: false,
            checked_at_epoch: 0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateCheckCache {
    #[serde(default)]
    pub last_info: Option<UpdateCheckInfo>,
    #[serde(default)]
    pub notified_version: String,
    #[serde(default)]
    pub dismissed_version: String,
}

fn cache_file_path() -> PathBuf {
    crate::storage::get_data_dir().join("update_check.json")
}

fn load_cache() -> UpdateCheckCache {
    let path = cache_file_path();
    let Ok(raw) = std::fs::read_to_string(path) else {
        return UpdateCheckCache::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_cache(cache: &UpdateCheckCache) -> Result<(), String> {
    let path = cache_file_path();
    if let Some(parent) = path.parent() {
        create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(cache).map_err(|e| e.to_string())?;
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())
}

fn now_unix_secs() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}

pub fn parse_semver(ver: &str) -> Option<(u32, u32, u32)> {
    let trimmed = ver.trim().trim_start_matches(['v', 'V']);
    if trimmed.is_empty() || trimmed.contains('-') {
        return None;
    }
    let core = trimmed.split('+').next()?.trim();
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    let patch = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

pub fn is_newer_version(current: &str, candidate: &str) -> bool {
    match (parse_semver(current), parse_semver(candidate)) {
        (Some(cur), Some(cand)) => cand > cur,
        _ => false,
    }
}

pub fn parse_github_release_json(
    raw_json: &str,
    current_version: &str,
    now_epoch: u64,
) -> Result<UpdateCheckInfo, String> {
    let val: serde_json::Value =
        serde_json::from_str(raw_json).map_err(|e| format!("Invalid GitHub JSON: {e}"))?;

    let is_draft = val.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_prerelease = val
        .get("prerelease")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let tag_name = val
        .get("tag_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
                format!("GitHub API: {msg}")
            } else {
                "GitHub release response is missing tag_name".to_string()
            }
        })?;

    let release_name = val
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&tag_name)
        .to_string();

    let html_url = val
        .get("html_url")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(GITHUB_RELEASES_PAGE)
        .to_string();

    let published_at = val
        .get("published_at")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("")
        .to_string();

    let normalized_current = current_version.trim().trim_start_matches(['v', 'V']);
    let latest_version = tag_name.trim_start_matches(['v', 'V']).to_string();
    let update_available =
        !is_draft && !is_prerelease && is_newer_version(normalized_current, &tag_name);

    Ok(UpdateCheckInfo {
        current_version: normalized_current.to_string(),
        latest_version,
        tag_name,
        release_name,
        html_url,
        published_at,
        update_available,
        dismissed: false,
        checked_at_epoch: now_epoch,
    })
}

fn fetch_latest_release_json() -> Result<String, String> {
    let curl_out = Command::new("curl")
        .args([
            "-sSL",
            "--max-time",
            "5",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: tether-desktop",
            GITHUB_API_LATEST_RELEASE,
        ])
        .output();

    if let Ok(out) = curl_out {
        if out.status.success() && !out.stdout.is_empty() {
            return Ok(String::from_utf8_lossy(&out.stdout).to_string());
        }
        #[cfg(not(target_os = "windows"))]
        {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(if err.is_empty() {
                format!("curl exited with {}", out.status)
            } else {
                err
            });
        }
    }

    #[cfg(target_os = "windows")]
    {
        let ps_cmd = format!(
            "$ProgressPreference='SilentlyContinue'; Invoke-RestMethod -Uri '{}' -Headers @{{'Accept'='application/vnd.github+json';'User-Agent'='tether-desktop'}} -TimeoutSec 5 | ConvertTo-Json -Depth 4",
            GITHUB_API_LATEST_RELEASE
        );
        let ps_out = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
            .output()
            .map_err(|e| format!("Failed to invoke curl or powershell: {e}"))?;
        if ps_out.status.success() && !ps_out.stdout.is_empty() {
            return Ok(String::from_utf8_lossy(&ps_out.stdout).to_string());
        }
        let err = String::from_utf8_lossy(&ps_out.stderr).trim().to_string();
        return Err(if err.is_empty() {
            format!("powershell exited with {}", ps_out.status)
        } else {
            err
        });
    }

    #[allow(unreachable_code)]
    Err("Failed to execute curl for update check".to_string())
}

fn sync_cached_info(mut info: UpdateCheckInfo, dismissed_version: &str) -> UpdateCheckInfo {
    info.current_version = CURRENT_VERSION.to_string();
    info.update_available = is_newer_version(CURRENT_VERSION, &info.latest_version);
    info.dismissed = !dismissed_version.is_empty() && dismissed_version == info.latest_version;
    info
}

pub fn load_cached_status() -> UpdateCheckInfo {
    let cache = load_cache();
    match cache.last_info {
        Some(info) => sync_cached_info(info, &cache.dismissed_version),
        None => UpdateCheckInfo::default(),
    }
}

pub fn check_for_updates(force: bool) -> Result<UpdateCheckInfo, String> {
    let mut cache = load_cache();
    let now = now_unix_secs();

    if !force {
        if let Some(ref cached) = cache.last_info {
            if now.saturating_sub(cached.checked_at_epoch) < UPDATE_CHECK_INTERVAL_SECS
                && cached.current_version == CURRENT_VERSION
            {
                return Ok(sync_cached_info(cached.clone(), &cache.dismissed_version));
            }
        }
    }

    let raw_json = fetch_latest_release_json()?;
    let mut info = parse_github_release_json(&raw_json, CURRENT_VERSION, now)?;
    info.dismissed =
        !cache.dismissed_version.is_empty() && cache.dismissed_version == info.latest_version;

    cache.last_info = Some(info.clone());
    let _ = save_cache(&cache);
    Ok(info)
}

pub fn mark_version_notified(version: &str) -> bool {
    let normalized = version.trim().trim_start_matches(['v', 'V']);
    if normalized.is_empty() {
        return false;
    }
    let mut cache = load_cache();
    if cache.notified_version == normalized {
        return false;
    }
    cache.notified_version = normalized.to_string();
    let _ = save_cache(&cache);
    true
}

pub fn dismiss_version(version: &str) {
    let normalized = version.trim().trim_start_matches(['v', 'V']);
    if normalized.is_empty() {
        return;
    }
    let mut cache = load_cache();
    cache.dismissed_version = normalized.to_string();
    if let Some(ref mut info) = cache.last_info {
        if info.latest_version == normalized {
            info.dismissed = true;
        }
    }
    let _ = save_cache(&cache);
}

pub fn open_release_url(url: &str) {
    let trimmed = url.trim();
    let target = if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        trimmed
    } else {
        GITHUB_RELEASES_PAGE
    };

    #[cfg(target_os = "linux")]
    let _ = Command::new("xdg-open")
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    #[cfg(target_os = "macos")]
    let _ = Command::new("open")
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd")
        .args(["/C", "start", "", target])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("1.5.0"), Some((1, 5, 0)));
        assert_eq!(parse_semver("v1.5.2"), Some((1, 5, 2)));
        assert_eq!(parse_semver(" V2.0.10+build.5 "), Some((2, 0, 10)));
        assert_eq!(parse_semver("v1.6.0-beta.1"), None);
        assert_eq!(parse_semver("1.5"), None);
        assert_eq!(parse_semver("invalid"), None);
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.5.0", "v1.5.1"));
        assert!(is_newer_version("1.5.0", "1.6.0"));
        assert!(is_newer_version("v1.5.0", "v2.0.0"));
        assert!(!is_newer_version("1.5.0", "v1.5.0"));
        assert!(!is_newer_version("1.5.0", "v1.4.9"));
        assert!(!is_newer_version("1.5.0", "v1.6.0-rc.1"));
    }

    #[test]
    fn test_parse_github_release_json_variants() {
        let newer_json = r#"{
            "tag_name": "v1.6.0",
            "name": "Tether v1.6.0",
            "html_url": "https://github.com/Phrozenn1/tether-desktop/releases/tag/v1.6.0",
            "published_at": "2026-11-15T10:00:00Z",
            "draft": false,
            "prerelease": false
        }"#;
        let info = parse_github_release_json(newer_json, "1.5.0", 12345).unwrap();
        assert!(info.update_available);
        assert_eq!(info.latest_version, "1.6.0");
        assert_eq!(info.tag_name, "v1.6.0");
        assert_eq!(info.release_name, "Tether v1.6.0");
        assert_eq!(info.checked_at_epoch, 12345);

        let same_info = parse_github_release_json(newer_json, "1.6.0", 12345).unwrap();
        assert!(!same_info.update_available);

        let older_info = parse_github_release_json(newer_json, "1.7.0", 12345).unwrap();
        assert!(!older_info.update_available);

        let prerelease_json = r#"{
            "tag_name": "v1.6.0",
            "name": "Tether v1.6.0-pre",
            "html_url": "https://github.com/Phrozenn1/tether-desktop/releases/tag/v1.6.0",
            "published_at": "2026-11-15T10:00:00Z",
            "draft": false,
            "prerelease": true
        }"#;
        let pre_info = parse_github_release_json(prerelease_json, "1.5.0", 12345).unwrap();
        assert!(!pre_info.update_available);

        let draft_json = r#"{
            "tag_name": "v1.6.0",
            "draft": true,
            "prerelease": false
        }"#;
        let draft_info = parse_github_release_json(draft_json, "1.5.0", 12345).unwrap();
        assert!(!draft_info.update_available);
    }
}
