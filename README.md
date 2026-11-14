# Tether Desktop ⚓

[![Release](https://img.shields.io/github/v/release/Phrozenn1/tether-desktop?style=flat-square&color=22d3ee&label=version)](https://github.com/Phrozenn1/tether-desktop/releases)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=flat-square)](https://github.com/Phrozenn1/tether-desktop)

### Universal Flowmodoro Timer, Focus Soundscapes & Cognitive Companion for Deep Work

**Tether** is a humane desktop companion built in **Rust** with **GTK4** and **Libadwaita** following GNOME Human Interface Guidelines (HIG). Designed for **writers, students, researchers, designers, video editors, and developers**—crafted with universal design principles that empower **neurodivergent minds** (**ADHD**, **Autism/ASD**, **Dyslexia**, **AuDHD**) as well as anyone seeking calm, sustainable focus free from rigid alarms, acoustic distractions, and shame-based streak counters.

Tether runs standalone across **Linux** (GNOME, KDE Plasma, Hyprland, Sway, X11, Wayland), **macOS**, and **Windows**, automatically tracking context across **Obsidian**, **LibreOffice**, **Typora**, **PDF readers**, **creative tools**, **Neovim**, and **VS Code**, with optional deep bi-directional plugins ([`tether-obsidian`](https://github.com/Phrozenn1/tether-obsidian), [`tether.nvim`](https://github.com/Phrozenn1/tether.nvim), and [`tether-vscode`](https://github.com/Phrozenn1/tether-vscode)).

---

## 🌟 Core Features & Strategies

### 1. ⏱️ Flowmodoro Engine & Banked Rest
Traditional Pomodoro (25m work / 5m break) shatters deep hyperfocus mid-thought, destroying fragile context that took 40 minutes to assemble. Tether counts **UP**:
- Ride your natural focus wave without arbitrary interruptions.
- Automatically earn proportional rest based on your chosen ratio (default **45:15** $\rightarrow$ 3:1).
- **Banked Break Time:** Returning early from a break saves unspent rest to your break bank and resumes focus smoothly.

### 2. ⚡ "Just 5 Minutes" Micro-Start (Friction Melt)
Overcome *task-initiation paralysis* by lowering the barrier to entry to just 5 minutes:
- Starts a calm 05:00 countdown followed by a gentle audio chime.
- **Seamless Auto-Transition:** Keep working and Tether glides straight into regular Flowmodoro counting up from 05:00.
- Stopping before 5 minutes gives a supportive, shame-free reset.

### 3. 🎧 Hybrid Focus Soundscapes & Real Nature Ambience
Auditory masking and neural entrainment combining procedural DSP with long, authentic field recordings. Runs **100% offline with zero network calls, zero external player dependencies, and <0.1% CPU**:
- **Procedural Masking & Neural Entrainment:**
  - **Brown Noise:** Deep low-frequency rumble ($1/f^2$) for concentration and masking distractions.
  - **Pink Noise:** Balanced $1/f$ waterfall spectrum for steady cognitive throughput.
  - **Green Noise:** Calming mid-frequency band (~500 Hz) tuned for misophonia and hyperacusis.
  - **40 Hz Gamma Pulse:** Calm brown noise carrier modulated at 40 Hz for neural gamma entrainment and attention anchoring.
- **Authentic Field Recording Loops (75–90s Cycles to Avoid Pattern Fatigue):**
  - **Gentle Rain** (75s loop from 96 kHz studio master), **Rolling Ocean Waves** (90s surf loop), **Mountain Brook** (75s loop), and **Forest Wind** (75s woodland canopy loop).
- **Auto-Play & Relative Volume:** Starts automatically on focus, fades smoothly on break, and sits unobtrusively at -20 LUFS alongside music or calls.

### 4. 🧠 AI & Context-Assisted RAM-Dumps (with Full Edit & Delete History)
Capture working memory before stepping away—or reconstruct it instantly when returning:
- **3-Line Cognitive Save-State:** Records **Now**, **Status**, and **Next Step** to `~/.local/state/tether/ramdumps.md`.
- **Non-Blocking Auto-Fill:** Uses active file/heading/document context (plus optional Git context) with automatic CLI AI assistance (**Antigravity**, **GitHub Copilot**, **Claude Code**, **Gemini CLI**, **Codex**, or **Ollama**) and a deterministic domain-aware heuristic fallback.
- **Full History CRUD:** Browse, edit, update, or delete any saved RAM-dump directly inside Tether Desktop or Obsidian.

### 5. 🌐 Universal App Integration & "Where Was I?" Recovery
Works seamlessly far beyond code editors—preventing "working memory amnesia" across any creative, study, or writing tool:
- **Supported Workflows & Automatic Tracking:**
  - **Writing & Notes:** **Obsidian** (via `tether-obsidian` + `obsidian://` deep-link resume), **Typora**, **LibreOffice Writer/Calc/Impress**.
  - **Study & PDFs:** **Sioyek**, **Evince**, **Zathura**, **Okular**, **Xournal++**.
  - **Image & Design:** **GIMP**, **Krita**, **Inkscape**, **Darktable**.
  - **Video, 3D & Audio:** **Kdenlive**, **Omacut**, **Blender**, **Audacity**, **OBS Studio**.
  - **Software Development:** **Neovim / Neovide**, **VS Code / VSCodium / Cursor**, **Helix**.
- **Context Anchor & 1-Click Resume:** Live OS window & open-file detection snapshots your active document, heading, or repository for 1-click resume.
- **"Where Was I?" Card:** Greets you after an absence with your last parked RAM-dump, evening shutdown summary, and active anchor.
- **Zero-Privilege Focus Sync:** Plugins report focus state via `editor_focus.json` across GNOME, KDE, Hyprland, Sway, X11, macOS, and Windows so Tether can auto-minimize to tray or Mini-HUD.

### 6. 🌙 Evening Shutdown Ritual
End your workday deliberately so your brain can actually unplug:
- Guided end-of-day wizard stops timers and soundscapes, writes today's closing summary and **tomorrow's first micro-step** (`last_shutdown.json`), and safeguards open impulses until morning.

### 7. ⏰ Time Anchor & Gentle Hard-Stop
Counteract *time blindness* during deep hyperfocus:
- Pin an end-of-day cutoff or meeting target (*Hard-Stop*) with gentle ambient cues before cutoff without harsh jarring alarms.

### 8. 📥 Interactive Impulse Buffer & Soft Distraction Speedbump
- **Impulse Parking:** Capture sudden stray thoughts without switching windows; sync interactive checkboxes (`- [ ]` $\leftrightarrow$ `- [x]`) directly to Markdown or JSON with configurable retention pruning (1h / 24h / 7d / Keep).
- **Distraction Speedbump (*Fartsdump*):** Calmly intercepts rabbit-hole apps or sites during focus so you can park the urge as an impulse or take a conscious break instead of losing an hour unintentionally.

### 9. 🎯 Dopamine Menu, Shame-Free Momentum & Mini-HUD
- **Dopamine Account & Menu:** Earn points for focus, Micro-Starts, and shielded impulses, redeemable guilt-free on custom rewards.
- **Shame-Free Momentum (Flow XP):** Tracks weekly progress, tiers, and logged hours with zero punitive streaks or red crosses.
- **Mini-HUD Mode:** Compact, always-on-top pill widget with live timer, soundscape toggle, and quick controls.

---

## ⌨️ Shortcuts

| Shortcut | Action |
| :--- | :--- |
| **`Space`** | Toggle Flowmodoro (Start Focus $\rightarrow$ Switch to Break $\rightarrow$ Reset) |
| **`Ctrl` + `1`–`4`** | Switch tabs (Flow, RAM-Dump, Impulses, Settings) |
| **`Escape`** | Shrink full dashboard back to Mini-HUD |
| **`Enter`** | Submit impulse or save active form |

---

## ⚡ Global Quick-Capture & Shared CLI

### Global Quick-Capture Modal (`--quick-capture`)
Bind a system-wide shortcut to capture impulses, save AI-assisted RAM-dumps, or trigger a 5-minute Micro-Start from any application without switching windows:
```bash
tether --quick-capture      # Unified impulse + RAM-dump + flow popup
tether --impulse            # Focused impulse-only prompt
tether --ramdump            # Focused RAM-dump prompt with auto-fill
```
**Hyprland example (`~/.config/hypr/bindings.conf`):**
```ini
bindd = SUPER, T, Tether Quick-Capture, exec, tether --quick-capture
```

### Shared Core CLI (`--cli`)
Scripts, window managers, and companion plugins can invoke `tether-core` operations directly via JSON:
```bash
tether --cli flow.status
tether --cli flow.micro_start
tether --cli impulses.add '{"text":"Check lecture notes"}'
tether --cli '{"operation":"ramdump.list"}'
```

### Waybar Status Module (`--waybar`)
Display live Flowmodoro status, active context anchor, and open impulse count directly in your Waybar (`~/.config/waybar/config.jsonc`):
```jsonc
"custom/tether": {
  "exec": "tether --waybar",
  "return-type": "json",
  "interval": 1,
  "on-click": "tether --cli flow.toggle",
  "on-click-middle": "tether --cli flow.micro_start",
  "on-click-right": "tether --quick-capture"
}
```

---

## 🛠️ Building & Cross-Platform Installation

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (edition 2021).
- **GTK4** (`libgtk-4-dev`) and **Libadwaita** (`libadwaita-1-dev` $\ge$ 1.5).

### Arch Linux (AUR / PKGBUILD)
```bash
# Quick binary installation:
cd packaging/arch
makepkg -p PKGBUILD-bin -si

# Or build from source:
makepkg -p PKGBUILD -si
```

### Flatpak
```bash
flatpak-builder --user --install --force-clean build-dir org.tether.Tether.yml
flatpak run org.tether.Tether
```

### Ubuntu / Debian (.deb)
```bash
cargo build --release
cargo deb
sudo dpkg -i target/debian/tether_*.deb
```

### Fedora / RHEL (.rpm)
```bash
cargo build --release
sudo dnf install -y target/rpm/tether-*.rpm
```

### macOS (.dmg)
```bash
brew install gtk4 libadwaita pkg-config
cargo build --release
```

### Windows
The full-featured Windows package is built with GTK via MSYS2. A separate WinUI 3 preview in [`windows/Tether.WinUI`](windows/Tether.WinUI) pairs native Windows controls with the shared Rust core (`.NET 8 SDK`, Rust MSVC toolchain, Windows 10 SDK):
```bash
dotnet publish windows/Tether.WinUI/Tether.WinUI.csproj -c Release -r win-x64 --self-contained true
```

---

## 📁 Shared State Directories

Tether stores shared state and markdown logs in standard OS directories:
- **Linux:** `~/.local/state/tether/` (shared across Desktop, Obsidian, Neovim, and VS Code)
- **macOS:** `~/Library/Application Support/tether/`
- **Windows:** `%LOCALAPPDATA%\tether\`

Key state files:
- `flow_state.json`: Real-time bi-directional Flowmodoro state synchronization.
- `editor_focus.json`: Platform-agnostic active application/editor focus state.
- `context_anchor.json`: Active document/file, project/vault, cursor position, and heading/branch anchor.
- `ramdumps.md`: Persistent working memory save-states (created, edited, and deleted cleanly).
- `impulses.md`: Interactive markdown impulse & side-quest checklist.
- `last_shutdown.json`: Evening shutdown summary and tomorrow's first micro-step for *"Where Was I?"*.
- `dopamine.json` & `stats.json`: Dopamine reward balance and weekly shame-free momentum metrics.

---

## 🧩 Companion Extensions

- **Obsidian:** [`tether-obsidian`](https://github.com/Phrozenn1/tether-obsidian) — Flowmodoro status bar, automatic vault/note anchoring, RAM-dump CRUD manager, impulse capture, and shutdown ritual.
- **Neovim:** [`tether.nvim`](https://github.com/Phrozenn1/tether.nvim) — Lua floating HUD, micro-start, yak-alarm, context anchor, and shutdown ritual.
- **VS Code:** [`tether-vscode`](https://github.com/Phrozenn1/tether-vscode) — Status bar integration, flow sync, and rapid impulse capture.

---

## 📄 License & Audio Credits

- **Code:** [GPL-2.0 License](LICENSE) © Kim Bech <kim@phrozenn.no>
- **Focus Soundscape Audio Assets:**
  - *Gentle Rain:* Based on field recording by Zuvji ([CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/))
  - *Ocean Waves:* Based on ambient surf recording ([CC-BY 3.0](https://creativecommons.org/licenses/by/3.0/))
  - *Mountain Brook:* Based on Okanagan Mountain snowmelt stream recording ([CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/))
  - *Forest Wind:* Based on Roland R-07 tripod woodland recording by Robert EA Harvey ([CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/))

---

## 🔬 Scientific Foundation & Research Rationale

Every design decision and workflow mechanic in Tether is engineered directly from peer-reviewed literature in cognitive neuroscience, executive dysfunction, stochastic resonance, and attention restoration theory:

👉 **Read the full scientific foundation:** [**`RESEARCH.md` — Scientific Rationale & Neurodivergent Research Foundation**](RESEARCH.md) *(74 peer-reviewed citations, universal cognitive ergonomics, and empirical analyses covering ADHD, Autism, Dyslexia, and Universal Design)*.
