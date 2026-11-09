use std::time::Instant;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Stroke, Vec2};

use crate::flowmodoro::{FlowState, Flowmodoro};
use crate::i18n::Strings;
use crate::impulse::ImpulseState;
use crate::ramdump::RamDumpState;
use crate::storage::{self, Language, SideQuestFormat, TetherSettings};
use crate::theme::{ThemeColors, ThemePreset};

#[derive(PartialEq, Eq, Clone, Copy)]
enum ActiveTab {
    Flowmodoro,
    RamDump,
    Impulses,
    YakShaving,
    Settings,
}

pub struct TetherApp {
    flow: Flowmodoro,
    audio: crate::audio::SoundscapePlayer,
    previous_flow_state: FlowState,
    ramdump: RamDumpState,
    impulse: ImpulseState,
    settings: TetherSettings,
    theme_preset: ThemePreset,
    theme: ThemeColors,
    active_tab: ActiveTab,
    always_on_top: bool,
    is_mini_hud: bool,
    last_update: Instant,
    last_theme_check: Instant,
    last_pos_check: Instant,
    current_yak_idx: usize,
    save_feedback: Option<String>,
}

impl Default for TetherApp {
    fn default() -> Self {
        let settings = storage::load_settings();
        let theme_preset = ThemePreset::Auto;
        let theme = ThemeColors::from_preset(theme_preset);
        let mut flow = Flowmodoro::new();
        flow.focus_ratio = settings.ratio_focus;
        flow.break_ratio = settings.ratio_break;

        let audio = crate::audio::SoundscapePlayer::new(
            settings.soundscape,
            settings.soundscape_volume,
            settings.soundscape_auto_play,
        );
        let previous_flow_state = flow.state;

        Self {
            flow,
            audio,
            previous_flow_state,
            ramdump: RamDumpState::new(),
            impulse: ImpulseState::new(),
            settings,
            theme_preset,
            theme,
            active_tab: ActiveTab::Flowmodoro,
            always_on_top: true,
            is_mini_hud: true,
            last_update: Instant::now(),
            last_theme_check: Instant::now(),
            last_pos_check: Instant::now(),
            current_yak_idx: 0,
            save_feedback: None,
        }
    }
}

impl TetherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Last JetBrains Mono hvis tilgjengelig
        let mut fonts = egui::FontDefinitions::default();
        let font_candidates = [
            "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf",
            "/usr/share/fonts/jetbrains-mono/JetBrainsMono-Regular.ttf",
            "/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf",
        ];
        for path in font_candidates {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "JetBrainsMono".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    family.insert(0, "JetBrainsMono".to_owned());
                }
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    family.insert(1, "JetBrainsMono".to_owned());
                }
                break;
            }
        }
        cc.egui_ctx.set_fonts(fonts);

        let app = Self::default();
        app.apply_visuals(&cc.egui_ctx);
        if let Some((x, y)) = app.settings.custom_pos_mini {
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(200));
                move_window_to_pos(x, y);
            });
        } else if app.settings.locked_monitor != "auto" && !app.settings.locked_monitor.is_empty() {
            let mon = app.settings.locked_monitor.clone();
            let is_mini = app.is_mini_hud;
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(200));
                move_window_to_monitor(&mon, is_mini);
            });
        }
        app
    }

    fn apply_visuals(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = self.theme.bg_base;
        visuals.panel_fill = self.theme.bg_panel;
        visuals.faint_bg_color = self.theme.bg_card;
        visuals.extreme_bg_color = Color32::from_rgb(
            self.theme.bg_base.r().saturating_sub(4),
            self.theme.bg_base.g().saturating_sub(4),
            self.theme.bg_base.b().saturating_sub(10),
        );
        visuals.window_stroke = Stroke::new(1.0_f32, self.theme.border_subtle);

        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);

        visuals.widgets.inactive.bg_fill = self.theme.bg_card;
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(
            self.theme.bg_card.r().saturating_add(15),
            self.theme.bg_card.g().saturating_add(15),
            self.theme.bg_card.b().saturating_add(25),
        );
        visuals.widgets.active.bg_fill = self.theme.accent;

        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.theme.text_muted);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, self.theme.text_primary);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        visuals.selection.bg_fill = self.theme.bg_card;
        visuals.selection.stroke = Stroke::new(1.0_f32, self.theme.accent);

        ctx.set_visuals(visuals);
    }

    fn toggle_hud_mode(&mut self, ctx: &egui::Context) {
        self.is_mini_hud = !self.is_mini_hud;
        if self.is_mini_hud {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(260.0, 46.0)));
            #[cfg(target_os = "linux")]
            hyprland_shrink(&self.settings);
            self.last_pos_check = Instant::now() + std::time::Duration::from_millis(500);
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(460.0, 580.0)));
            #[cfg(target_os = "linux")]
            hyprland_expand(&self.settings.locked_monitor);
        }
    }

    fn check_os_theme(&mut self, ctx: &egui::Context) {
        if self.theme_preset == ThemePreset::Auto {
            let now = Instant::now();
            if now.duration_since(self.last_theme_check) >= std::time::Duration::from_secs(3) {
                self.last_theme_check = now;
                let new_theme = ThemeColors::from_preset(ThemePreset::Auto);
                if new_theme.accent != self.theme.accent || new_theme.bg_base != self.theme.bg_base {
                    self.theme = new_theme;
                    self.apply_visuals(ctx);
                }
            }
        }
    }
}

pub fn get_tether_window_pos() -> Option<(i32, i32)> {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            if let Ok(out) = std::process::Command::new("hyprctl")
                .args(["clients", "-j"])
                .output()
            {
                if out.status.success() {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                        if let Some(arr) = val.as_array() {
                            for client in arr {
                                if client.get("class").and_then(|c| c.as_str()) == Some("tether") {
                                    if let Some(at) = client.get("at").and_then(|a| a.as_array()) {
                                        if at.len() == 2 {
                                            if let (Some(x), Some(y)) = (at[0].as_i64(), at[1].as_i64()) {
                                                return Some((x as i32, y as i32));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn move_window_to_pos(x: i32, y: i32) {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            std::thread::spawn(move || {
                let move_cmd = format!(
                    "hl.dsp.window.move({{ x = {}, y = {}, relative = false, window = \"class:tether\" }})",
                    x, y
                );
                let _ = std::process::Command::new("hyprctl")
                    .args(["dispatch", &move_cmd])
                    .output();
            });
        }
    }
}

fn get_target_monitor(locked_monitor: &str) -> Option<storage::MonitorInfo> {
    let monitors = storage::get_available_monitors();
    if locked_monitor.is_empty() || locked_monitor == "auto" {
        monitors.iter().find(|m| m.focused).cloned().or_else(|| monitors.first().cloned())
    } else {
        monitors.iter().find(|m| m.name == locked_monitor).cloned().or_else(|| {
            monitors.iter().find(|m| m.focused).cloned().or_else(|| monitors.first().cloned())
        })
    }
}

pub fn move_window_to_monitor(locked_monitor: &str, is_mini_hud: bool) {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            let target_mon = get_target_monitor(locked_monitor);
            if let Some(mon) = target_mon {
                std::thread::spawn(move || {
                    let (x, y) = if is_mini_hud {
                        mon.mini_hud_pos()
                    } else {
                        mon.expanded_center_pos()
                    };
                    let move_cmd = format!(
                        "hl.dsp.window.move({{ x = {}, y = {}, relative = false, window = \"class:tether\" }})",
                        x, y
                    );
                    let _ = std::process::Command::new("hyprctl")
                        .args(["dispatch", &move_cmd])
                        .output();
                });
            }
        }
    }
}

fn hyprland_expand(locked_monitor: &str) {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            let target_mon = get_target_monitor(locked_monitor);
            std::thread::spawn(move || {
                let _ = std::process::Command::new("hyprctl")
                    .args([
                        "dispatch",
                        "hl.dsp.window.resize({ x = 460, y = 580, relative = false, window = \"class:tether\" })",
                    ])
                    .output();
                if let Some(mon) = target_mon {
                    let (cx, cy) = mon.expanded_center_pos();
                    let move_cmd = format!(
                        "hl.dsp.window.move({{ x = {}, y = {}, relative = false, window = \"class:tether\" }})",
                        cx, cy
                    );
                    let _ = std::process::Command::new("hyprctl")
                        .args(["dispatch", &move_cmd])
                        .output();
                } else {
                    let _ = std::process::Command::new("hyprctl")
                        .args([
                            "dispatch",
                            "hl.dsp.window.center({ window = \"class:tether\" })",
                        ])
                        .output();
                }
            });
        }
    }
}

fn hyprland_shrink(settings: &TetherSettings) {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            let custom_pos = settings.custom_pos_mini;
            let locked_mon = settings.locked_monitor.clone();
            std::thread::spawn(move || {
                let _ = std::process::Command::new("hyprctl")
                    .args([
                        "dispatch",
                        "hl.dsp.window.resize({ x = 260, y = 46, relative = false, window = \"class:tether\" })",
                    ])
                    .output();
                if let Some((x, y)) = custom_pos {
                    let move_cmd = format!(
                        "hl.dsp.window.move({{ x = {}, y = {}, relative = false, window = \"class:tether\" }})",
                        x, y
                    );
                    let _ = std::process::Command::new("hyprctl")
                        .args(["dispatch", &move_cmd])
                        .output();
                } else {
                    let target_mon = get_target_monitor(&locked_mon);
                    if let Some(mon) = target_mon {
                        let (x, y) = mon.mini_hud_pos();
                        let move_cmd = format!(
                            "hl.dsp.window.move({{ x = {}, y = {}, relative = false, window = \"class:tether\" }})",
                            x, y
                        );
                        let _ = std::process::Command::new("hyprctl")
                            .args(["dispatch", &move_cmd])
                            .output();
                    }
                }
            });
        }
    }
}

impl eframe::App for TetherApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update);
        self.last_update = now;

        self.flow.tick(dt);
        self.check_os_theme(ctx);

        let old_state = self.previous_flow_state;
        let new_state = self.flow.state;
        if old_state != new_state {
            self.previous_flow_state = new_state;
            self.audio.handle_flow_state_transition(old_state, new_state);
        }

        // Synkronisering og repainting
        ctx.request_repaint_after(std::time::Duration::from_millis(300));

        // Tastatursnarvei (Mellomrom = Toggle Flowmodoro)
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) && !ctx.wants_keyboard_input() {
            self.flow.toggle();
        }

        // Escape krymper fullt dashboard tilbake til Mini-HUD
        if !self.is_mini_hud && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.toggle_hud_mode(ctx);
        }

        // Periodisk sjekk av vindusposisjon hvis brukeren har flyttet Mini-HUD med musen
        if self.is_mini_hud && now >= self.last_pos_check && now.duration_since(self.last_pos_check) >= std::time::Duration::from_millis(800) {
            self.last_pos_check = now;
            if let Some((cur_x, cur_y)) = get_tether_window_pos() {
                let default_pos = get_target_monitor(&self.settings.locked_monitor).map(|m| m.mini_hud_pos());
                let is_at_default = if let Some((dx, dy)) = default_pos {
                    (cur_x - dx).abs() <= 10 && (cur_y - dy).abs() <= 10
                } else {
                    false
                };

                let should_save = match self.settings.custom_pos_mini {
                    Some((old_x, old_y)) => {
                        if is_at_default {
                            self.settings.custom_pos_mini = None;
                            true
                        } else if (cur_x - old_x).abs() > 4 || (cur_y - old_y).abs() > 4 {
                            self.settings.custom_pos_mini = Some((cur_x, cur_y));
                            true
                        } else {
                            false
                        }
                    }
                    None => {
                        if !is_at_default {
                            self.settings.custom_pos_mini = Some((cur_x, cur_y));
                            true
                        } else {
                            false
                        }
                    }
                };

                if should_save {
                    let _ = storage::save_settings(&self.settings);
                }
            }
        }

        // Mini-HUD modus
        if self.is_mini_hud {
            self.render_mini_hud_window(ctx);
            return;
        }

        // Full Dashboard modus
        self.render_full_dashboard(ctx);
    }
}

impl TetherApp {
    fn render_mini_hud_window(&mut self, ctx: &egui::Context) {
        let lang = self.settings.language;
        let (border_color, dot_color, aura_color, text_color, badge_bg, tag) = match self.flow.state {
            FlowState::MicroStart => (
                Color32::from_rgb(250, 185, 45),
                Color32::from_rgb(250, 185, 45),
                Color32::from_rgba_premultiplied(250, 185, 45, 65),
                Color32::from_rgb(255, 245, 215),
                Color32::from_rgba_premultiplied(65, 45, 10, 200),
                Strings::tag_micro(lang),
            ),
            FlowState::Focus => (
                self.theme.focus_green,
                self.theme.focus_green,
                Color32::from_rgba_premultiplied(self.theme.focus_green.r(), self.theme.focus_green.g(), self.theme.focus_green.b(), 60),
                Color32::from_rgb(230, 255, 240),
                Color32::from_rgba_premultiplied(20, 60, 40, 200),
                Strings::tag_focus(lang),
            ),
            FlowState::Break => (
                self.theme.break_red,
                self.theme.break_red,
                Color32::from_rgba_premultiplied(self.theme.break_red.r(), self.theme.break_red.g(), self.theme.break_red.b(), 60),
                Color32::from_rgb(255, 230, 230),
                Color32::from_rgba_premultiplied(70, 25, 30, 200),
                Strings::tag_break(lang),
            ),
            FlowState::Idle => (
                self.theme.border_subtle,
                self.theme.idle_color,
                Color32::from_rgba_premultiplied(self.theme.idle_color.r(), self.theme.idle_color.g(), self.theme.idle_color.b(), 40),
                self.theme.text_muted,
                Color32::from_rgba_premultiplied(25, 32, 60, 200),
                Strings::tag_ready(lang),
            ),
        };

        let stroke_width = if self.flow.is_active() { 1.5_f32 } else { 1.0_f32 };

        let panel = egui::CentralPanel::default().frame(
            egui::Frame::none()
                .fill(Color32::from_rgba_premultiplied(self.theme.bg_base.r(), self.theme.bg_base.g(), self.theme.bg_base.b(), 235))
                .stroke(Stroke::new(stroke_width, border_color))
                .rounding(23.0)
                .inner_margin(egui::Margin {
                    left: 14.0,
                    right: 12.0,
                    top: 6.0,
                    bottom: 6.0,
                }),
        );

        let res = panel.show(ctx, |ui| {
            let progress_pct = self.flow.progress_pct();

            ui.horizontal_centered(|ui| {
                // Dra-ikon (Anker)
                let drag_resp = ui.label(RichText::new("⚓").size(16.0).color(border_color));
                if drag_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }
                if ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) && drag_resp.hovered() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.add_space(4.0);

                // Pulserende status-prikk
                let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                let center = dot_rect.center();
                ui.painter().circle_filled(center, 6.0, aura_color);
                ui.painter().circle_filled(center, 3.5, dot_color);

                ui.add_space(4.0);

                // Digital Klokkevisning (Klikk for å starte/pause)
                let time_btn = egui::Button::new(
                    RichText::new(self.flow.current_time_display())
                        .size(20.0)
                        .monospace()
                        .strong()
                        .color(text_color),
                )
                .frame(false);

                let time_resp = ui.add(time_btn).on_hover_text(Strings::time_toggle_tooltip(lang));
                if time_resp.clicked() {
                    self.flow.toggle();
                }

                ui.add_space(6.0);

                // Sleek status badge
                egui::Frame::none()
                    .fill(badge_bg)
                    .stroke(Stroke::new(1.0_f32, border_color))
                    .rounding(5.0)
                    .inner_margin(egui::Margin::symmetric(7.0, 1.5))
                    .show(ui, |ui| {
                        ui.label(RichText::new(tag).size(9.0).strong().color(border_color));
                    });

                // Høyre kontroller
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Ekspander til fullt dashboard
                    let expand_btn = egui::Button::new(
                        RichText::new("⛶").size(15.0).color(self.theme.text_primary),
                    )
                    .min_size(Vec2::new(26.0, 26.0))
                    .rounding(6.0)
                    .fill(self.theme.bg_card);

                    if ui.add(expand_btn).on_hover_text(Strings::expand_tooltip(lang)).clicked() {
                        self.toggle_hud_mode(ctx);
                    }

                    // Handlingsikon (Start/Pause/Avbryt hurtigknapp)
                    let action_icon = match self.flow.state {
                        FlowState::Idle => "▶",
                        FlowState::MicroStart => "✕",
                        FlowState::Focus => "☕",
                        FlowState::Break => "⏹",
                    };
                    let act_btn = egui::Button::new(
                        RichText::new(action_icon).size(13.0).color(border_color),
                    )
                    .min_size(Vec2::new(24.0, 24.0))
                    .frame(false);
                    if ui.add(act_btn).on_hover_text(Strings::time_toggle_tooltip(lang)).clicked() {
                        self.flow.toggle();
                    }

                    // Fokus-lyd hurtigknapp
                    let sound_color = if self.audio.is_playing {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    };
                    let sound_btn = egui::Button::new(
                        RichText::new("🎧").size(12.0).color(sound_color),
                    )
                    .min_size(Vec2::new(22.0, 22.0))
                    .frame(false);
                    if ui.add(sound_btn).on_hover_text(if self.audio.is_playing { "Stopp fokus-lyd" } else { "Start fokus-lyd" }).clicked() {
                        self.audio.toggle_play();
                    }

                    // ⚡ Mikro-start hurtigknapp når i Idle
                    if self.flow.state == FlowState::Idle {
                        let micro_btn = egui::Button::new(
                            RichText::new("⚡").size(12.0).color(Color32::from_rgb(250, 185, 45)),
                        )
                        .min_size(Vec2::new(22.0, 22.0))
                        .frame(false);
                        if ui.add(micro_btn).on_hover_text(Strings::btn_micro_tooltip(lang)).clicked() {
                            self.flow.start_micro_start();
                        }
                    }
                });
            });

            // Mikro-framdriftsindikator langs bunnen av pillen
            if self.flow.is_active() && progress_pct > 0.0 {
                let rect = ui.max_rect();
                let bar_width = (rect.width() - 40.0) * progress_pct;
                let bar_rect = Rect::from_min_max(
                    Pos2::new(rect.min.x + 20.0, rect.max.y - 1.5),
                    Pos2::new(rect.min.x + 20.0 + bar_width, rect.max.y),
                );
                ui.painter().rect_filled(bar_rect, 1.0_f32, border_color);
            }
        });

        if res.response.double_clicked() {
            self.toggle_hud_mode(ctx);
        }

        res.response.context_menu(|ui| {
            if ui.button(Strings::context_menu_open(lang)).clicked() {
                self.toggle_hud_mode(ctx);
                ui.close_menu();
            }
            if ui.button(Strings::context_menu_reset(lang)).clicked() {
                self.flow.reset();
                ui.close_menu();
            }
            ui.separator();
            if ui.button(Strings::context_menu_quit(lang)).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                ui.close_menu();
            }
        });
    }

    fn render_full_dashboard(&mut self, ctx: &egui::Context) {
        let lang = self.settings.language;
        let (border_color, status_text) = match self.flow.state {
            FlowState::MicroStart => (Color32::from_rgb(250, 185, 45), Strings::tag_micro(lang)),
            FlowState::Focus => (self.theme.focus_green, Strings::tag_focus(lang)),
            FlowState::Break => (self.theme.break_red, Strings::tag_break(lang)),
            FlowState::Idle => (self.theme.idle_color, Strings::tag_ready(lang)),
        };

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_base)
                    .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                    .rounding(16.0),
            )
            .show(ctx, |ui| {
                // Header
                egui::Frame::none()
                    .fill(self.theme.bg_panel)
                    .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                    .rounding(egui::Rounding { nw: 16.0, ne: 16.0, sw: 0.0, se: 0.0 })
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let header_title = ui.heading(
                                RichText::new("⚓ TETHER")
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                            if header_title.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                            }
                            if ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) && header_title.hovered() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                            }

                            // Status chip
                            egui::Frame::none()
                                .fill(self.theme.bg_card)
                                .stroke(Stroke::new(1.0_f32, border_color))
                                .rounding(4.0)
                                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(status_text).size(10.0).strong().color(border_color));
                                });

                            // Neovim quick status
                            if storage::is_neovim_running() {
                                ui.label(RichText::new("🟢 NVIM").size(10.0).strong().color(self.theme.focus_green));
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Lukk
                                if ui
                                    .button(RichText::new("✕").size(14.0).color(self.theme.text_muted))
                                    .on_hover_text(Strings::context_menu_quit(lang))
                                    .clicked()
                                {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }

                                // Krymp til Mini-HUD
                                let mini_btn = egui::Button::new(
                                    RichText::new(Strings::mini_hud_btn(lang))
                                        .size(11.5)
                                        .strong()
                                        .color(self.theme.accent),
                                );
                                if ui.add(mini_btn).on_hover_text("Minimer til flytende hjørne-HUD (Esc)").clicked() {
                                    self.toggle_hud_mode(ctx);
                                }

                                // Fest øverst
                                if ui.checkbox(&mut self.always_on_top, Strings::always_on_top(lang)).changed() {
                                    let level = if self.always_on_top {
                                        egui::WindowLevel::AlwaysOnTop
                                    } else {
                                        egui::WindowLevel::Normal
                                    };
                                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                                }
                            });
                        });

                        ui.add_space(8.0);

                        // 5 Segmenterte Fane-velgere
                        ui.horizontal(|ui| {
                            self.render_tab_pill(ui, ActiveTab::Flowmodoro, Strings::tab_flowmodoro(lang));
                            self.render_tab_pill(ui, ActiveTab::RamDump, Strings::tab_ramdump(lang));
                            self.render_tab_pill(ui, ActiveTab::Impulses, Strings::tab_sidequests(lang));
                            self.render_tab_pill(ui, ActiveTab::YakShaving, Strings::tab_yak(lang));
                            self.render_tab_pill(ui, ActiveTab::Settings, Strings::tab_settings(lang));
                        });
                    });

                // Hovedinnhold i fane
                ui.add_space(10.0);
                match self.active_tab {
                    ActiveTab::Flowmodoro => self.render_flowmodoro(ui),
                    ActiveTab::RamDump => self.render_ramdump(ui),
                    ActiveTab::Impulses => self.render_impulses(ui),
                    ActiveTab::YakShaving => self.render_yak_shaving(ui),
                    ActiveTab::Settings => self.render_settings(ui, ctx),
                }
            });
    }

    fn render_tab_pill(&mut self, ui: &mut egui::Ui, tab: ActiveTab, label: &str) {
        let is_selected = self.active_tab == tab;
        let (bg, text_color, stroke) = if is_selected {
            (
                self.theme.bg_card,
                self.theme.text_primary,
                Stroke::new(1.0_f32, self.theme.accent),
            )
        } else {
            (
                Color32::from_rgba_premultiplied(self.theme.bg_card.r(), self.theme.bg_card.g(), self.theme.bg_card.b(), 100),
                self.theme.text_muted,
                Stroke::NONE,
            )
        };

        let btn = egui::Button::new(RichText::new(label).size(11.5).strong().color(text_color))
            .fill(bg)
            .stroke(stroke)
            .rounding(6.0)
            .min_size(Vec2::new(76.0, 26.0));

        if ui.add(btn).clicked() {
            self.active_tab = tab;
        }
    }

    fn render_flowmodoro(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        let (status_text, border_color, text_color) = match self.flow.state {
            FlowState::MicroStart => ("⚡ MIKRO-START (5 MIN)", Color32::from_rgb(250, 185, 45), Color32::from_rgb(250, 185, 45)),
            FlowState::Focus => (Strings::focus_title(lang), self.theme.focus_green, self.theme.focus_green),
            FlowState::Break => (Strings::break_title(lang), self.theme.break_red, self.theme.break_red),
            FlowState::Idle => (Strings::ready_title(lang), self.theme.idle_color, self.theme.text_muted),
        };

        ui.vertical_centered(|ui| {
            ui.add_space(6.0);

            // Hovedkort med klokke og status
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.5_f32, border_color))
                .rounding(16.0)
                .inner_margin(egui::Margin::symmetric(24.0, 18.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(status_text).size(12.0).strong().color(border_color));
                    ui.add_space(4.0);

                    ui.label(
                        RichText::new(self.flow.current_time_display())
                            .size(54.0)
                            .monospace()
                            .strong()
                            .color(text_color),
                    );

                    let sub_info = match self.flow.state {
                        FlowState::MicroStart => {
                            let rem = crate::flowmodoro::format_time(self.flow.micro_remaining_secs);
                            match lang {
                                Language::Norwegian => format!("⚡ Bare 5 min for å bryte dørstokkmila ({} gjenstår). Null skyldfølelse!", rem),
                                Language::English => format!("⚡ Just 5 mins to melt the friction ({} left). Zero guilt!", rem),
                            }
                        }
                        FlowState::Focus => {
                            let earned = self.flow.calculate_break_time();
                            let earned_str = crate::flowmodoro::format_time(earned);
                            Strings::sub_focus(lang, &earned_str, self.flow.focus_ratio, self.flow.break_ratio)
                        }
                        FlowState::Break => Strings::sub_break(lang).to_string(),
                        FlowState::Idle => Strings::sub_idle(lang).to_string(),
                    };
                    ui.label(RichText::new(sub_info).size(11.5).color(self.theme.text_muted));
                });

            ui.add_space(18.0);

            // Handlingsknapper
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 6.0);

                let (btn_text, btn_fill, btn_fg) = match self.flow.state {
                    FlowState::Idle => (Strings::btn_start_focus(lang), Color32::from_rgb(35, 90, 55), Color32::from_rgb(230, 255, 240)),
                    FlowState::MicroStart => ("✕ Avbryt (Null skyldfølelse)", Color32::from_rgb(90, 60, 20), Color32::from_rgb(255, 235, 180)),
                    FlowState::Focus => (Strings::btn_take_break(lang), Color32::from_rgb(110, 35, 40), Color32::from_rgb(255, 230, 230)),
                    FlowState::Break => (Strings::btn_end_break(lang), self.theme.bg_card, self.theme.text_primary),
                };

                let main_btn = egui::Button::new(RichText::new(btn_text).size(15.0).strong().color(btn_fg))
                    .fill(btn_fill)
                    .rounding(10.0)
                    .min_size(Vec2::new(150.0, 42.0));

                if ui.add(main_btn).clicked() {
                    self.flow.toggle();
                }

                if self.flow.state == FlowState::Idle {
                    let micro_btn = egui::Button::new(RichText::new(Strings::btn_micro_start(lang)).size(14.0).strong().color(Color32::from_rgb(255, 235, 180)))
                        .fill(Color32::from_rgb(90, 65, 15))
                        .rounding(10.0)
                        .min_size(Vec2::new(140.0, 42.0));
                    if ui.add(micro_btn).on_hover_text(Strings::btn_micro_tooltip(lang)).clicked() {
                        self.flow.start_micro_start();
                    }
                }

                if self.flow.is_active() {
                    let reset_btn = egui::Button::new(RichText::new(Strings::btn_reset(lang)).size(13.0).color(self.theme.text_muted))
                        .fill(self.theme.bg_panel)
                        .rounding(10.0)
                        .min_size(Vec2::new(90.0, 42.0));
                    if ui.add(reset_btn).clicked() {
                        self.flow.reset();
                    }
                }
            });

            ui.add_space(14.0);
            ui.label(
                RichText::new(Strings::shortcut_hint(lang))
                    .size(11.0)
                    .color(self.theme.text_muted),
            );

            ui.add_space(18.0);

            // Forholdstall og dagens statistikk
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                .rounding(12.0)
                .inner_margin(14.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Strings::ratio_title(lang)).size(12.0).strong());
                        if ui.add(egui::DragValue::new(&mut self.flow.focus_ratio).range(1..=120).suffix("m")).changed() {
                            self.settings.ratio_focus = self.flow.focus_ratio;
                            let _ = storage::save_settings(&self.settings);
                        }
                        ui.label(":");
                        if ui.add(egui::DragValue::new(&mut self.flow.break_ratio).range(1..=60).suffix("m")).changed() {
                            self.settings.ratio_break = self.flow.break_ratio;
                            let _ = storage::save_settings(&self.settings);
                        }
                    });

                    ui.add_space(6.0);
                    let total_mins = self.flow.total_focus_secs / 60;
                    ui.label(
                        RichText::new(Strings::stats_focus_today(lang, total_mins))
                            .size(11.5)
                            .color(self.theme.focus_green),
                    );
                });
        });
    }

    fn render_ramdump(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        ui.heading(RichText::new(Strings::ramdump_heading(lang)).color(self.theme.text_primary));
        ui.label(
            RichText::new(Strings::ramdump_subtitle(lang))
                .size(11.5)
                .color(self.theme.text_muted),
        );
        ui.add_space(12.0);

        egui::Frame::none()
            .fill(self.theme.bg_panel)
            .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
            .rounding(12.0)
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.label(RichText::new(Strings::field_now(lang)).size(12.0).strong().color(self.theme.accent));
                ui.add_sized([ui.available_width(), 26.0], egui::TextEdit::singleline(&mut self.ramdump.now).hint_text(Strings::field_now_hint(lang)));
                ui.add_space(8.0);

                ui.label(RichText::new(Strings::field_status(lang)).size(12.0).strong().color(self.theme.accent));
                ui.add_sized([ui.available_width(), 26.0], egui::TextEdit::singleline(&mut self.ramdump.status).hint_text(Strings::field_status_hint(lang)));
                ui.add_space(8.0);

                ui.label(RichText::new(Strings::field_next(lang)).size(12.0).strong().color(self.theme.focus_green));
                ui.add_sized([ui.available_width(), 26.0], egui::TextEdit::singleline(&mut self.ramdump.next_step).hint_text(Strings::field_next_hint(lang)));
                ui.add_space(12.0);

                let save_btn = egui::Button::new(
                    RichText::new(Strings::btn_save_ramdump(lang)).size(14.0).strong().color(self.theme.text_primary),
                )
                .fill(self.theme.bg_card)
                .rounding(8.0)
                .min_size(Vec2::new(ui.available_width(), 36.0));

                if ui.add(save_btn).clicked() {
                    let _ = self.ramdump.save();
                }

                if let Some(msg) = &self.ramdump.feedback_message {
                    ui.add_space(6.0);
                    ui.label(RichText::new(msg).color(self.theme.focus_green).size(12.0));
                }
            });

        if let Some((time, now, status, next)) = &self.ramdump.last_dump {
            ui.add_space(14.0);
            ui.label(RichText::new(Strings::last_dump_heading(lang, time)).strong().size(12.0));

            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(self.theme.focus_green.r(), self.theme.focus_green.g(), self.theme.focus_green.b(), 60)))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(Strings::bullet_now(lang, now)).color(self.theme.text_primary));
                    ui.label(RichText::new(Strings::bullet_status(lang, status)).color(self.theme.text_muted));
                    ui.label(RichText::new(Strings::bullet_next(lang, next)).strong().color(self.theme.focus_green));
                });
        }
    }

    fn render_impulses(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        ui.heading(RichText::new(Strings::sidequests_heading(lang)).color(self.theme.text_primary));
        ui.label(
            RichText::new(Strings::sidequests_subtitle(lang))
                .size(11.5)
                .color(self.theme.text_muted),
        );
        ui.add_space(12.0);

        // Input card
        ui.horizontal(|ui| {
            let input = ui.add_sized(
                [ui.available_width() - 85.0, 34.0],
                egui::TextEdit::singleline(&mut self.impulse.input_text)
                    .hint_text(Strings::sidequests_input_hint(lang)),
            );

            let submitted = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let add_btn = egui::Button::new(RichText::new(Strings::btn_capture(lang)).strong().color(self.theme.text_primary))
                .fill(self.theme.bg_card)
                .rounding(8.0)
                .min_size(Vec2::new(80.0, 34.0));
            let clicked = ui.add(add_btn).clicked();

            if submitted || clicked {
                let _ = self.impulse.capture_with_settings(&self.settings);
            }
        });

        if let Some(feedback) = &self.impulse.feedback {
            ui.add_space(6.0);
            ui.label(RichText::new(feedback).color(self.theme.focus_green).size(12.0));
        }

        ui.add_space(14.0);
        ui.label(RichText::new(Strings::saved_impulses_heading(lang)).strong().size(12.0));

        let mut to_remove = None;
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            if self.impulse.items.is_empty() {
                ui.add_space(10.0);
                ui.label(RichText::new(Strings::no_impulses(lang)).italics().color(self.theme.text_muted));
            } else {
                for (idx, item) in self.impulse.items.iter().enumerate().rev() {
                    egui::Frame::none()
                        .fill(self.theme.bg_panel)
                        .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(item).size(12.5).color(self.theme.text_primary));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(RichText::new("✓").size(12.0).color(self.theme.focus_green)).on_hover_text("Fjern / Fullfør").clicked() {
                                        to_remove = Some(idx);
                                    }
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
            }
        });

        if let Some(idx) = to_remove {
            let _ = self.impulse.remove_with_settings(&self.settings, idx);
        }
    }

    fn render_yak_shaving(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        let jabs = Strings::yak_jabs(lang);
        ui.heading(RichText::new(Strings::yak_heading(lang)).color(self.theme.text_primary));
        ui.label(
            RichText::new(Strings::yak_subtitle(lang))
                .size(11.5)
                .color(self.theme.text_muted),
        );
        ui.add_space(14.0);

        let current_jab = jabs[self.current_yak_idx % jabs.len()];
        egui::Frame::none()
            .fill(self.theme.bg_panel)
            .stroke(Stroke::new(1.5_f32, self.theme.break_red))
            .rounding(14.0)
            .inner_margin(18.0)
            .show(ui, |ui| {
                ui.label(RichText::new("“").size(32.0).color(self.theme.break_red));
                ui.label(RichText::new(current_jab).size(15.0).italics().strong().color(self.theme.text_primary));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let next_btn = egui::Button::new(
                        RichText::new(Strings::btn_next_jab(lang)).size(12.0).strong().color(self.theme.text_primary),
                    )
                    .fill(self.theme.bg_card)
                    .rounding(6.0);

                    if ui.add(next_btn).clicked() {
                        self.current_yak_idx = (self.current_yak_idx + 1) % jabs.len();
                    }

                    if ui.button(RichText::new(Strings::btn_back_to_flow(lang)).size(12.0).strong().color(self.theme.focus_green)).clicked() {
                        self.active_tab = ActiveTab::Flowmodoro;
                    }
                });
            });

        ui.add_space(16.0);
        ui.label(RichText::new(Strings::more_reminders(lang)).strong().size(12.0));

        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            for (i, jab) in jabs.iter().enumerate() {
                egui::Frame::none()
                    .fill(self.theme.bg_panel)
                    .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(10.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{}.", i + 1)).color(self.theme.break_red).strong().size(11.5));
                            ui.label(RichText::new(*jab).size(11.5).color(self.theme.text_muted));
                        });
                    });
                ui.add_space(3.0);
            }
        });
    }

    fn render_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let lang = self.settings.language;
        ui.heading(RichText::new(Strings::settings_heading(lang)).color(self.theme.text_primary));
        ui.add_space(8.0);

        egui::ScrollArea::vertical().max_height(460.0).show(ui, |ui| {
            // Seksjon: Språk & Tema
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(Strings::sec_general(lang)).strong().size(13.0).color(self.theme.accent));
                    ui.add_space(6.0);

                    // Språk
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Strings::lbl_language(lang)).size(12.0));
                        ui.radio_value(&mut self.settings.language, Language::English, "English");
                        ui.radio_value(&mut self.settings.language, Language::Norwegian, "Norsk");
                    });
                    ui.add_space(8.0);

                    // Tema-velger
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Strings::lbl_theme(lang)).size(12.0));
                        let current_preset = self.theme_preset;
                        let mut selected = current_preset;

                        egui::ComboBox::from_id_salt("theme_selector")
                            .selected_text(match selected {
                                ThemePreset::Auto => "Auto (Follow OS / Omarchy)",
                                ThemePreset::Ethereal => "Ethereal",
                                ThemePreset::CatppuccinMocha => "Catppuccin Mocha",
                                ThemePreset::TokyoNight => "Tokyo Night",
                                ThemePreset::Nord => "Nord",
                                ThemePreset::GruvboxDark => "Gruvbox Dark",
                                ThemePreset::Cyberpunk => "Cyberpunk Neon",
                                ThemePreset::RosePine => "Rosé Pine",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected, ThemePreset::Auto, "Auto (Follow OS / Omarchy)");
                                ui.selectable_value(&mut selected, ThemePreset::Ethereal, "Ethereal");
                                ui.selectable_value(&mut selected, ThemePreset::CatppuccinMocha, "Catppuccin Mocha");
                                ui.selectable_value(&mut selected, ThemePreset::TokyoNight, "Tokyo Night");
                                ui.selectable_value(&mut selected, ThemePreset::Nord, "Nord");
                                ui.selectable_value(&mut selected, ThemePreset::GruvboxDark, "Gruvbox Dark");
                                ui.selectable_value(&mut selected, ThemePreset::Cyberpunk, "Cyberpunk Neon");
                                ui.selectable_value(&mut selected, ThemePreset::RosePine, "Rosé Pine");
                            });

                        if selected != current_preset {
                            self.theme_preset = selected;
                            self.theme = ThemeColors::from_preset(selected);
                            self.apply_visuals(ctx);
                        }
                    });
                });

            ui.add_space(10.0);

            // Seksjon: Flowmodoro Forholdstall
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(Strings::sec_flowmodoro(lang)).strong().size(13.0).color(self.theme.accent));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Strings::ratio_title(lang)).size(12.0));
                        if ui.add(egui::DragValue::new(&mut self.settings.ratio_focus).range(1..=120).suffix("m")).changed() {
                            self.flow.focus_ratio = self.settings.ratio_focus;
                            self.flow.seq += 1;
                            self.flow.broadcast_state();
                        }
                        ui.label(":");
                        if ui.add(egui::DragValue::new(&mut self.settings.ratio_break).range(1..=60).suffix("m")).changed() {
                            self.flow.break_ratio = self.settings.ratio_break;
                            self.flow.seq += 1;
                            self.flow.broadcast_state();
                        }
                    });
                });

            ui.add_space(10.0);

            // Seksjon: Fokus-lyd & Støygenerator
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(Strings::sec_soundscape(lang)).strong().size(13.0).color(self.theme.accent));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Strings::lbl_soundscape_type(lang)).size(12.0));
                        let mut current_sound = self.audio.current_type;
                        let sound_label = match lang {
                            Language::Norwegian => current_sound.label_no(),
                            Language::English => current_sound.label_en(),
                        };

                        egui::ComboBox::from_id_salt("soundscape_selector")
                            .selected_text(sound_label)
                            .show_ui(ui, |ui| {
                                for st in [
                                    crate::audio::SoundscapeType::Off,
                                    crate::audio::SoundscapeType::BrownNoise,
                                    crate::audio::SoundscapeType::PinkNoise,
                                    crate::audio::SoundscapeType::GentleRain,
                                ] {
                                    let l = match lang {
                                        Language::Norwegian => st.label_no(),
                                        Language::English => st.label_en(),
                                    };
                                    ui.selectable_value(&mut current_sound, st, l);
                                }
                            });

                        if current_sound != self.audio.current_type {
                            self.audio.set_sound(current_sound);
                            self.settings.soundscape = current_sound;
                            let _ = storage::save_settings(&self.settings);
                        }

                        // Test avspilling knapp
                        let play_icon = if self.audio.is_playing { "⏸ Pause" } else { "▶ Spill" };
                        if ui.button(play_icon).clicked() {
                            self.audio.toggle_play();
                        }
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Strings::lbl_soundscape_vol(lang)).size(12.0));
                        let mut vol = self.audio.volume;
                        if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false)).changed() {
                            self.audio.set_volume(vol);
                            self.settings.soundscape_volume = vol;
                            let _ = storage::save_settings(&self.settings);
                        }
                        ui.label(format!("{:.0}%", vol * 100.0));
                    });

                    ui.add_space(6.0);
                    if ui.checkbox(&mut self.settings.soundscape_auto_play, Strings::chk_soundscape_auto(lang)).changed() {
                        self.audio.auto_play_on_focus = self.settings.soundscape_auto_play;
                        let _ = storage::save_settings(&self.settings);
                    }
                });

            ui.add_space(10.0);

            // Seksjon: Side-Quests lagring og format
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(Strings::sec_sidequests(lang)).strong().size(13.0).color(self.theme.accent));
                    ui.add_space(6.0);

                    ui.label(RichText::new(Strings::lbl_filepath(lang)).size(12.0));
                    ui.horizontal(|ui| {
                        ui.add_sized([ui.available_width() - 110.0, 26.0], egui::TextEdit::singleline(&mut self.settings.side_quests_file));
                        if ui.button(Strings::btn_browse(lang)).clicked() {
                            if let Some(path) = storage::pick_impulse_file() {
                                self.settings.side_quests_file = path.to_string_lossy().to_string();
                            }
                        }
                    });
                    ui.add_space(6.0);

                    // Hurtigknapper for sti
                    ui.horizontal(|ui| {
                        if ui.button("Standard").clicked() {
                            self.settings.side_quests_file = storage::get_data_dir().join("impulses.md").to_string_lossy().to_string();
                        }
                        if ui.button("~/Notes/side-quests.md").clicked() {
                            self.settings.side_quests_file = "~/Notes/side-quests.md".to_string();
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new(Strings::lbl_format(lang)).size(12.0));
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.settings.side_quests_format, SideQuestFormat::MarkdownCheckbox, "Markdown - [ ]");
                        ui.radio_value(&mut self.settings.side_quests_format, SideQuestFormat::MarkdownBullet, "Markdown •");
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.settings.side_quests_format, SideQuestFormat::TodoTxt, "Todo.txt");
                        ui.radio_value(&mut self.settings.side_quests_format, SideQuestFormat::Json, "JSON");
                    });
                });

            ui.add_space(10.0);

            // Seksjon: Neovim-integrasjon
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(Strings::sec_integration(lang)).strong().size(13.0).color(self.theme.accent));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        if storage::is_neovim_running() {
                            ui.label(RichText::new(Strings::nvim_status_online(lang)).color(self.theme.focus_green).strong());
                        } else {
                            ui.label(RichText::new(Strings::nvim_status_offline(lang)).color(self.theme.text_muted));
                        }

                        if ui.button(RichText::new(Strings::btn_open_nvim(lang)).strong().color(self.theme.accent)).clicked() {
                            storage::open_or_focus_neovim(&self.settings.terminal_command);
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new(Strings::lbl_terminal_cmd(lang)).size(12.0));
                    ui.add_sized([ui.available_width(), 26.0], egui::TextEdit::singleline(&mut self.settings.terminal_command));

                    ui.add_space(8.0);
                    ui.checkbox(&mut self.settings.auto_launch_desktop, Strings::chk_auto_desktop(lang));
                });

            ui.add_space(10.0);

            // Seksjon: Skjerm & Plassering (Monitor Lock)
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(Strings::sec_monitor(lang)).strong().size(13.0).color(self.theme.accent));
                    ui.add_space(4.0);
                    ui.label(RichText::new(Strings::hint_monitor(lang)).size(11.0).color(self.theme.text_muted));
                    ui.add_space(8.0);

                    let monitors = storage::get_available_monitors();
                    let current_lock = self.settings.locked_monitor.clone();
                    let current_label = if current_lock == "auto" || current_lock.is_empty() {
                        Strings::opt_auto_monitor(lang).to_string()
                    } else if let Some(m) = monitors.iter().find(|m| m.name == current_lock) {
                        m.display_label()
                    } else {
                        current_lock.clone()
                    };

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Strings::lbl_locked_monitor(lang)).size(12.0));
                        let mut selected_mon = self.settings.locked_monitor.clone();
                        egui::ComboBox::from_id_salt("locked_monitor_combo")
                            .selected_text(&current_label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected_mon, "auto".to_string(), Strings::opt_auto_monitor(lang));
                                for m in &monitors {
                                    ui.selectable_value(&mut selected_mon, m.name.clone(), m.display_label());
                                }
                            });

                        if selected_mon != self.settings.locked_monitor {
                            self.settings.locked_monitor = selected_mon.clone();
                            move_window_to_monitor(&selected_mon, self.is_mini_hud);
                            let _ = storage::save_settings(&self.settings);
                        }
                    });

                    ui.add_space(8.0);
                    if ui.button(Strings::btn_move_now(lang)).clicked() {
                        move_window_to_monitor(&self.settings.locked_monitor, self.is_mini_hud);
                    }

                    ui.add_space(8.0);
                    if let Some((x, y)) = self.settings.custom_pos_mini {
                        ui.label(RichText::new(Strings::lbl_saved_pos(lang, x, y)).size(12.0).color(self.theme.accent));
                        if ui.button(Strings::btn_reset_pos(lang)).clicked() {
                            self.settings.custom_pos_mini = None;
                            move_window_to_monitor(&self.settings.locked_monitor, self.is_mini_hud);
                            let _ = storage::save_settings(&self.settings);
                        }
                    } else {
                        let default_text = match lang {
                            Language::Norwegian => "📌 Posisjon: Standard hjørne (følger valgt skjerm)",
                            Language::English => "📌 Position: Default corner (follows selected display)",
                        };
                        ui.label(RichText::new(default_text).size(12.0).color(self.theme.text_muted));
                    }
                });

            ui.add_space(10.0);

            // Seksjon: Varsler
            egui::Frame::none()
                .fill(self.theme.bg_panel)
                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(Strings::sec_notifications(lang)).strong().size(13.0).color(self.theme.accent));
                    ui.add_space(6.0);
                    ui.checkbox(&mut self.settings.notifications_enabled, Strings::chk_notifications(lang));
                    ui.checkbox(&mut self.settings.remind_ramdump, Strings::chk_remind_ramdump(lang));
                });

            ui.add_space(14.0);

            // Lagre-knapp
            let save_btn = egui::Button::new(
                RichText::new(Strings::btn_save(lang)).size(14.0).strong().color(self.theme.text_primary),
            )
            .fill(self.theme.accent)
            .rounding(8.0)
            .min_size(Vec2::new(ui.available_width(), 38.0));

            if ui.add(save_btn).clicked() {
                self.flow.focus_ratio = self.settings.ratio_focus;
                self.flow.break_ratio = self.settings.ratio_break;
                self.flow.seq += 1;
                self.flow.broadcast_state();
                self.impulse.reload(&self.settings);
                let _ = storage::save_settings(&self.settings);
                self.save_feedback = Some(Strings::save_success(lang).to_string());
            }

            if let Some(msg) = &self.save_feedback {
                ui.add_space(6.0);
                ui.label(RichText::new(msg).color(self.theme.focus_green).size(12.0));
            }
        });
    }
}
