# Tether Desktop ⚓

[![Release](https://img.shields.io/github/v/release/Phrozenn1/tether-desktop?color=blue&label=version)](https://github.com/Phrozenn1/tether-desktop/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue)](https://github.com/Phrozenn1/tether-desktop)

### Universal Flowmodoro Timer, Focus Soundscapes & Cognitive Companion for Deep Work

**Tether** is an unbreakable desktop companion built in **Rust** with **GTK4** and **Libadwaita** following GNOME Human Interface Guidelines (HIG). Designed for **creators, knowledge workers, students, researchers, and professionals**—crafted with universal design principles that empower **neurodivergent minds** (**ADHD**, **Autism/ASD**, **Dyslexia**, **AuDHD**) as well as **neurotypical individuals** seeking calm, sustainable focus free from arbitrary countdown timers, acoustic distractions, and shame-based streak counters.

Tether operates completely standalone as a desktop application across **Linux** (GNOME, KDE Plasma, Hyprland, Sway, X11, Wayland), **macOS**, and **Windows**, with optional deep bi-directional save-state synchronization for developers and technical writers using **Neovim** ([`tether.nvim`](https://github.com/Phrozenn1/tether.nvim)) and **VS Code** ([`tether-vscode`](https://github.com/Phrozenn1/tether-vscode)).

---

## 🌟 Core Features & Strategies

### 1. ⏱️ Flowmodoro Engine (Count-Up Focus)
Traditional Pomodoro (25m work / 5m break) forcibly breaks deep hyperfocus mid-thought, destroying fragile context that took 40 minutes to assemble. Tether counts **UP**:
- Work uninterrupted as long as your natural flow state lasts.
- Automatically earns proportional breaks based on your chosen ratio (standard **45:15** $\rightarrow$ 3:1).
- **Banked Break Time:** Returning early from a break saves your unspent rest time to your bank and resumes your focus session seamlessly.

### 2. ⚡ "Just 5 Minutes" Micro-Start (Friction Melt)
Overcome *task-initiation paralysis* by lowering the psychological barrier to entry to just 5 minutes:
- Starts a warm amber countdown from 05:00.
- Gentle audio chime signals completion: *"5 minutes completed! 🎉 You are in motion."*
- **Seamless Auto-Transition:** If you keep working, Tether automatically glides into regular Flowmodoro counting up from 05:00.
- Stopping before 5 minutes gives a kind, shame-free reset: *"Good attempt! Totally fine to stop here."*

### 3. 🎧 Hybrid Focus Soundscapes & Real Nature Ambience
Scientifically engineered auditory masking and neural entrainment combining procedural synthesis with long, authentic field recordings. Runs **100% offline with zero network calls, zero external player dependencies, and <0.1% CPU**:
- **Procedural Masking & Neural Entrainment (Infinite, Mathematical DSP):**
  - **Brown Noise (Brun støy):** Deep, soothing low-frequency rumble ($1/f^2$) for heavy concentration and masking tinnitus/office chatter.
  - **Pink Noise (Rosa støy):** Balanced $1/f$ waterfall spectrum for steady, sustained cognitive throughput.
  - **Green Noise (Grønn støy):** Calming mid-frequency nature band centered around ~500 Hz (filtering harsh highs and muddy subs), soothing for misophonia and hyperacusis.
  - **40 Hz Gamma Pulse (40 Hz Gamma-puls):** Calming brown noise carrier amplitude-modulated at 40 Hz for neural gamma entrainment, anchoring executive attention in ADHD brains.
- **Authentic Field Recording Loops (75–90s Long Cycles to Eliminate Pattern Fatigue):**
  - **Gentle Rain (Mildt regn):** 75s seamless loop of soothing, steady rainfall (from 96 kHz studio master).
  - **Rolling Ocean Waves (Rullende havbølger):** 90s seamless loop (1.5 minutes!) of rhythmic ocean surf crashing and washing over shoreline stones.
  - **Mountain Brook (Sildrende fjellbekk):** 75s seamless loop of clear mountain water rushing over rounded stones in a national park.
  - **Forest Wind (Skogsvind):** 75s seamless loop of Roland R-07 tripod recording capturing woodland wind gusting through the treetop canopy.
- **In-Row Controls on Flow Tab:** Instant soundscape dropdown, alternating play/pause button (`▶` / `⏸`), and speaker button opening a flush-right volume popover.
- **Relative Gain / Volume:** Stream gain is balanced at -20 LUFS to play quietly alongside your background music or podcasts without taking over your system master volume.
- **Auto-Play:** Automatically starts playback when entering focus and smoothly fades down when taking a break.

### 4. 📥 Interactive Impulse Buffer & Retention Pruning
Protect your flow when intrusive side-thoughts strike (*"Pay electricity bill"*, *"Check Rust docs"*, *"Refactor database"*):
- Capture thoughts in seconds without opening a browser or switching windows.
- **Interactive Checkboxes:** Check off tasks directly in the desktop app; Tether syncs checkboxes live to your markdown file (`- [ ]` $\leftrightarrow$ `- [x]`).
- **Configurable Retention Cleanup:** Keep your impulse list clean without clutter. Choose when completed tasks are pruned:
  - *After 1 hour*, *After 24 hours*, *After 7 days*, or *Keep forever*.

### 5. ⚓ Context Anchor & 1-Click Code Resumption
Prevents "working memory amnesia" after taking breaks, eating lunch, or returning the next morning:
- Automatically snapshots your active Git repository, branch, file, line number, column, and last note.
- Displays a dedicated **Context Anchor Card** on the Flow tab.
- **🚀 [Resume in Neovim / VS Code]:** 1 click focuses or launches your editor right where you left off.

### 6. 🌐 Platform-Agnostic Editor Focus Tracking
Unlike window-manager-specific tools, Tether uses zero-privilege atomic state synchronization (`editor_focus.json`):
- Both `tether.nvim` and `tether-vscode` report focus changes (`FocusGained` / `FocusLost` / `onDidChangeWindowState`).
- Tether Desktop detects active editor focus on **any desktop environment** (GNOME, KDE Plasma, Sway, Hyprland, X11, macOS, Windows).
- **Auto-Minimize / Tray:** Configure Tether to automatically minimize to Tray or shrink to Mini-HUD whenever you are actively typing in your code editor.

### 7. 🔥 Momentum & Flyt-XP (Shame-Free Progression)
- Replaces punishing streak counters with **positive weekly momentum**:
  - *Level / Tier progression*, *hours logged*, *impulses parked*.
- No red crosses, no resets, and no Rejection Sensitive Dysphoria (RSD) if you take a weekend or sick day off.

### 8. 🗕 Mini-HUD & Content-Driven Window Fitting
- **Mini-HUD Mode:** Sleek, always-on-top pill widget with live status badge, timer, and quick controls.
- **Dynamic Content Sizing:** The main window dynamically sizes itself to fit the Flow tab content without requiring any vertical scrolling to see your momentum stats.

---

## ⌨️ Shortcuts

| Shortcut | Action |
| :--- | :--- |
| **`Space`** | Toggle Flowmodoro (Start Focus $\rightarrow$ Switch to Break $\rightarrow$ Reset) |
| **`Ctrl` + `1`–`4`** | Switch tabs (Flyt, RAM-dump, Impulser, Oppsett) |
| **`Escape`** | Shrink full dashboard back to Mini-HUD |
| **`Enter`** | Submit side-quest impulse or save RAM-dump |

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
# Build & install local Flatpak (GNOME 49 runtime):
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

The current full-featured Windows package is built with GTK through MSYS2. A separate WinUI 3 preview is being developed with native Windows controls and the shared Rust timer core. Build the preview on Windows with the .NET 8 SDK, Rust MSVC toolchain, and Windows 10 SDK:

```bash
dotnet publish windows/Tether.WinUI/Tether.WinUI.csproj -c Release -r win-x64 --self-contained true
```

The WinUI preview currently ports the focus timer only; use the GTK package for the remaining desktop features until the port reaches feature parity.

---

## 📁 Shared State Directories

Tether stores shared state and markdown logs in standard OS directories:
- **Linux:** `~/.local/state/tether/` (shared with `tether.nvim` and `tether-vscode`!)
- **macOS:** `~/Library/Application Support/tether/`
- **Windows:** `%LOCALAPPDATA%\tether\`

Key state files:
- `flow_state.json`: Real-time bi-directional timer synchronization.
- `editor_focus.json`: Zero-privilege platform-agnostic editor focus state.
- `context_anchor.json`: Code position, git branch, and save-state anchor.
- `impulses.md`: Parsed interactive markdown task checklist.

---

## 🧩 Companion Extensions

- **Neovim:** [`tether.nvim`](https://github.com/Phrozenn1/tether.nvim) — Lua floating HUD, micro-start, yak-alarm, and context anchor.
- **VS Code:** [`tether-vscode`](https://github.com/Phrozenn1/tether-vscode) — Status bar integration, flow sync, and rapid RAM-dump commands.

---

## 📄 License & Audio Credits

- **Code:** [MIT License](LICENSE) © Kim Bech <kim@phrozenn.no>
- **Focus Soundscape Audio Assets:**
  - *Gentle Rain:* Based on field recording by Zuvji ([CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/))
  - *Ocean Waves:* Based on ambient surf recording ([CC-BY 3.0](https://creativecommons.org/licenses/by/3.0/))
  - *Mountain Brook:* Based on Okanagan Mountain snowmelt stream recording ([CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/))
  - *Forest Wind:* Based on Roland R-07 tripod woodland recording by Robert EA Harvey ([CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/))

---

## 🔬 Scientific Foundation & Research Rationale

Every design decision and workflow mechanic in Tether is engineered directly from peer-reviewed literature in cognitive neuroscience, executive dysfunction, stochastic resonance, and attention restoration theory:

👉 **Read the full scientific foundation:** [**`RESEARCH.md` — Scientific Rationale & Neurodivergent Research Foundation**](RESEARCH.md) *(74 peer-reviewed citations, universal cognitive ergonomics, and empirical analyses covering ADHD, Autism, Dyslexia, and Universal Design)*.
