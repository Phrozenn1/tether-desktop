# Tether Desktop ⚓

### Cross-Platform Flowmodoro Timer, RAM-Dump & ADHD Workflow Engine

A standalone desktop application written in **Rust** (using [egui / eframe](https://github.com/emilk/egui)) designed to bring the **Tether** ADHD workflow strategies to your entire operating system—compatible with **Linux**, **macOS**, and **Windows**.

---

## 🌟 Core Strategies

1. **Flowmodoro Engine (Count Up):**
   - Counts **UP** during focus so you never get interrupted mid-thought.
   - Automatically calculates your earned break based on the **45:15 ratio** ($\frac{1}{3}$ of work time).
   - Dynamic visual indicators: 🟢 **Green** glowing frame in Focus, 🔴 **Red** in Break.
   - Desktop notifications via native OS notification centers (libnotify / D-Bus on Linux, Toast on Windows, NotificationCenter on macOS).
2. **60-Second RAM-Dump Save-State:**
   - 3 rapid fields: `[NÅ]`, `[STATUS]`, `[NESTE STEG]`.
   - Freezes working memory before leaving your desk to prevent resumption paralysis.
   - Displays your previous save-state card on return.
3. **Side-Quest / Impulse Buffer:**
   - Single-line capture for intrusive thoughts (*"Pay bill"*, *"Check library"*).
   - Offloads working memory to a local markdown file without context-switching.
4. **Always-On-Top Mode (📌):**
   - Keeps Tether floating as a heads-up widget over your browser, IDE, or PDF reader.
5. **Yak-Shaving Realism Checks 🪒:**
   - Sarcastic reminders when you catch yourself spending an hour tweaking minor details instead of doing the actual task.

---

## ⌨️ Shortcuts

- **`Space`**: Toggle Flowmodoro (Start Focus $\rightarrow$ Switch to Break $\rightarrow$ Reset).
- **`Enter`**: Submit Side-Quest impulse.

---

## 🛠️ Building & Cross-Platform Installation

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (`cargo` and `rustc`).

### Linux
```bash
cargo build --release
# Binary produced at target/release/tether-desktop
```
Installed automatically to `~/.local/bin/tether` and registered in your desktop launcher as `Tether`.

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
Tether stores markdown logs in your OS standard state/data directories:
- **Linux:** `~/.local/state/tether/` (shared with `tether.nvim`!)
- **macOS:** `~/Library/Application Support/tether/`
- **Windows:** `%APPDATA%\tether\`

---

## 📄 License
MIT © Kim Bech
