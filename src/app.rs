use std::time::Instant;
use eframe::egui::{self, Color32, RichText, Stroke};

use crate::flowmodoro::{FlowState, Flowmodoro};
use crate::impulse::ImpulseState;
use crate::ramdump::RamDumpState;

#[derive(PartialEq, Eq)]
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
    last_update: Instant,
    yak_jabs: Vec<&'static str>,
}

impl Default for TetherApp {
    fn default() -> Self {
        Self {
            flow: Flowmodoro::new(),
            ramdump: RamDumpState::new(),
            impulse: ImpulseState::new(),
            active_tab: ActiveTab::Flowmodoro,
            always_on_top: true,
            last_update: Instant::now(),
            yak_jabs: vec![
                "Å tweake verktøyene dine teller dessverre ikke som pensumlesing.",
                "Jada, koden kjører sikkert mye raskere bare du bytter fargetema én gang til.",
                "Gratulerer! Du har nå brukt 40 minutter på å automatisere en 3-sekunders oppgave.",
                "Den jaken du barberer nå har vært naken i over en halvtime.",
                "Du skulle fikse én feilmelding. Hvorfor refaktorerer du hele prosjektet nå?",
                "Du er fire lag dypere inn i et kaninhull enn da du startet. Pust og gå tilbake til kjerneoppgaven.",
                "Dette er ikke produktivitet. Dette er prokrastinering i smoking.",
                "Hjernen din: 'Dette er superviktig og haster!' Realiteten: 'Du vasker tastaturet med q-tips'.",
            ],
        }
    }
}

impl TetherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = Color32::from_rgb(18, 19, 23);
        visuals.panel_fill = Color32::from_rgb(22, 24, 30);
        visuals.window_stroke = Stroke::new(1.0_f32, Color32::from_rgb(45, 48, 58));
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
        visuals.widgets.active.rounding = egui::Rounding::same(6.0);
        cc.egui_ctx.set_visuals(visuals);

        Self::default()
    }
}

impl eframe::App for TetherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update);
        self.last_update = now;

        self.flow.tick(dt);

        if self.flow.is_active() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Space)) && !ctx.wants_keyboard_input() {
            self.flow.toggle();
        }

        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("⚓ Tether").strong().color(Color32::from_rgb(240, 240, 245)));
                ui.label(RichText::new("ADHD Flow & Save-State").color(Color32::from_rgb(140, 145, 160)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.always_on_top, "📌 Fest øverst").changed() {
                        let level = if self.always_on_top {
                            egui::WindowLevel::AlwaysOnTop
                        } else {
                            egui::WindowLevel::Normal
                        };
                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                    }
                });
            });
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, ActiveTab::Flowmodoro, "⏱ Flowmodoro");
                ui.selectable_value(&mut self.active_tab, ActiveTab::RamDump, "🧠 RAM-Dump");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Impulses, "💡 Side-Quests");
                ui.selectable_value(&mut self.active_tab, ActiveTab::YakShaving, "🪒 Yak-Alarm");
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                ActiveTab::Flowmodoro => self.render_flowmodoro(ui),
                ActiveTab::RamDump => self.render_ramdump(ui),
                ActiveTab::Impulses => self.render_impulses(ui),
                ActiveTab::YakShaving => self.render_yak_shaving(ui),
            }
        });
    }
}

impl TetherApp {
    fn render_flowmodoro(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            let (status_text, border_color, text_color) = match self.flow.state {
                FlowState::Focus => ("FOKUSØKT (FLYTSONE)", Color32::from_rgb(152, 195, 121), Color32::from_rgb(152, 195, 121)),
                FlowState::Break => ("PAUSE (HVILE)", Color32::from_rgb(224, 108, 117), Color32::from_rgb(224, 108, 117)),
                FlowState::Idle => ("KLAR TIL ØKT", Color32::from_rgb(120, 125, 140), Color32::from_rgb(180, 185, 200)),
            };

            ui.label(RichText::new(status_text).strong().size(13.0).color(border_color));
            ui.add_space(8.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(26, 28, 36))
                .stroke(Stroke::new(2.5_f32, border_color))
                .rounding(16.0)
                .inner_margin(egui::Margin::symmetric(36.0, 18.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(self.flow.current_time_display())
                            .size(46.0)
                            .monospace()
                            .strong()
                            .color(text_color),
                    );
                });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 4.0);

                let btn_text = match self.flow.state {
                    FlowState::Idle => "▶ Start Fokus",
                    FlowState::Focus => "☕ Ta Pause (45:15)",
                    FlowState::Break => "⏹ Avslutt Pause",
                };

                let btn = egui::Button::new(RichText::new(btn_text).size(16.0).strong())
                    .min_size(egui::vec2(160.0, 42.0));

                if ui.add(btn).clicked() {
                    self.flow.toggle();
                }

                if self.flow.is_active() {
                    let reset_btn = egui::Button::new(RichText::new("↺ Nullstill").size(14.0))
                        .min_size(egui::vec2(90.0, 42.0));
                    if ui.add(reset_btn).clicked() {
                        self.flow.reset();
                    }
                }
            });

            ui.add_space(16.0);
            ui.label(
                RichText::new("Tastatursnarvei: Trykk Mellomrom for å starte/skifte.")
                    .size(11.0)
                    .color(Color32::from_rgb(110, 115, 130)),
            );

            ui.add_space(24.0);
            ui.separator();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Pauseregel (Forholdstall):").size(12.0));
                ui.add(egui::DragValue::new(&mut self.flow.focus_ratio).range(1..=120).suffix("m fokus"));
                ui.label(":");
                ui.add(egui::DragValue::new(&mut self.flow.break_ratio).range(1..=60).suffix("m pause"));
            });
        });
    }

    fn render_ramdump(&mut self, ui: &mut egui::Ui) {
        ui.heading("🧠 60-sekunders RAM-Dump");
        ui.label(
            RichText::new("Frys tilstanden i arbeidsminnet før du forlater pulten, så du slipper oppstartsparalyse etterpå.")
                .size(12.0)
                .color(Color32::from_rgb(150, 155, 170)),
        );
        ui.add_space(10.0);

        ui.label(RichText::new("[NÅ]: Hva holder du på med akkurat nå?").strong());
        ui.text_edit_singleline(&mut self.ramdump.now);
        ui.add_space(6.0);

        ui.label(RichText::new("[STATUS]: Hvor langt kom du / hva mangler?").strong());
        ui.text_edit_singleline(&mut self.ramdump.status);
        ui.add_space(6.0);

        ui.label(RichText::new("[NESTE STEG]: Det aller første mekaniske steget ved retur:").strong());
        ui.text_edit_singleline(&mut self.ramdump.next_step);
        ui.add_space(12.0);

        if ui.button(RichText::new("💾 Lagre RAM-Dump").size(15.0).strong()).clicked() {
            let _ = self.ramdump.save();
        }

        if let Some(msg) = &self.ramdump.feedback_message {
            ui.add_space(6.0);
            ui.label(RichText::new(msg).color(Color32::from_rgb(152, 195, 121)).size(12.0));
        }

        if let Some((time, now, status, next)) = &self.ramdump.last_dump {
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(RichText::new(format!("Siste forankrede tilstand (kl. {}):", time)).strong());

            egui::Frame::none()
                .fill(Color32::from_rgb(26, 28, 36))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 65, 80)))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("• Nå: {}", now)).color(Color32::WHITE));
                    ui.label(RichText::new(format!("• Status: {}", status)).color(Color32::from_rgb(200, 200, 210)));
                    ui.label(RichText::new(format!("• Neste steg: {}", next)).strong().color(Color32::from_rgb(152, 195, 121)));
                });
        }
    }

    fn render_impulses(&mut self, ui: &mut egui::Ui) {
        ui.heading("💡 Side-Quest & Impulsbuffer");
        ui.label(
            RichText::new("Dukket det opp en tilfeldig tanke? Skriv den ned her og kom deg tilbake til kjerneoppgaven uten å spore av.")
                .size(12.0)
                .color(Color32::from_rgb(150, 155, 170)),
        );
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let input = ui.add_sized(
                [ui.available_width() - 90.0, 32.0],
                egui::TextEdit::singleline(&mut self.impulse.input_text)
                    .hint_text("Fang strøtanke (f.eks. 'Husk å betale regning')..."),
            );

            let submitted = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let clicked = ui.button(RichText::new("+ Fange").strong()).clicked();

            if submitted || clicked {
                let _ = self.impulse.capture();
            }
        });

        if let Some(feedback) = &self.impulse.feedback {
            ui.add_space(6.0);
            ui.label(RichText::new(feedback).color(Color32::from_rgb(152, 195, 121)).size(12.0));
        }

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("Tidligere fangede impulser:").strong());

        egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
            if self.impulse.items.is_empty() {
                ui.label(RichText::new("Ingen lagrede impulser enda.").italics().color(Color32::GRAY));
            } else {
                for item in self.impulse.items.iter().rev() {
                    ui.label(RichText::new(item).size(12.5));
                }
            }
        });
    }

    fn render_yak_shaving(&mut self, ui: &mut egui::Ui) {
        ui.heading("🪒 Yak-Shaving & Kaninhull-sjekk");
        ui.label(
            RichText::new("En realitetssjekk når du merker at du gjør alt annet enn det du egentlig skulle gjøre:")
                .size(12.0)
                .color(Color32::from_rgb(150, 155, 170)),
        );
        ui.add_space(12.0);

        for (i, jab) in self.yak_jabs.iter().enumerate() {
            egui::Frame::none()
                .fill(Color32::from_rgb(26, 28, 36))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 48, 60)))
                .rounding(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}.", i + 1)).color(Color32::from_rgb(224, 108, 117)).strong());
                        ui.label(RichText::new(*jab).italics().color(Color32::from_rgb(220, 220, 230)));
                    });
                });
            ui.add_space(6.0);
        }
    }
}
