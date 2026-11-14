use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::storage::{
    self, AiAssistMode, ContextAnchor, Language, ShutdownRecord, TetherSettings,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub project_name: String,
    pub project_path: String,
    pub branch: String,
    pub file_rel: String,
    pub line: usize,
    pub enclosing_symbol: Option<String>,
    pub changed_files: Vec<String>,
    pub changed_count: usize,
    pub diff_stat_summary: Option<String>,
    pub recent_commits: Vec<String>,
    pub code_excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamDumpSuggestion {
    pub now: String,
    pub status: String,
    pub next_step: String,
    pub source_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereWasICard {
    pub absence_text: String,
    pub is_after_long_absence: bool,
    pub headline: String,
    pub bullet_last_state: String,
    pub bullet_git: String,
    pub bullet_impulses: String,
    pub suggested_first_step: String,
}

fn find_enclosing_symbol(lines: &[&str], target_line_1based: usize) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let max_idx = target_line_1based.saturating_sub(1).min(lines.len() - 1);
    let markers = [
        "pub async fn ",
        "pub fn ",
        "async fn ",
        "fn ",
        "def ",
        "async def ",
        "function ",
        "export function ",
        "export async function ",
        "class ",
        "impl ",
        "pub struct ",
        "struct ",
        "pub enum ",
        "enum ",
    ];

    for idx in (0..=max_idx).rev() {
        let trimmed = lines[idx].trim_start();
        for md in ["### ", "## ", "# "] {
            if let Some(rest) = trimmed.strip_prefix(md) {
                let heading: String = rest.trim().chars().take(48).collect();
                if !heading.is_empty() {
                    return Some(heading);
                }
            }
        }
        for tex in ["\\subsection{", "\\section{"] {
            if let Some(rest) = trimmed.strip_prefix(tex) {
                let heading: String = rest.split('}').next().unwrap_or("").trim().chars().take(48).collect();
                if !heading.is_empty() {
                    return Some(heading);
                }
            }
        }
        for marker in markers {
            if let Some(rest) = trimmed.strip_prefix(marker) {
                let sig: String = rest
                    .split(&['(', '{', '<', ':'][..])
                    .next()
                    .unwrap_or("")
                    .trim()
                    .chars()
                    .take(48)
                    .collect();
                if !sig.is_empty() {
                    let prefix = marker.trim_end();
                    return Some(format!("{} {}", prefix, sig));
                }
            }
        }
    }
    None
}

fn extract_code_excerpt(file_path: &str, line_1based: usize) -> (Option<String>, String) {
    let Ok(content) = std::fs::read_to_string(file_path) else {
        return (None, String::new());
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (None, String::new());
    }

    let enclosing = find_enclosing_symbol(&lines, line_1based);
    let center = line_1based.saturating_sub(1).min(lines.len() - 1);
    let start = center.saturating_sub(12);
    let end = (center + 13).min(lines.len());

    let mut excerpt = String::new();
    for (offset, l) in lines[start..end].iter().enumerate() {
        let line_no = start + offset + 1;
        excerpt.push_str(&format!("{:>4}: {}\n", line_no, l));
    }

    (enclosing, excerpt)
}

fn run_git(path: &str, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn gather_project_snapshot(anchor: Option<&ContextAnchor>) -> ProjectSnapshot {
    let mut snap = ProjectSnapshot::default();

    let resolved_path: Option<String> = anchor
        .and_then(|a| {
            if !a.project_path.is_empty() && Path::new(&a.project_path).is_dir() {
                Some(a.project_path.clone())
            } else {
                None
            }
        })
        .or_else(|| {
            std::env::current_dir().ok().and_then(|cwd| {
                if cwd.join(".git").exists() {
                    Some(cwd.to_string_lossy().to_string())
                } else {
                    None
                }
            })
        });

    if let Some(a) = anchor {
        snap.project_name = a.project_name.clone();
        snap.branch = a.branch.clone();
        snap.file_rel = if !a.relative_path.is_empty() {
            a.relative_path.clone()
        } else {
            a.file_path.clone()
        };
        snap.line = a.line.max(1);

        if !a.file_path.is_empty() && Path::new(&a.file_path).is_file() {
            let (sym, excerpt) = extract_code_excerpt(&a.file_path, snap.line);
            snap.enclosing_symbol = sym;
            snap.code_excerpt = excerpt;
        }
    }

    if let Some(proj_path) = resolved_path {
        snap.project_path = proj_path.clone();
        if snap.project_name.is_empty() {
            snap.project_name = Path::new(&proj_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| proj_path.clone());
        }

        if snap.branch.is_empty() {
            if let Some(b) = run_git(&proj_path, &["branch", "--show-current"]) {
                let b = b.trim().to_string();
                if !b.is_empty() {
                    snap.branch = b;
                }
            }
        }

        if let Some(status_out) = run_git(&proj_path, &["status", "--porcelain"]) {
            let files: Vec<String> = status_out
                .lines()
                .filter_map(|l| {
                    l.get(3..)
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                })
                .collect();
            snap.changed_count = files.len();
            snap.changed_files = files.into_iter().take(4).collect();
        }

        if let Some(diff_stat) = run_git(&proj_path, &["diff", "--stat", "HEAD"]) {
            if let Some(last) = diff_stat.lines().last() {
                let trimmed = last.trim();
                if !trimmed.is_empty() {
                    snap.diff_stat_summary = Some(trimmed.to_string());
                }
            }
        }

        if let Some(log_out) = run_git(&proj_path, &["log", "-n", "3", "--oneline"]) {
            snap.recent_commits = log_out
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
        }
    }

    snap
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkDomain {
    WritingNotes,
    StudyPdf,
    CreativeMedia,
    Coding,
}

fn detect_work_domain(snap: &ProjectSnapshot) -> WorkDomain {
    let f = snap.file_rel.to_lowercase();
    let p = snap.project_name.to_lowercase();
    let is_obs_app = p == "obs" || p.starts_with("obs ") || p.contains("obs studio");

    if f.ends_with(".pdf") || f.ends_with(".epub") {
        WorkDomain::StudyPdf
    } else if [
        ".kdenlive", ".blend", ".xcf", ".kra", ".svg", ".aup3", ".mp4", ".mkv", ".mov", ".png",
        ".jpg", ".webp", ".wav", ".flac",
    ]
    .iter()
    .any(|ext| f.ends_with(ext))
    {
        WorkDomain::CreativeMedia
    } else if [".md", ".txt", ".odt", ".docx", ".rtf", ".tex", ".org", ".xopp"]
        .iter()
        .any(|ext| f.ends_with(ext))
    {
        WorkDomain::WritingNotes
    } else if ["sioyek", "evince", "zathura", "okular"].iter().any(|k| p.contains(k)) {
        WorkDomain::StudyPdf
    } else if ["obsidian", "libreoffice", "typora", "xournal"]
        .iter()
        .any(|k| p.contains(k))
    {
        WorkDomain::WritingNotes
    } else if is_obs_app
        || ["kdenlive", "omacut", "gimp", "krita", "inkscape", "blender", "audacity"]
            .iter()
            .any(|k| p.contains(k))
    {
        WorkDomain::CreativeMedia
    } else {
        WorkDomain::Coding
    }
}

pub fn heuristic_ramdump_from_snapshot(snap: &ProjectSnapshot, lang: Language) -> RamDumpSuggestion {
    let is_no = lang == Language::Norwegian;
    let domain = detect_work_domain(snap);
    let has_git_context = snap.changed_count > 0 || !snap.recent_commits.is_empty();

    let project_label = if snap.project_name.is_empty() {
        if is_no { "arbeidsøkten".to_string() } else { "the session".to_string() }
    } else if snap.branch.is_empty() {
        snap.project_name.clone()
    } else {
        format!("{} ({})", snap.project_name, snap.branch)
    };

    let sym_part = snap
        .enclosing_symbol
        .as_ref()
        .map(|s| format!(" ({})", s))
        .unwrap_or_default();

    let loc_str = if snap.line > 1 {
        format!("{}:{}", snap.file_rel, snap.line)
    } else {
        snap.file_rel.clone()
    };

    let now = if !snap.file_rel.is_empty() {
        if is_no {
            format!("Jobber i {} — {}{}", project_label, loc_str, sym_part)
        } else {
            format!("Working in {} — {}{}", project_label, loc_str, sym_part)
        }
    } else if !snap.changed_files.is_empty() {
        let preview = snap.changed_files.join(", ");
        if is_no {
            format!("Endringer i {}: {}", project_label, preview)
        } else {
            format!("Editing {}: {}", project_label, preview)
        }
    } else if is_no {
        format!("Arbeider i {}", project_label)
    } else {
        format!("Working in {}", project_label)
    };

    let commit_part = snap
        .recent_commits
        .first()
        .map(|c| {
            if is_no {
                format!("Siste commit: {}", c)
            } else {
                format!("Latest commit: {}", c)
            }
        })
        .unwrap_or_default();

    let status = if snap.changed_count > 0 {
        let files_preview = snap.changed_files.join(", ");
        let stat_suffix = snap
            .diff_stat_summary
            .as_ref()
            .map(|s| format!(" [{}]", s))
            .unwrap_or_default();
        if is_no {
            if commit_part.is_empty() {
                format!("{} ucommitede fil(er): {}{}", snap.changed_count, files_preview, stat_suffix)
            } else {
                format!("{} ucommitede fil(er): {}{}. {}", snap.changed_count, files_preview, stat_suffix, commit_part)
            }
        } else if commit_part.is_empty() {
            format!("{} uncommitted file(s): {}{}", snap.changed_count, files_preview, stat_suffix)
        } else {
            format!("{} uncommitted file(s): {}{}. {}", snap.changed_count, files_preview, stat_suffix, commit_part)
        }
    } else if !commit_part.is_empty() {
        if is_no {
            format!("Ren arbeidskopi. {}", commit_part)
        } else {
            format!("Clean working tree. {}", commit_part)
        }
    } else if let Some(sym) = &snap.enclosing_symbol {
        if is_no {
            format!("Aktiv i {} • {}", project_label, sym)
        } else {
            format!("Active in {} • {}", project_label, sym)
        }
    } else if !snap.file_rel.is_empty() {
        if is_no {
            format!("Aktiv i {} • {}", project_label, snap.file_rel)
        } else {
            format!("Active in {} • {}", project_label, snap.file_rel)
        }
    } else if is_no {
        format!("Aktiv økt i {}", project_label)
    } else {
        format!("Active session in {}", project_label)
    };

    let next_step = if !snap.file_rel.is_empty() {
        if has_git_context {
            let other_file = snap.changed_files.iter().find(|f| *f != &snap.file_rel);
            match (other_file, is_no) {
                (Some(other), true) => format!(
                    "Fortsett i {}:{} og ferdigstill endringene i {}",
                    snap.file_rel,
                    snap.line.max(1),
                    other
                ),
                (Some(other), false) => format!(
                    "Resume {}:{} and finish pending changes in {}",
                    snap.file_rel,
                    snap.line.max(1),
                    other
                ),
                (None, true) => format!("Fortsett i {}:{} og verifiser med tester", snap.file_rel, snap.line.max(1)),
                (None, false) => format!("Resume {}:{} and verify with tests", snap.file_rel, snap.line.max(1)),
            }
        } else {
            match (domain, is_no) {
                (WorkDomain::WritingNotes, true) => {
                    format!("Skriv eller ferdigstill neste avsnitt i {}", snap.file_rel)
                }
                (WorkDomain::WritingNotes, false) => {
                    format!("Write or refine next paragraph in {}", snap.file_rel)
                }
                (WorkDomain::StudyPdf, true) => {
                    format!("Les neste avsnitt i {} og noter ett hovedpunkt", snap.file_rel)
                }
                (WorkDomain::StudyPdf, false) => {
                    format!("Read next section in {} and note one key takeaway", snap.file_rel)
                }
                (WorkDomain::CreativeMedia, true) => {
                    format!("Fortsett redigeringen av {} i {}", snap.file_rel, project_label)
                }
                (WorkDomain::CreativeMedia, false) => {
                    format!("Continue editing {} in {}", snap.file_rel, project_label)
                }
                (WorkDomain::Coding, true) => {
                    format!("Fortsett i {} og ta ett lite steg videre", loc_str)
                }
                (WorkDomain::Coding, false) => {
                    format!("Resume {} and complete one small step", loc_str)
                }
            }
        }
    } else if let Some(first_changed) = snap.changed_files.first() {
        if is_no {
            format!("Åpne {} og fullfør påbegynte endringer", first_changed)
        } else {
            format!("Open {} and complete pending changes", first_changed)
        }
    } else if is_no {
        format!("Velg ett lite steg i {} og ta en 5m mikrostart", project_label)
    } else {
        format!("Pick one small step in {} and start a 5m session", project_label)
    };

    let source_label = if has_git_context {
        if is_no { "Git- og kodeanalyse" } else { "Git & code analysis" }
    } else if is_no {
        "Kontekst- og dokumentanalyse"
    } else {
        "Context & document analysis"
    };

    RamDumpSuggestion {
        now,
        status,
        next_step,
        source_label: source_label.to_string(),
    }
}

struct CliProviderSpec {
    mode: AiAssistMode,
    display_name: &'static str,
    source_label: &'static str,
    mise_rel_paths: &'static [&'static str],
    bin_names: &'static [&'static str],
}

const CLI_PROVIDERS: &[CliProviderSpec] = &[
    CliProviderSpec {
        mode: AiAssistMode::Antigravity,
        display_name: "Antigravity (agy)",
        source_label: "Antigravity (agy)",
        mise_rel_paths: &["antigravity-cli/latest/agy", "antigravity-cli/latest/antigravity"],
        bin_names: &["agy", "antigravity"],
    },
    CliProviderSpec {
        mode: AiAssistMode::Copilot,
        display_name: "GitHub Copilot",
        source_label: "GitHub Copilot",
        mise_rel_paths: &["copilot/latest/copilot"],
        bin_names: &["copilot"],
    },
    CliProviderSpec {
        mode: AiAssistMode::Gemini,
        display_name: "Gemini CLI",
        source_label: "Gemini CLI",
        mise_rel_paths: &["gemini/latest/node_modules/.bin/gemini"],
        bin_names: &["gemini"],
    },
    CliProviderSpec {
        mode: AiAssistMode::Codex,
        display_name: "Codex",
        source_label: "OpenAI Codex",
        mise_rel_paths: &["codex/latest/bin/codex"],
        bin_names: &["codex"],
    },
    CliProviderSpec {
        mode: AiAssistMode::OpenCode,
        display_name: "OpenCode",
        source_label: "OpenCode",
        mise_rel_paths: &["opencode/latest/node_modules/.bin/opencode"],
        bin_names: &["opencode"],
    },
    CliProviderSpec {
        mode: AiAssistMode::Claude,
        display_name: "Claude CLI",
        source_label: "Claude CLI",
        mise_rel_paths: &["claude/latest/bin/claude"],
        bin_names: &["claude"],
    },
];

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn find_binary(mise_rel_paths: &[&str], bin_names: &[&str]) -> Option<PathBuf> {
    let home = home_dir();

    if let Some(ref h) = home {
        let mise_root = h.join(".local/share/mise/installs");
        for rel in mise_rel_paths {
            let cand = mise_root.join(rel);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }

    let mut search_dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();

    if let Some(ref h) = home {
        search_dirs.push(h.join(".local/bin"));
        search_dirs.push(h.join(".local/share/mise/shims"));
        search_dirs.push(h.join(".gemini/antigravity-cli/bin"));
    }
    search_dirs.push(PathBuf::from("/usr/local/bin"));
    search_dirs.push(PathBuf::from("/usr/bin"));

    for dir in search_dirs {
        for bin in bin_names {
            let cand = dir.join(bin);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }

    None
}

fn build_cli_env_path() -> Option<std::ffi::OsString> {
    let mut dirs = Vec::new();
    if let Some(h) = home_dir() {
        dirs.push(h.join(".local/share/mise/installs/node/latest/bin"));
        dirs.push(h.join(".local/bin"));
    }
    if let Some(existing) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(dirs).ok()
}

/// Returns display names of installed local or CLI AI tools usable for RAM-dump suggestions.
pub fn detect_installed_ai_tools() -> Vec<String> {
    let mut tools = Vec::new();
    for spec in CLI_PROVIDERS {
        if find_binary(spec.mise_rel_paths, spec.bin_names).is_some() {
            tools.push(spec.display_name.to_string());
        }
    }
    if find_binary(&[], &["ollama"]).is_some() {
        tools.push("Ollama".to_string());
    }
    tools
}

fn parse_ai_ramdump_lines(response_text: &str, model_label: String) -> Option<RamDumpSuggestion> {
    let mut now = String::new();
    let mut status = String::new();
    let mut next_step = String::new();

    for line in response_text.lines() {
        let cleaned = line
            .trim_start_matches(|c: char| matches!(c, '-' | '*' | '#' | '>' | '`' | '_' | ' ' | '\t'));
        let Some((raw_label, raw_val)) = cleaned.split_once(':') else {
            continue;
        };

        let norm_label: String = raw_label
            .chars()
            .filter(|c| !matches!(c, '[' | ']' | '*' | '_' | '`'))
            .collect::<String>()
            .trim()
            .to_ascii_uppercase();

        let val = raw_val
            .trim()
            .trim_start_matches(|c: char| matches!(c, '*' | '_' | ' ' | '\t'))
            .trim()
            .to_string();

        if val.is_empty() {
            continue;
        }

        match norm_label.as_str() {
            "NOW" | "NÅ" => now = val,
            "STATUS" => status = val,
            "NEXT" | "NEXT STEP" | "NESTE" | "NESTE STEG" => next_step = val,
            _ => {}
        }
    }

    if now.is_empty() && status.is_empty() && next_step.is_empty() {
        return None;
    }

    Some(RamDumpSuggestion {
        now,
        status,
        next_step,
        source_label: model_label,
    })
}

fn build_ai_ramdump_prompt(snap: &ProjectSnapshot, lang: Language) -> String {
    let lang_instr = if lang == Language::Norwegian {
        "Reply in Norwegian (bokmål)."
    } else {
        "Reply in English."
    };
    format!(
        "Summarize this user's current work state concisely for an ADHD working-memory save-state (RAM-dump). The activity may be writing, studying, note-taking, image/video editing, design, or coding. {} Do not run tools or edit files. Reply with ONLY 3 single-line fields prefixed with NOW:, STATUS:, NEXT:\n\
         Workspace/App: {} (branch: {})\n\
         Active file/document: {}:{}\n\
         Current section/symbol: {}\n\
         Changed files ({}): {}\n\
         Recent commits: {}\n\
         Excerpt:\n{}",
        lang_instr,
        snap.project_name,
        snap.branch,
        snap.file_rel,
        snap.line,
        snap.enclosing_symbol.as_deref().unwrap_or("none"),
        snap.changed_count,
        snap.changed_files.join(", "),
        snap.recent_commits.join(" | "),
        snap.code_excerpt.chars().take(800).collect::<String>(),
    )
}

fn run_cli_prompt(bin: &Path, args: &[&str], project_path: &str) -> Option<String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_path = std::env::temp_dir().join(format!("tether_ai_{}_{}.txt", std::process::id(), nanos));
    let tmp_file = std::fs::File::create(&tmp_path).ok()?;

    let timeout_bin = Path::new("/usr/bin/timeout");
    let mut cmd = if timeout_bin.is_file() {
        let mut c = Command::new(timeout_bin);
        c.args(["--kill-after=1", "15"]).arg(bin);
        c
    } else {
        Command::new(bin)
    };

    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(tmp_file))
        .stderr(Stdio::null());

    if let Some(env_path) = build_cli_env_path() {
        cmd.env("PATH", env_path);
    }
    if !project_path.is_empty() && Path::new(project_path).is_dir() {
        cmd.current_dir(project_path);
    }

    let status = cmd.status().ok();
    let content = std::fs::read_to_string(&tmp_path).ok();
    let _ = std::fs::remove_file(&tmp_path);

    let status = status?;
    let text = content?;
    if !status.success() || text.trim().is_empty() {
        return None;
    }
    Some(text)
}

fn try_cli_provider(
    spec: &CliProviderSpec,
    snap: &ProjectSnapshot,
    lang: Language,
) -> Option<RamDumpSuggestion> {
    let bin = find_binary(spec.mise_rel_paths, spec.bin_names)?;
    let prompt = build_ai_ramdump_prompt(snap, lang);
    let args: Vec<&str> = match spec.mode {
        AiAssistMode::Antigravity => {
            vec!["--print-timeout", "15s", "--disable-slash-commands", "-p", &prompt]
        }
        AiAssistMode::Copilot => vec!["-s", "-p", &prompt],
        AiAssistMode::Gemini => vec!["-p", &prompt],
        AiAssistMode::Codex => vec!["exec", &prompt],
        AiAssistMode::OpenCode => vec!["--prompt", &prompt],
        AiAssistMode::Claude => vec!["-p", &prompt],
        _ => return None,
    };

    let stdout = run_cli_prompt(&bin, &args, &snap.project_path)?;
    parse_ai_ramdump_lines(&stdout, spec.source_label.to_string())
}

fn try_any_cli_ramdump(snap: &ProjectSnapshot, lang: Language) -> Option<RamDumpSuggestion> {
    let mut tried = 0usize;
    for spec in CLI_PROVIDERS {
        if spec.mode == AiAssistMode::OpenCode {
            continue;
        }
        if find_binary(spec.mise_rel_paths, spec.bin_names).is_none() {
            continue;
        }
        tried += 1;
        if let Some(res) = try_cli_provider(spec, snap, lang) {
            return Some(res);
        }
        if tried >= 2 {
            break;
        }
    }
    None
}

fn try_ollama_ramdump(snap: &ProjectSnapshot, model: &str, lang: Language) -> Option<RamDumpSuggestion> {
    let prompt = build_ai_ramdump_prompt(snap, lang);
    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false
    });

    let out = Command::new("curl")
        .args([
            "-s",
            "--max-time",
            "2",
            "-X",
            "POST",
            "http://127.0.0.1:11434/api/generate",
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload.to_string(),
        ])
        .output()
        .ok()?;

    if !out.status.success() || out.stdout.is_empty() {
        return None;
    }

    let val: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let response_text = val.get("response")?.as_str()?;
    parse_ai_ramdump_lines(response_text, format!("Ollama ({})", model))
}

/// Generates a smart RAM-dump suggestion using configured local/CLI AI backends or Git/code analysis.
pub fn suggest_ramdump_fields(anchor: Option<&ContextAnchor>, settings: &TetherSettings) -> RamDumpSuggestion {
    let lang = settings.language.resolve();
    let snap = gather_project_snapshot(anchor);
    let ollama_model = if settings.ollama_model.trim().is_empty() {
        "qwen2.5-coder:7b"
    } else {
        settings.ollama_model.trim()
    };

    match settings.ai_assist_mode {
        AiAssistMode::GitOnly => {}
        AiAssistMode::Ollama => {
            if let Some(res) = try_ollama_ramdump(&snap, ollama_model, lang) {
                return res;
            }
        }
        AiAssistMode::Cli => {
            if let Some(res) = try_any_cli_ramdump(&snap, lang) {
                return res;
            }
        }
        AiAssistMode::Auto => {
            if let Some(res) = try_ollama_ramdump(&snap, ollama_model, lang) {
                return res;
            }
            if let Some(res) = try_any_cli_ramdump(&snap, lang) {
                return res;
            }
        }
        specific_mode => {
            if let Some(spec) = CLI_PROVIDERS.iter().find(|s| s.mode == specific_mode) {
                if let Some(res) = try_cli_provider(spec, &snap, lang) {
                    return res;
                }
            }
        }
    }

    heuristic_ramdump_from_snapshot(&snap, lang)
}

pub fn format_absence_duration(elapsed_secs: u64, is_no: bool) -> String {
    let hours = elapsed_secs / 3600;
    let days = hours / 24;
    if days >= 2 {
        if is_no { format!("{} dager siden", days) } else { format!("{} days ago", days) }
    } else if days == 1 {
        if is_no { "Sist aktiv i går".to_string() } else { "Last active yesterday".to_string() }
    } else if hours >= 1 {
        if is_no { format!("{} timer siden", hours) } else { format!("{} hours ago", hours) }
    } else {
        let mins = (elapsed_secs / 60).max(1);
        if is_no { format!("{} min siden", mins) } else { format!("{} mins ago", mins) }
    }
}

pub fn build_where_was_i_from_parts(
    settings: &TetherSettings,
    anchor: Option<&ContextAnchor>,
    last_dump: Option<&(String, String, String, String)>,
    last_shutdown: Option<&ShutdownRecord>,
    impulses: &[storage::ImpulseItem],
    snap: &ProjectSnapshot,
    now_unix: u64,
) -> Option<WhereWasICard> {
    if anchor.is_none() && last_dump.is_none() && last_shutdown.is_none() && snap.changed_count == 0 {
        return None;
    }

    let is_no = settings.language.resolve() == Language::Norwegian;

    let anchor_ts = anchor.map(|a| a.timestamp).unwrap_or(0);
    let shutdown_ts = last_shutdown.map(|s| s.timestamp.max(0) as u64).unwrap_or(0);
    let latest_ts = anchor_ts.max(shutdown_ts);

    let elapsed_secs = if latest_ts > 0 && now_unix >= latest_ts {
        now_unix - latest_ts
    } else {
        0
    };

    let is_after_long_absence = elapsed_secs >= 4 * 3600
        || last_shutdown
            .map(|s| {
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                !s.date.is_empty() && s.date != today
            })
            .unwrap_or(false);

    let absence_text = if latest_ts > 0 {
        format_absence_duration(elapsed_secs, is_no)
    } else if let Some(sd) = last_shutdown {
        if is_no {
            format!("Parkert {}", sd.date)
        } else {
            format!("Parked {}", sd.date)
        }
    } else if is_no {
        "Nylig aktiv".to_string()
    } else {
        "Recently active".to_string()
    };

    let headline = if !snap.project_name.is_empty() && !snap.file_rel.is_empty() {
        let branch = if snap.branch.is_empty() {
            String::new()
        } else {
            format!(" [{}]", snap.branch)
        };
        format!("{}{} — {}:{}", snap.project_name, branch, snap.file_rel, snap.line.max(1))
    } else if !snap.project_name.is_empty() {
        snap.project_name.clone()
    } else if is_no {
        "Siste arbeidsøkt".to_string()
    } else {
        "Last work session".to_string()
    };

    let bullet_last_state = if let Some(sd) = last_shutdown {
        if !sd.summary_now.trim().is_empty() {
            if is_no {
                format!("Kvelds-notat ({}): {}", sd.date, sd.summary_now)
            } else {
                format!("Evening note ({}): {}", sd.date, sd.summary_now)
            }
        } else if let Some((time, now, _, _)) = last_dump {
            format!("RAM-dump ({}): {}", time, now)
        } else {
            anchor.map(|a| a.last_note.clone()).unwrap_or_default()
        }
    } else if let Some((time, now, status, _)) = last_dump {
        if status.is_empty() {
            format!("RAM-dump ({}): {}", time, now)
        } else {
            format!("RAM-dump ({}): {} ({})", time, now, status)
        }
    } else if let Some(a) = anchor {
        if !a.last_note.is_empty() {
            a.last_note.clone()
        } else if is_no {
            format!("Sist i {}:{}", a.relative_path, a.line)
        } else {
            format!("Last at {}:{}", a.relative_path, a.line)
        }
    } else {
        String::new()
    };

    let bullet_git = if snap.changed_count > 0 {
        if is_no {
            format!("Git: {} ucommitede filer ({})", snap.changed_count, snap.changed_files.join(", "))
        } else {
            format!("Git: {} uncommitted files ({})", snap.changed_count, snap.changed_files.join(", "))
        }
    } else if let Some(commit) = snap.recent_commits.first() {
        if is_no {
            format!("Git: ren arbeidskopi • {}", commit)
        } else {
            format!("Git: clean tree • {}", commit)
        }
    } else if is_no {
        "Git: ingen endringer registrert".to_string()
    } else {
        "Git: no changes detected".to_string()
    };

    let open_impulses: Vec<&str> = impulses
        .iter()
        .filter(|i| !i.done)
        .map(|i| i.text.as_str())
        .collect();
    let bullet_impulses = if open_impulses.is_empty() {
        if is_no {
            "Impulser: ingen åpne punkter".to_string()
        } else {
            "Impulses: no open items".to_string()
        }
    } else {
        let preview: Vec<&str> = open_impulses.iter().copied().take(2).collect();
        if is_no {
            format!("{} åpne impulser: {}", open_impulses.len(), preview.join(" • "))
        } else {
            format!("{} open impulses: {}", open_impulses.len(), preview.join(" • "))
        }
    };

    let suggested_first_step = if let Some(sd) = last_shutdown {
        if !sd.tomorrow_first_step.trim().is_empty() {
            sd.tomorrow_first_step.clone()
        } else if let Some((_, _, _, next)) = last_dump {
            if !next.trim().is_empty() {
                next.clone()
            } else {
                heuristic_ramdump_from_snapshot(snap, settings.language.resolve()).next_step
            }
        } else {
            heuristic_ramdump_from_snapshot(snap, settings.language.resolve()).next_step
        }
    } else if let Some((_, _, _, next)) = last_dump {
        if !next.trim().is_empty() {
            next.clone()
        } else {
            heuristic_ramdump_from_snapshot(snap, settings.language.resolve()).next_step
        }
    } else {
        heuristic_ramdump_from_snapshot(snap, settings.language.resolve()).next_step
    };

    Some(WhereWasICard {
        absence_text,
        is_after_long_absence,
        headline,
        bullet_last_state,
        bullet_git,
        bullet_impulses,
        suggested_first_step,
    })
}

pub fn build_where_was_i(settings: &TetherSettings) -> Option<WhereWasICard> {
    let anchor = storage::load_context_anchor();
    let last_dump = storage::load_last_ramdump();
    let last_shutdown = storage::load_shutdown_record();
    let impulses = storage::load_structured_impulses_with_cleanup(settings);
    let snap = gather_project_snapshot(anchor.as_ref());
    let now_unix = chrono::Utc::now().timestamp().max(0) as u64;

    build_where_was_i_from_parts(
        settings,
        anchor.as_ref(),
        last_dump.as_ref(),
        last_shutdown.as_ref(),
        &impulses,
        &snap,
        now_unix,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{ImpulseItem, LanguageSetting};

    #[test]
    fn test_heuristic_ramdump_from_snapshot() {
        let snap = ProjectSnapshot {
            project_name: "tether-desktop".to_string(),
            branch: "feat/phase4".to_string(),
            file_rel: "src/ui.rs".to_string(),
            line: 142,
            enclosing_symbol: Some("pub fn update_ui_tick".to_string()),
            changed_files: vec!["src/ui.rs".to_string(), "core/src/storage.rs".to_string()],
            changed_count: 2,
            recent_commits: vec!["8d1eea2 ci(windows)".to_string()],
            ..Default::default()
        };

        let sug = heuristic_ramdump_from_snapshot(&snap, Language::Norwegian);
        assert!(sug.now.contains("tether-desktop (feat/phase4)"));
        assert!(sug.now.contains("src/ui.rs:142"));
        assert!(sug.now.contains("pub fn update_ui_tick"));
        assert!(sug.status.contains("2 ucommitede fil(er)"));
        assert!(sug.next_step.contains("src/ui.rs:142"));
        assert!(sug.next_step.contains("core/src/storage.rs"));
    }

    #[test]
    fn test_build_where_was_i_after_long_absence() {
        let mut settings = TetherSettings::default();
        settings.language = LanguageSetting::Norwegian;

        let anchor = ContextAnchor {
            timestamp: 1000,
            project_name: "tether".to_string(),
            branch: "main".to_string(),
            file_path: "/tmp/main.rs".to_string(),
            relative_path: "src/main.rs".to_string(),
            line: 25,
            ..Default::default()
        };

        let shutdown = ShutdownRecord {
            timestamp: 1000,
            date: "2026-11-13".to_string(),
            tomorrow_first_step: "Skriv tester for TimeAnchor".to_string(),
            summary_now: "Ferdigstilte audio chime".to_string(),
            remaining_impulses: 1,
        };

        let impulses = vec![ImpulseItem {
            text: "Sjekk Hyprland window rules".to_string(),
            created_at: None,
            done: false,
            completed_at: None,
        }];

        let snap = ProjectSnapshot {
            project_name: "tether".to_string(),
            branch: "main".to_string(),
            file_rel: "src/main.rs".to_string(),
            line: 25,
            changed_count: 1,
            changed_files: vec!["src/main.rs".to_string()],
            ..Default::default()
        };

        let now_unix = 1000 + 6 * 3600; // 6 hours later
        let card = build_where_was_i_from_parts(
            &settings,
            Some(&anchor),
            None,
            Some(&shutdown),
            &impulses,
            &snap,
            now_unix,
        )
        .expect("Should build card");

        assert!(card.is_after_long_absence);
        assert_eq!(card.absence_text, "6 timer siden");
        assert!(card.bullet_last_state.contains("Ferdigstilte audio chime"));
        assert!(card.bullet_impulses.contains("1 åpne impulser"));
        assert_eq!(card.suggested_first_step, "Skriv tester for TimeAnchor");
    }

    #[test]
    fn test_parse_ai_ramdump_lines() {
        let raw = "NOW: Fixing layout\nSTATUS: 2 files changed\nNEXT: Run cargo test";
        let res = parse_ai_ramdump_lines(raw, "Ollama (qwen)".to_string()).unwrap();
        assert_eq!(res.now, "Fixing layout");
        assert_eq!(res.status, "2 files changed");
        assert_eq!(res.next_step, "Run cargo test");
    }

    #[test]
    fn test_parse_ai_ramdump_markdown_and_norwegian() {
        let raw = "- **NÅ:** Jobber med `context_engine.rs`\n### [STATUS]: 3 endrede filer\n* **NESTE STEG:** Kjør `cargo test`";
        let res = parse_ai_ramdump_lines(raw, "Antigravity (agy)".to_string()).unwrap();
        assert_eq!(res.now, "Jobber med `context_engine.rs`");
        assert_eq!(res.status, "3 endrede filer");
        assert_eq!(res.next_step, "Kjør `cargo test`");
    }

    #[test]
    fn test_detect_installed_ai_tools_runs() {
        let tools = detect_installed_ai_tools();
        for tool in tools {
            assert!(!tool.trim().is_empty());
        }
    }

    #[test]
    fn test_find_markdown_and_latex_headings() {
        let md = ["# Prosjektplan", "", "## Bakgrunn", "Tekst her"].join("\n");
        let lines: Vec<&str> = md.lines().collect();
        assert_eq!(find_enclosing_symbol(&lines, 4), Some("Bakgrunn".to_string()));

        let tex = ["\\section{Innledning}", "Første avsnitt", "\\subsection{Metode}"].join("\n");
        let tex_lines: Vec<&str> = tex.lines().collect();
        assert_eq!(find_enclosing_symbol(&tex_lines, 3), Some("Metode".to_string()));
    }

    #[test]
    fn test_heuristic_universal_notes_and_media() {
        let notes_snap = ProjectSnapshot {
            project_name: "Obsidian Vault".to_string(),
            file_rel: "Drøfting Labprosjekt.md".to_string(),
            line: 1,
            enclosing_symbol: Some("Konklusjon".to_string()),
            ..Default::default()
        };
        let sug_notes = heuristic_ramdump_from_snapshot(&notes_snap, Language::Norwegian);
        assert!(sug_notes.now.contains("Drøfting Labprosjekt.md"));
        assert!(sug_notes.status.contains("Konklusjon"));
        assert!(sug_notes.next_step.contains("avsnitt"));
        assert_eq!(sug_notes.source_label, "Kontekst- og dokumentanalyse");

        let video_snap = ProjectSnapshot {
            project_name: "Kdenlive".to_string(),
            file_rel: "intro_klipp.kdenlive".to_string(),
            line: 1,
            ..Default::default()
        };
        let sug_video = heuristic_ramdump_from_snapshot(&video_snap, Language::English);
        assert!(sug_video.next_step.contains("Continue editing intro_klipp.kdenlive"));
    }
}
