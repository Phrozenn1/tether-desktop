# Tether Desktop ⚓

### Cross-Platform Flowmodoro Timer, RAM-Dump & ADHD Workflow Engine

A standalone desktop application written in **Rust** (using [egui / eframe](https://github.com/emilk/egui)) designed to bring the **Tether** ADHD workflow strategies to your entire operating system—compatible with **Linux** (Wayland / X11), **macOS**, and **Windows**.

---

## 🌟 Core Features & Strategies

1. **Flowmodoro Engine (Count Up):**
   - Counts **UP** during focus so you never get interrupted mid-thought.
   - Automatically calculates your earned break based on customizable ratios (e.g. **45:15**).
   - Dynamic visual indicators: 🟢 **Green** glowing aura in Focus, 🔴 **Red** in Break, 🔵 **Blue** in Ready/Idle.
   - Desktop notifications via native OS notification centers.

2. **Bidirectional Neovim Sync ([`tether.nvim`](https://github.com/Phrozenn1/tether.nvim)):**
   - Shared atomic state (`~/.local/state/tether/flow_state.json`).
   - Starting, pausing, or breaking in Neovim updates Desktop in real time, and vice versa.
   - Synchronizes break ratios and session epochs dynamically across restarts.

3. **60-Second RAM-Dump Save-State:**
   - 3 rapid fields: `[NÅ]`, `[STATUS]`, `[NESTE STEG]`.
   - Freezes working memory before taking a break to prevent resumption paralysis.
   - Displays your previous save-state card on return.

4. **Side-Quest / Impulse Buffer:**
   - Single-line capture for intrusive thoughts (*"Pay bill"*, *"Check library"*).
   - Offloads working memory without context-switching.
   - Native system file dialog (`rfd` / XDG Portal) to pick storage destination.
   - Multi-format support: Markdown `- [ ]`, Markdown `•`, Todo.txt, and JSON.

5. **Mini-HUD with Persistent Coordinates:**
   - Sleek pill HUD mode with live pulsating status dot, drag anchor, and time display.
   - Automatically remembers dragged window positions across restarts and expand/shrink cycles on Hyprland/Wayland.
   - Monitor locking support (pin HUD to specific displays or auto-detect).

6. **Theming & Localization:**
   - Automatic OS / Omarchy live theme detection.
   - Curated presets: *Ethereal*, *Catppuccin Mocha*, *Tokyo Night*, *Nord*, *Gruvbox Dark*, *Cyberpunk Neon*, and *Rosé Pine*.
   - Multilingual UI: English and Norwegian (Norsk).

7. **Yak-Shaving Realism Checks 🪒:**
   - Sarcastic reality checks when you catch yourself tweaking trivial details instead of doing the actual work.

---

## ⌨️ Shortcuts

- **`Space`**: Toggle Flowmodoro (Start Focus $\rightarrow$ Switch to Break $\rightarrow$ Reset).
- **`Escape`**: Shrink full dashboard back to Mini-HUD.
- **`Enter`**: Submit Side-Quest impulse.

---

## 🛠️ Building & Cross-Platform Installation

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (`cargo` and `rustc`).

### Linux
```bash
cargo build --release
# Installs to ~/.local/bin/tether:
cp target/release/tether-desktop ~/.local/bin/tether
```

### macOS
```bash
cargo build --release
# Native Cocoa / winit backend — works on Intel & Apple Silicon
```

### Windows
```bash
cargo build --release
# Native Win32 / DirectX / OpenGL backend via eframe
```

---

## 📁 Shared State
Tether stores shared state and markdown logs in OS standard directories:
- **Linux:** `~/.local/state/tether/` (shared with `tether.nvim`!)
- **macOS:** `~/Library/Application Support/tether/`
- **Windows:** `%APPDATA%\tether\`

---

## 📄 License
MIT © Kim Bech
