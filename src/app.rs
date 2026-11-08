use std::time::Instant;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Stroke, Vec2};

use crate::flowmodoro::{FlowState, Flowmodoro};
use crate::impulse::ImpulseState;
use crate::ramdump::RamDumpState;

#[derive(PartialEq, Eq, Clone, Copy)]
enum ActiveTab {
    Flowmodoro,
    RamDump,
    Impulses,
    YakShaving,
}

pub struct TetherApp {
    flow: Flowmodoro,
    ramdump: RamDumpState,
    impulse: ImpulseState,
    active_tab: ActiveTab,
    always_on_top: bool,
    is_mini_hud: bool,
    last_update: Instant,
    yak_jabs: Vec<&'static str>,
    current_yak_idx: usize,
}

impl Default for TetherApp {
    fn default() -> Self {
        Self {
            flow: Flowmodoro::new(),
            ramdump: RamDumpState::new(),
            impulse: ImpulseState::new(),
            active_tab: ActiveTab::Flowmodoro,
            always_on_top: true,
            is_mini_hud: true,
            last_update: Instant::now(),
            current_yak_idx: 0,
            yak_jabs: vec![
                "Å tweake verktøyene dine teller dessverre ikke som pensumlesing.",
                "Jada, koden kjører sikkert mye raskere bare du bytter fargetema én gang til.",
                "Gratulerer! Du har nå brukt 40 minutter på å automatisere en 3-sekunders oppgave.",
                "Den jaken du barberer nå har vært naken i over en halvtime.",
                "Du skulle fikse én feilmelding. Hvorfor refaktorerer du hele prosjektet nå?",
                "Du er fire lag dypere inn i et kaninhull enn da du startet. Pust og gå tilbake til kjerneoppgaven.",
                "Dette er ikke produktivitet. Dette er prokrastinering i smoking.",
                "Hjernen din: 'Dette er superviktig og haster!' Realiteten: 'Du vasker tastaturet med q-tips'.",
                "Nei, du trenger ikke skrive ditt eget rammeverk før du løser denne oppgaven.",
                "Arkitekturen er fin nok nå. Skriv den faktiske koden.",
            ],
        }
    }
}

impl TetherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Last JetBrains Mono hvis tilgjengelig på systemet
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

        // Ethereal Theme Palette
        let bg_base = Color32::from_rgb(6, 11, 30);
        let panel_bg = Color32::from_rgb(10, 16, 40);
        let card_bg = Color32::from_rgb(16, 24, 56);
        let border_subtle = Color32::from_rgba_premultiplied(109, 125, 182, 60);
        let accent = Color32::from_rgb(125, 130, 217);
        let text_primary = Color32::from_rgb(255, 235, 220);

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = bg_base;
        visuals.panel_fill = panel_bg;
        visuals.faint_bg_color = card_bg;
        visuals.extreme_bg_color = Color32::from_rgb(4, 7, 20);
        visuals.window_stroke = Stroke::new(1.0_f32, border_subtle);

        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);

        visuals.widgets.inactive.bg_fill = Color32::from_rgb(19, 26, 58);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(37, 46, 86);
        visuals.widgets.active.bg_fill = accent;

        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(175, 185, 215));
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, text_primary);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        visuals.selection.bg_fill = Color32::from_rgb(37, 46, 86);
        visuals.selection.stroke = Stroke::new(1.0_f32, accent);

        cc.egui_ctx.set_visuals(visuals);

        Self::default()
    }

    fn toggle_hud_mode(&mut self, ctx: &egui::Context) {
        self.is_mini_hud = !self.is_mini_hud;
        if self.is_mini_hud {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(260.0, 46.0)));
            #[cfg(target_os = "linux")]
            hyprland_shrink();
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(460.0, 580.0)));
            #[cfg(target_os = "linux")]
            hyprland_expand();
        }
    }
}

fn hyprland_expand() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            std::thread::spawn(|| {
                let _ = std::process::Command::new("hyprctl")
                    .args([
                        "dispatch",
                        "hl.dsp.window.resize({ x = 460, y = 580, relative = false, window = \"class:tether\" })",
                    ])
                    .output();
                let _ = std::process::Command::new("hyprctl")
                    .args([
                        "dispatch",
                        "hl.dsp.window.center({ window = \"class:tether\" })",
                    ])
                    .output();
            });
        }
    }
}

fn hyprland_shrink() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            std::thread::spawn(|| {
                let target_x = get_current_monitor_top_right_x();
                let _ = std::process::Command::new("hyprctl")
                    .args([
                        "dispatch",
                        "hl.dsp.window.resize({ x = 260, y = 46, relative = false, window = \"class:tether\" })",
                    ])
                    .output();
                let move_cmd = format!(
                    "hl.dsp.window.move({{ x = {}, y = 48, relative = false, window = \"class:tether\" }})",
                    target_x
                );
                let _ = std::process::Command::new("hyprctl")
                    .args(["dispatch", &move_cmd])
                    .output();
            });
        }
    }
}

fn get_current_monitor_top_right_x() -> i32 {
    let clients_out = std::process::Command::new("hyprctl").args(["clients", "-j"]).output();
    let mon_id = if let Ok(out) = clients_out {
        if let Ok(clients) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
            clients.as_array().and_then(|arr| {
                arr.iter().find(|c| c["class"] == "tether").and_then(|c| c["monitor"].as_i64())
            }).unwrap_or(0)
        } else { 0 }
    } else { 0 };

    if let Ok(out) = std::process::Command::new("hyprctl").args(["monitors", "-j"]).output() {
        if let Ok(mons) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
            if let Some(arr) = mons.as_array() {
                if let Some(m) = arr.iter().find(|m| m["id"] == mon_id) {
                    let x = m["x"].as_f64().unwrap_or(0.0);
                    let w = m["width"].as_f64().unwrap_or(1920.0);
                    let scale = m["scale"].as_f64().unwrap_or(1.0);
                    let logical_w = w / scale;
                    return (x + logical_w - 284.0) as i32;
                }
            }
        }
    }
    1636
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

        // Alltid oppdater med jevne mellomrom så vi fanger opp synkronisering fra Neovim
        ctx.request_repaint_after(std::time::Duration::from_millis(300));

        // Global tastatursnarvei (Mellomrom = Toggle Flowmodoro)
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) && !ctx.wants_keyboard_input() {
            self.flow.toggle();
        }

        // Escape krymper fullt dashboard tilbake til Mini-HUD
        if !self.is_mini_hud && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.toggle_hud_mode(ctx);
        }

        // Mini-HUD modus (kompakt flytende pill i hjørnet)
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
        let (border_color, dot_color, aura_color, text_color, badge_bg, tag) = match self.flow.state {
            FlowState::Focus => (
                Color32::from_rgb(120, 225, 155),
                Color32::from_rgb(120, 225, 155),
                Color32::from_rgba_premultiplied(120, 225, 155, 60),
                Color32::from_rgb(230, 255, 240),
                Color32::from_rgba_premultiplied(20, 60, 40, 200),
                "FOKUS",
            ),
            FlowState::Break => (
                Color32::from_rgb(237, 91, 90),
                Color32::from_rgb(237, 91, 90),
                Color32::from_rgba_premultiplied(237, 91, 90, 60),
                Color32::from_rgb(255, 230, 230),
                Color32::from_rgba_premultiplied(70, 25, 30, 200),
                "PAUSE",
            ),
            FlowState::Idle => (
                Color32::from_rgba_premultiplied(109, 125, 182, 90),
                Color32::from_rgb(109, 125, 182),
                Color32::from_rgba_premultiplied(109, 125, 182, 40),
                Color32::from_rgb(190, 195, 220),
                Color32::from_rgba_premultiplied(25, 32, 60, 200),
                "KLAR",
            ),
        };

        let stroke_width = if self.flow.is_active() { 1.5_f32 } else { 1.0_f32 };

        let panel = egui::CentralPanel::default().frame(
            egui::Frame::none()
                .fill(Color32::from_rgba_premultiplied(8, 12, 28, 235))
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

                let time_resp = ui.add(time_btn).on_hover_text("Klikk for å veksle fokus/pause (Mellomrom)");
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
                        RichText::new("⛶").size(15.0).color(Color32::from_rgb(200, 210, 240)),
                    )
                    .min_size(Vec2::new(26.0, 26.0))
                    .rounding(6.0)
                    .fill(Color32::from_rgba_premultiplied(37, 46, 86, 150));

                    if ui.add(expand_btn).on_hover_text("Åpne/lukk fullt dashboard (Dobbeltklikk virker også)").clicked() {
                        self.toggle_hud_mode(ctx);
                    }

                    // Handlingsikon (Start/Pause hurtigknapp)
                    let action_icon = match self.flow.state {
                        FlowState::Idle => "▶",
                        FlowState::Focus => "☕",
                        FlowState::Break => "⏹",
                    };
                    let act_btn = egui::Button::new(
                        RichText::new(action_icon).size(13.0).color(border_color),
                    )
                    .min_size(Vec2::new(24.0, 24.0))
                    .frame(false);
                    if ui.add(act_btn).on_hover_text("Skift modus").clicked() {
                        self.flow.toggle();
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
                ui.painter().rect_filled(bar_rect, 1.0, border_color);
            }
        });

        // Dobbeltklikk hvor som helst på pillen ekspanderer også
        if res.response.double_clicked() {
            self.toggle_hud_mode(ctx);
        }

        res.response.context_menu(|ui| {
            if ui.button("⛶ Åpne fullt panel").clicked() {
                self.toggle_hud_mode(ctx);
                ui.close_menu();
            }
            if ui.button("⏹ Nullstill økt").clicked() {
                self.flow.reset();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("✕ Avslutt Tether").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                ui.close_menu();
            }
        });
    }

    fn render_full_dashboard(&mut self, ctx: &egui::Context) {
        let (border_color, status_text) = match self.flow.state {
            FlowState::Focus => (Color32::from_rgb(120, 225, 155), "FOKUS"),
            FlowState::Break => (Color32::from_rgb(237, 91, 90), "PAUSE"),
            FlowState::Idle => (Color32::from_rgb(125, 130, 217), "KLAR"),
        };

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(6, 11, 30))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(109, 125, 182, 80)))
                    .rounding(16.0),
            )
            .show(ctx, |ui| {
                // Header
                egui::Frame::none()
                    .fill(Color32::from_rgb(10, 16, 40))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(109, 125, 182, 50)))
                    .rounding(egui::Rounding { nw: 16.0, ne: 16.0, sw: 0.0, se: 0.0 })
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let header_title = ui.heading(
                                RichText::new("⚓ TETHER")
                                    .strong()
                                    .color(Color32::from_rgb(255, 235, 220)),
                            );
                            if header_title.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                            }
                            if ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) && header_title.hovered() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                            }

                            // Status chip
                            egui::Frame::none()
                                .fill(Color32::from_rgba_premultiplied(37, 46, 86, 180))
                                .stroke(Stroke::new(1.0_f32, border_color))
                                .rounding(4.0)
                                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(status_text).size(10.0).strong().color(border_color));
                                });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Lukk
                                if ui
                                    .button(RichText::new("✕").size(14.0).color(Color32::from_rgb(180, 190, 220)))
                                    .on_hover_text("Lukk programmet")
                                    .clicked()
                                {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }

                                // Krymp til Mini-HUD
                                let mini_btn = egui::Button::new(
                                    RichText::new("🗕 Mini-HUD")
                                        .size(11.5)
                                        .strong()
                                        .color(Color32::from_rgb(125, 130, 217)),
                                );
                                if ui.add(mini_btn).on_hover_text("Minimer til flytende hjørne-HUD (Esc)").clicked() {
                                    self.toggle_hud_mode(ctx);
                                }

                                // Fest øverst
                                if ui.checkbox(&mut self.always_on_top, "📌 Alltid øverst").changed() {
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

                        // Segmentert fane-velger (Pill Tabs)
                        ui.horizontal(|ui| {
                            self.render_tab_pill(ui, ActiveTab::Flowmodoro, "⏱ Flowmodoro");
                            self.render_tab_pill(ui, ActiveTab::RamDump, "🧠 RAM-Dump");
                            self.render_tab_pill(ui, ActiveTab::Impulses, "💡 Side-Quests");
                            self.render_tab_pill(ui, ActiveTab::YakShaving, "🪒 Yak-Alarm");
                        });
                    });

                // Hovedinnhold i fane
                ui.add_space(10.0);
                match self.active_tab {
                    ActiveTab::Flowmodoro => self.render_flowmodoro(ui),
                    ActiveTab::RamDump => self.render_ramdump(ui),
                    ActiveTab::Impulses => self.render_impulses(ui),
                    ActiveTab::YakShaving => self.render_yak_shaving(ui),
                }
            });
    }

    fn render_tab_pill(&mut self, ui: &mut egui::Ui, tab: ActiveTab, label: &str) {
        let is_selected = self.active_tab == tab;
        let (bg, text_color, stroke) = if is_selected {
            (
                Color32::from_rgb(37, 46, 86),
                Color32::from_rgb(255, 235, 220),
                Stroke::new(1.0_f32, Color32::from_rgb(125, 130, 217)),
            )
        } else {
            (
                Color32::from_rgba_premultiplied(16, 24, 56, 120),
                Color32::from_rgb(150, 160, 190),
                Stroke::NONE,
            )
        };

        let btn = egui::Button::new(RichText::new(label).size(12.0).strong().color(text_color))
            .fill(bg)
            .stroke(stroke)
            .rounding(6.0)
            .min_size(Vec2::new(95.0, 26.0));

        if ui.add(btn).clicked() {
            self.active_tab = tab;
        }
    }

    fn render_flowmodoro(&mut self, ui: &mut egui::Ui) {
        let (status_text, border_color, text_color) = match self.flow.state {
            FlowState::Focus => ("FOKUSØKT (FLYTSONE)", Color32::from_rgb(120, 225, 155), Color32::from_rgb(120, 225, 155)),
            FlowState::Break => ("PAUSE (HVILE)", Color32::from_rgb(237, 91, 90), Color32::from_rgb(237, 91, 90)),
            FlowState::Idle => ("KLAR TIL NY ØKT", Color32::from_rgb(125, 130, 217), Color32::from_rgb(180, 190, 220)),
        };

        ui.vertical_centered(|ui| {
            ui.add_space(6.0);

            // Hovedkort med klokke og status
            egui::Frame::none()
                .fill(Color32::from_rgb(12, 18, 46))
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
                        FlowState::Focus => {
                            let earned = self.flow.calculate_break_time();
                            format!("Opptjent pause hittil: {} (45:15 forhold)", crate::flowmodoro::format_time(earned))
                        }
                        FlowState::Break => "Husk å ta en 60s RAM-dump og strekke beina!".to_string(),
                        FlowState::Idle => "Jobb så lenge du har flyt. Pausen beregnes automatisk.".to_string(),
                    };
                    ui.label(RichText::new(sub_info).size(11.5).color(Color32::from_rgb(160, 170, 200)));
                });

            ui.add_space(18.0);

            // Handlingsknapper
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 4.5);

                let (btn_text, btn_fill, btn_fg) = match self.flow.state {
                    FlowState::Idle => ("▶ Start Flytsone", Color32::from_rgb(35, 90, 55), Color32::from_rgb(230, 255, 240)),
                    FlowState::Focus => ("☕ Ta Pause", Color32::from_rgb(110, 35, 40), Color32::from_rgb(255, 230, 230)),
                    FlowState::Break => ("⏹ Avslutt Pause", Color32::from_rgb(37, 46, 86), Color32::from_rgb(255, 235, 220)),
                };

                let main_btn = egui::Button::new(RichText::new(btn_text).size(15.0).strong().color(btn_fg))
                    .fill(btn_fill)
                    .rounding(10.0)
                    .min_size(Vec2::new(160.0, 42.0));

                if ui.add(main_btn).clicked() {
                    self.flow.toggle();
                }

                if self.flow.is_active() {
                    let reset_btn = egui::Button::new(RichText::new("↺ Nullstill").size(13.0).color(Color32::from_rgb(180, 190, 210)))
                        .fill(Color32::from_rgb(25, 32, 60))
                        .rounding(10.0)
                        .min_size(Vec2::new(90.0, 42.0));
                    if ui.add(reset_btn).clicked() {
                        self.flow.reset();
                    }
                }
            });

            ui.add_space(14.0);
            ui.label(
                RichText::new("Snarvei: Trykk Mellomrom når som helst for å veksle.")
                    .size(11.0)
                    .color(Color32::from_rgb(120, 130, 160)),
            );

            ui.add_space(18.0);

            // Forholdstall og dagens statistikk
            egui::Frame::none()
                .fill(Color32::from_rgb(12, 18, 46))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(109, 125, 182, 50)))
                .rounding(12.0)
                .inner_margin(14.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Forholdstall:").size(12.0).strong());
                        ui.add(egui::DragValue::new(&mut self.flow.focus_ratio).range(1..=120).suffix("m fokus"));
                        ui.label(":");
                        ui.add(egui::DragValue::new(&mut self.flow.break_ratio).range(1..=60).suffix("m pause"));
                    });

                    ui.add_space(6.0);
                    let total_mins = self.flow.total_focus_secs / 60;
                    ui.label(
                        RichText::new(format!("Total fokus i dag: {} minutter", total_mins))
                            .size(11.5)
                            .color(Color32::from_rgb(120, 225, 155)),
                    );
                });
        });
    }

    fn render_ramdump(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("🧠 60-sekunders RAM-Dump").color(Color32::from_rgb(255, 235, 220)));
        ui.label(
            RichText::new("Frys tilstanden i arbeidsminnet før du forlater pulten, så slipper du oppstartsparalyse etterpå.")
                .size(11.5)
                .color(Color32::from_rgb(160, 170, 200)),
        );
        ui.add_space(12.0);

        egui::Frame::none()
            .fill(Color32::from_rgb(12, 18, 46))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(109, 125, 182, 50)))
            .rounding(12.0)
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.label(RichText::new("[NÅ] Hva holder du på med akkurat nå?").size(12.0).strong().color(Color32::from_rgb(125, 130, 217)));
                ui.add_sized([ui.available_width(), 26.0], egui::TextEdit::singleline(&mut self.ramdump.now).hint_text("f.eks. Skriver ferdig sql-migrering"));
                ui.add_space(8.0);

                ui.label(RichText::new("[STATUS] Hva mangler / hvor stoppet det?").size(12.0).strong().color(Color32::from_rgb(125, 130, 217)));
                ui.add_sized([ui.available_width(), 26.0], egui::TextEdit::singleline(&mut self.ramdump.status).hint_text("f.eks. Må legge til foreign key constraint"));
                ui.add_space(8.0);

                ui.label(RichText::new("[NESTE MEKANISKE STEG] Aller første tastetrykk ved retur:").size(12.0).strong().color(Color32::from_rgb(120, 225, 155)));
                ui.add_sized([ui.available_width(), 26.0], egui::TextEdit::singleline(&mut self.ramdump.next_step).hint_text("f.eks. Kjør 'cargo test --lib'"));
                ui.add_space(12.0);

                let save_btn = egui::Button::new(
                    RichText::new("💾 Forankre Tilstand").size(14.0).strong().color(Color32::from_rgb(255, 235, 220)),
                )
                .fill(Color32::from_rgb(37, 46, 86))
                .rounding(8.0)
                .min_size(Vec2::new(ui.available_width(), 36.0));

                if ui.add(save_btn).clicked() {
                    let _ = self.ramdump.save();
                }

                if let Some(msg) = &self.ramdump.feedback_message {
                    ui.add_space(6.0);
                    ui.label(RichText::new(msg).color(Color32::from_rgb(120, 225, 155)).size(12.0));
                }
            });

        if let Some((time, now, status, next)) = &self.ramdump.last_dump {
            ui.add_space(14.0);
            ui.label(RichText::new(format!("Forrige forankring (kl. {}):", time)).strong().size(12.0));

            egui::Frame::none()
                .fill(Color32::from_rgb(10, 15, 38))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(120, 225, 155, 60)))
                .rounding(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("• Nå: {}", now)).color(Color32::from_rgb(255, 240, 230)));
                    ui.label(RichText::new(format!("• Status: {}", status)).color(Color32::from_rgb(180, 190, 215)));
                    ui.label(RichText::new(format!("• Neste mekaniske steg: {}", next)).strong().color(Color32::from_rgb(120, 225, 155)));
                });
        }
    }

    fn render_impulses(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("💡 Side-Quest & Impulsbuffer").color(Color32::from_rgb(255, 235, 220)));
        ui.label(
            RichText::new("Dukket det opp en tilfeldig tanke? Fang den her slik at du kan holde fokus på kjerneoppgaven.")
                .size(11.5)
                .color(Color32::from_rgb(160, 170, 200)),
        );
        ui.add_space(12.0);

        // Input card
        ui.horizontal(|ui| {
            let input = ui.add_sized(
                [ui.available_width() - 85.0, 34.0],
                egui::TextEdit::singleline(&mut self.impulse.input_text)
                    .hint_text("Skriv strøtanke her (f.eks. 'Sjekk e-post om flybilletter')..."),
            );

            let submitted = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let add_btn = egui::Button::new(RichText::new("+ Fang").strong().color(Color32::from_rgb(255, 235, 220)))
                .fill(Color32::from_rgb(37, 46, 86))
                .rounding(8.0)
                .min_size(Vec2::new(80.0, 34.0));
            let clicked = ui.add(add_btn).clicked();

            if submitted || clicked {
                let _ = self.impulse.capture();
            }
        });

        if let Some(feedback) = &self.impulse.feedback {
            ui.add_space(6.0);
            ui.label(RichText::new(feedback).color(Color32::from_rgb(120, 225, 155)).size(12.0));
        }

        ui.add_space(14.0);
        ui.label(RichText::new("Fangede impulser (venter til etter økten):").strong().size(12.0));

        let mut to_remove = None;
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            if self.impulse.items.is_empty() {
                ui.add_space(10.0);
                ui.label(RichText::new("Ingen lagrede impulser enda. Hodet ditt er rent! 🧘").italics().color(Color32::from_rgb(120, 130, 160)));
            } else {
                for (idx, item) in self.impulse.items.iter().enumerate().rev() {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(12, 18, 46))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(109, 125, 182, 40)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(item).size(12.5).color(Color32::from_rgb(230, 235, 250)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(RichText::new("✓").size(12.0).color(Color32::from_rgb(120, 225, 155))).on_hover_text("Fjern / Fullført").clicked() {
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
            let _ = self.impulse.remove(idx);
        }
    }

    fn render_yak_shaving(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("🪒 Yak-Shaving & Kaninhull-sjekk").color(Color32::from_rgb(255, 235, 220)));
        ui.label(
            RichText::new("En sunn dose realitetssjekk når du merker at du gjør alt annet enn det du egentlig burde gjøre:")
                .size(11.5)
                .color(Color32::from_rgb(160, 170, 200)),
        );
        ui.add_space(14.0);

        // Fremhevet sitat-kort
        let current_jab = self.yak_jabs[self.current_yak_idx % self.yak_jabs.len()];
        egui::Frame::none()
            .fill(Color32::from_rgb(14, 20, 52))
            .stroke(Stroke::new(1.5_f32, Color32::from_rgb(237, 91, 90)))
            .rounding(14.0)
            .inner_margin(18.0)
            .show(ui, |ui| {
                ui.label(RichText::new("“").size(32.0).color(Color32::from_rgb(237, 91, 90)));
                ui.label(RichText::new(current_jab).size(15.0).italics().strong().color(Color32::from_rgb(255, 240, 230)));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let next_btn = egui::Button::new(
                        RichText::new("🎲 Nytt Stikk").size(12.0).strong().color(Color32::from_rgb(255, 235, 220)),
                    )
                    .fill(Color32::from_rgb(37, 46, 86))
                    .rounding(6.0);

                    if ui.add(next_btn).clicked() {
                        self.current_yak_idx = (self.current_yak_idx + 1) % self.yak_jabs.len();
                    }

                    if ui.button(RichText::new("🚀 Tilbake til flyten!").size(12.0).strong().color(Color32::from_rgb(120, 225, 155))).clicked() {
                        self.active_tab = ActiveTab::Flowmodoro;
                    }
                });
            });

        ui.add_space(16.0);
        ui.label(RichText::new("Flere påminnelser:").strong().size(12.0));

        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            for (i, jab) in self.yak_jabs.iter().enumerate() {
                egui::Frame::none()
                    .fill(Color32::from_rgb(10, 15, 38))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(109, 125, 182, 30)))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(10.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{}.", i + 1)).color(Color32::from_rgb(237, 91, 90)).strong().size(11.5));
                            ui.label(RichText::new(*jab).size(11.5).color(Color32::from_rgb(200, 210, 230)));
                        });
                    });
                ui.add_space(3.0);
            }
        });
    }
}
