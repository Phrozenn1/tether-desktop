use crate::{context_engine, flowmodoro, storage};
use gtk4::prelude::*;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickCaptureMode {
    All,
    ImpulseOnly,
    RamDumpOnly,
}

impl QuickCaptureMode {
    pub fn from_args(args: &[String]) -> Self {
        if args.iter().any(|a| a == "--impulse") {
            Self::ImpulseOnly
        } else if args.iter().any(|a| a == "--ramdump") {
            Self::RamDumpOnly
        } else {
            Self::All
        }
    }
}

/// Format a concise status line from the shared Flowmodoro state.
fn format_flow_badge(is_no: bool) -> String {
    let Ok(shared) = flowmodoro::apply_shared_flow_action("status", "quick-capture") else {
        return if is_no {
            "Flyt: Hvile".to_string()
        } else {
            "Flow: Idle".to_string()
        };
    };

    match shared.state.as_str() {
        "micro_start" => format!(
            "🌱 {} ({})",
            if is_no { "Mikrostart" } else { "Micro-Start" },
            flowmodoro::format_time(300_u64.saturating_sub(
                chrono::Utc::now().timestamp().max(0) as u64 - shared.start_epoch.min(chrono::Utc::now().timestamp().max(0) as u64)
            ))
        ),
        "focus" => format!(
            "🔥 {} ({})",
            if is_no { "Fokus" } else { "Focus" },
            flowmodoro::format_time(shared.elapsed_focus_sec)
        ),
        "break" => format!(
            "☕ {} ({})",
            if is_no { "Pause" } else { "Break" },
            flowmodoro::format_time(shared.remaining_break_sec)
        ),
        _ => if is_no {
            "⏸ Klar for ny økt".to_string()
        } else {
            "⏸ Ready for session".to_string()
        },
    }
}

/// Run dedicated centered Quick-Capture window (via `tether --quick-capture`, `--impulse`, or `--ramdump`).
pub fn run_quick_capture_dialog(args: &[String]) {
    let mode = QuickCaptureMode::from_args(args);
    let settings = storage::load_settings();
    let is_no = matches!(settings.language.resolve(), storage::Language::Norwegian);

    let app = libadwaita::Application::builder()
        .application_id("org.tether.QuickCapture")
        .build();

    app.connect_activate(move |app| {
        let title = if is_no {
            "Tether Hurtigfangst"
        } else {
            "Tether Quick-Capture"
        };
        let subtitle = if is_no {
            "Esc for å lukke • Synkroniseres med Tether Desktop"
        } else {
            "Esc to close • Syncs with Tether Desktop"
        };

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title(title)
            .default_width(520)
            .resizable(false)
            .modal(true)
            .build();

        let toolbar_view = libadwaita::ToolbarView::new();
        let header_bar = libadwaita::HeaderBar::new();
        header_bar.set_show_end_title_buttons(true);
        let title_widget = libadwaita::WindowTitle::new(title, subtitle);
        header_bar.set_title_widget(Some(&title_widget));
        toolbar_view.add_top_bar(&header_bar);

        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.set_margin_top(16);
        root.set_margin_bottom(18);
        root.set_margin_start(20);
        root.set_margin_end(20);

        let feedback_label = gtk4::Label::new(None);
        feedback_label.add_css_class("error");
        feedback_label.set_halign(gtk4::Align::Start);
        feedback_label.set_visible(false);

        let mut primary_impulse_entry: Option<gtk4::Entry> = None;
        let mut primary_now_entry: Option<gtk4::Entry> = None;

        if matches!(mode, QuickCaptureMode::All | QuickCaptureMode::ImpulseOnly) {
            let imp_header = gtk4::Label::new(Some(if is_no {
                "⚡ Parkér impuls / avsporing"
            } else {
                "⚡ Park Impulse / Side-Quest"
            }));
            imp_header.add_css_class("heading");
            imp_header.set_halign(gtk4::Align::Start);
            root.append(&imp_header);

            let imp_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            let imp_entry = gtk4::Entry::new();
            imp_entry.set_hexpand(true);
            imp_entry.set_placeholder_text(Some(if is_no {
                "Skriv tanke og trykk Enter..."
            } else {
                "Capture thought and press Enter..."
            }));

            let imp_btn = gtk4::Button::with_label(if is_no {
                "Parkér (Enter)"
            } else {
                "Park (Enter)"
            });
            imp_btn.add_css_class("suggested-action");

            imp_row.append(&imp_entry);
            imp_row.append(&imp_btn);
            root.append(&imp_row);

            let save_impulse = {
                let imp_entry = imp_entry.clone();
                let feedback = feedback_label.clone();
                let app_ref = app.clone();
                let settings_clone = settings.clone();
                move || {
                    let text = imp_entry.text().trim().to_string();
                    if text.is_empty() {
                        feedback.set_text(if is_no {
                            "Skriv inn en kort impuls før du parkerer."
                        } else {
                            "Enter a short impulse before parking."
                        });
                        feedback.set_visible(true);
                        return;
                    }
                    if let Err(err) = storage::append_impulse_with_settings(&settings_clone, &text) {
                        feedback.set_text(&err.to_string());
                        feedback.set_visible(true);
                        return;
                    }
                    let mut stats = storage::load_user_stats();
                    stats.record_impulse();
                    let _ = storage::save_user_stats(&stats);
                    app_ref.quit();
                }
            };

            let save_on_activate = save_impulse.clone();
            imp_entry.connect_activate(move |_| save_on_activate());
            let save_on_click = save_impulse;
            imp_btn.connect_clicked(move |_| save_on_click());

            primary_impulse_entry = Some(imp_entry);
        }

        if mode == QuickCaptureMode::All {
            root.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
        }

        if matches!(mode, QuickCaptureMode::All | QuickCaptureMode::RamDumpOnly) {
            let ram_header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            let ram_header = gtk4::Label::new(Some(if is_no {
                "🧠 Hurtig RAM-Dump (Arbeidsminne)"
            } else {
                "🧠 Quick RAM-Dump (Save-State)"
            }));
            ram_header.add_css_class("heading");
            ram_header.set_halign(gtk4::Align::Start);
            ram_header.set_hexpand(true);

            let autofill_btn = gtk4::Button::with_label(if is_no {
                "✨ Fyll ut kontekst"
            } else {
                "✨ Auto-fill Context"
            });
            autofill_btn.add_css_class("flat");

            ram_header_row.append(&ram_header);
            ram_header_row.append(&autofill_btn);
            root.append(&ram_header_row);

            let make_field = |tag: &str, placeholder: &str| -> (gtk4::Box, gtk4::Entry) {
                let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
                let lbl = gtk4::Label::new(Some(tag));
                lbl.add_css_class("dim-label");
                lbl.set_width_chars(12);
                lbl.set_xalign(0.0);
                let entry = gtk4::Entry::new();
                entry.set_hexpand(true);
                entry.set_placeholder_text(Some(placeholder));
                row.append(&lbl);
                row.append(&entry);
                (row, entry)
            };

            let (now_row, now_entry) = make_field(
                if is_no { "[NÅ]" } else { "[NOW]" },
                if is_no { "Hva jobber du med akkurat nå?" } else { "What are you working on right now?" },
            );
            let (status_row, status_entry) = make_field(
                "[STATUS]",
                if is_no { "Hvor langt har du kommet / hva gjenstår?" } else { "Current state / findings / blockers" },
            );
            let (next_row, next_entry) = make_field(
                if is_no { "[NESTE STEG]" } else { "[NEXT STEP]" },
                if is_no { "Konkret første handling når du er tilbake" } else { "Concrete first action when returning" },
            );

            root.append(&now_row);
            root.append(&status_row);
            root.append(&next_row);

            let ram_actions = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            ram_actions.set_halign(gtk4::Align::End);
            let save_ram_btn = gtk4::Button::with_label(if is_no {
                "💾 Lagre RAM-dump"
            } else {
                "💾 Save RAM-dump"
            });
            if mode == QuickCaptureMode::RamDumpOnly {
                save_ram_btn.add_css_class("suggested-action");
            }
            ram_actions.append(&save_ram_btn);
            root.append(&ram_actions);

            // Non-blocking AI / heuristic context autofill
            {
                let btn_clone = autofill_btn.clone();
                let now_c = now_entry.clone();
                let status_c = status_entry.clone();
                let next_c = next_entry.clone();
                let settings_c = settings.clone();
                autofill_btn.connect_clicked(move |btn| {
                    let prev_label = btn.label().unwrap_or_default().to_string();
                    btn.set_sensitive(false);
                    btn.set_label(if is_no { "⏳ Analyserer..." } else { "⏳ Analyzing..." });

                    let anchor = storage::load_context_anchor();
                    let s = settings_c.clone();
                    let (tx, rx) = mpsc::channel();
                    std::thread::spawn(move || {
                        let suggestion = context_engine::suggest_ramdump_fields(anchor.as_ref(), &s);
                        let _ = tx.send(suggestion);
                    });

                    let btn_weak = btn_clone.downgrade();
                    let now_weak = now_c.downgrade();
                    let status_weak = status_c.downgrade();
                    let next_weak = next_c.downgrade();
                    gtk4::glib::timeout_add_local(Duration::from_millis(80), move || {
                        match rx.try_recv() {
                            Ok(suggestion) => {
                                if let Some(b) = btn_weak.upgrade() {
                                    b.set_sensitive(true);
                                    b.set_label(&prev_label);
                                }
                                if let Some(e) = now_weak.upgrade() {
                                    e.set_text(&suggestion.now);
                                }
                                if let Some(e) = status_weak.upgrade() {
                                    e.set_text(&suggestion.status);
                                }
                                if let Some(e) = next_weak.upgrade() {
                                    e.set_text(&suggestion.next_step);
                                }
                                gtk4::glib::ControlFlow::Break
                            }
                            Err(mpsc::TryRecvError::Empty) => gtk4::glib::ControlFlow::Continue,
                            Err(mpsc::TryRecvError::Disconnected) => {
                                if let Some(b) = btn_weak.upgrade() {
                                    b.set_sensitive(true);
                                    b.set_label(&prev_label);
                                }
                                gtk4::glib::ControlFlow::Break
                            }
                        }
                    });
                });
            }

            // Save RAM-dump handler
            {
                let now_c = now_entry.clone();
                let status_c = status_entry.clone();
                let next_c = next_entry.clone();
                let feedback = feedback_label.clone();
                let app_ref = app.clone();
                let do_save = move || {
                    let n = now_c.text().trim().to_string();
                    let s = status_c.text().trim().to_string();
                    let nx = next_c.text().trim().to_string();
                    if n.is_empty() && s.is_empty() && nx.is_empty() {
                        feedback.set_text(if is_no {
                            "Fyll ut minst ett RAM-dump-felt før lagring."
                        } else {
                            "Fill in at least one RAM-dump field before saving."
                        });
                        feedback.set_visible(true);
                        return;
                    }
                    if let Err(err) = storage::append_ramdump(&n, &s, &nx) {
                        feedback.set_text(&err.to_string());
                        feedback.set_visible(true);
                        return;
                    }
                    app_ref.quit();
                };
                let do_save_btn = do_save.clone();
                save_ram_btn.connect_clicked(move |_| do_save_btn());
                let do_save_enter = do_save;
                next_entry.connect_activate(move |_| do_save_enter());
            }

            primary_now_entry = Some(now_entry);
        }

        if mode == QuickCaptureMode::All {
            root.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

            let flow_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            let status_lbl = gtk4::Label::new(Some(&format_flow_badge(is_no)));
            status_lbl.add_css_class("dim-label");
            status_lbl.set_hexpand(true);
            status_lbl.set_halign(gtk4::Align::Start);

            let micro_btn = gtk4::Button::with_label(if is_no {
                "🌱 5m Mikrostart"
            } else {
                "🌱 5m Micro-Start"
            });
            let toggle_btn = gtk4::Button::with_label(if is_no {
                "⏯ Start / Pause Fokus"
            } else {
                "⏯ Toggle Focus"
            });

            flow_bar.append(&status_lbl);
            flow_bar.append(&micro_btn);
            flow_bar.append(&toggle_btn);
            root.append(&flow_bar);

            let app_micro = app.clone();
            let feedback_micro = feedback_label.clone();
            micro_btn.connect_clicked(move |_| {
                match flowmodoro::apply_shared_flow_action("micro_start", "quick-capture") {
                    Ok(_) => app_micro.quit(),
                    Err(e) => {
                        feedback_micro.set_text(&e);
                        feedback_micro.set_visible(true);
                    }
                }
            });

            let app_toggle = app.clone();
            let feedback_toggle = feedback_label.clone();
            toggle_btn.connect_clicked(move |_| {
                match flowmodoro::apply_shared_flow_action("toggle", "quick-capture") {
                    Ok(_) => app_toggle.quit(),
                    Err(e) => {
                        feedback_toggle.set_text(&e);
                        feedback_toggle.set_visible(true);
                    }
                }
            });
        }

        root.append(&feedback_label);

        toolbar_view.set_content(Some(&root));
        window.set_child(Some(&toolbar_view));

        let key_controller = gtk4::EventControllerKey::new();
        let app_esc = app.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                app_esc.quit();
                gtk4::glib::Propagation::Stop
            } else {
                gtk4::glib::Propagation::Proceed
            }
        });
        window.add_controller(key_controller);

        window.present();
        if let Some(entry) = primary_impulse_entry.as_ref().or(primary_now_entry.as_ref()) {
            entry.grab_focus();
        }

        #[cfg(target_os = "linux")]
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            let _ = std::process::Command::new("hyprctl")
                .args(["dispatch", "centerwindow"])
                .output();
            let _ = std::process::Command::new("hyprctl")
                .args(["dispatch", "focuswindow", "class:org.tether.QuickCapture"])
                .output();
        }
    });

    let empty_args: [&str; 0] = [];
    app.run_with_args(&empty_args);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_capture_mode_parses_flags() {
        assert_eq!(QuickCaptureMode::from_args(&[]), QuickCaptureMode::All);
        assert_eq!(
            QuickCaptureMode::from_args(&["--quick-capture".into()]),
            QuickCaptureMode::All
        );
        assert_eq!(
            QuickCaptureMode::from_args(&["--impulse".into()]),
            QuickCaptureMode::ImpulseOnly
        );
        assert_eq!(
            QuickCaptureMode::from_args(&["--ramdump".into()]),
            QuickCaptureMode::RamDumpOnly
        );
    }
}
