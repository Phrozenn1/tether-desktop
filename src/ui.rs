use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use libadwaita::glib;
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, Clamp, ComboRow, EntryRow, HeaderBar,
    PreferencesGroup, PreferencesPage, SpinRow, SwitchRow, ViewStack, ViewSwitcher,
    ViewSwitcherBar,
};

use crate::audio::{SoundscapePlayer, SoundscapeType};
use crate::flowmodoro::{FlowState, Flowmodoro};
use crate::i18n::Strings;
use crate::impulse::ImpulseState;
use crate::ramdump::RamDumpState;
use crate::storage::{self, ContextAnchor, Language, TetherSettings, UserStats};

pub struct AppState {
    pub flow: Flowmodoro,
    pub audio: SoundscapePlayer,
    pub previous_state: FlowState,
    pub settings: TetherSettings,
    pub stats: UserStats,
    pub anchor: Option<ContextAnchor>,
    pub impulse: ImpulseState,
    pub ramdump: RamDumpState,
    pub last_update: Instant,
    pub last_anchor_check: Instant,
    pub last_stats_check: Instant,
}

pub struct TetherUi {
    pub state: Rc<RefCell<AppState>>,

    // Dynamic header widgets
    pub status_badge: gtk4::Label,
    pub audio_btn: gtk4::Button,
    pub nvim_status: gtk4::Button,

    // Dynamic flowmodoro widgets
    pub timer_label: gtk4::Label,
    pub hero_badge: gtk4::Label,
    pub sub_info_label: gtk4::Label,
    pub main_action_btn: gtk4::Button,
    pub micro_btn: gtk4::Button,
    pub reset_btn: gtk4::Button,
    pub ratio_label: gtk4::Label,
    pub today_stats_label: gtk4::Label,

    // Context anchor widgets
    pub anchor_row: ActionRow,
    pub anchor_resume_btn: gtk4::Button,

    // Momentum widgets
    pub momentum_tier_row: ActionRow,
    pub momentum_progress: gtk4::ProgressBar,
    pub stat_flow_val: gtk4::Label,
    pub stat_sessions_val: gtk4::Label,
    pub stat_impulses_val: gtk4::Label,

    // RAM-dump widgets
    pub ram_now_entry: EntryRow,
    pub ram_status_entry: EntryRow,
    pub ram_next_entry: EntryRow,
    pub last_dump_group: PreferencesGroup,
    pub last_dump_row: ActionRow,

    // Impulses widgets
    pub impulse_entry: EntryRow,
    pub impulses_list_box: gtk4::ListBox,
}

impl TetherUi {
    pub fn build(app: &Application) -> Rc<Self> {
        let settings = storage::load_settings();
        let initial_settings = settings.clone();
        let mut flow = Flowmodoro::new();
        flow.focus_ratio = settings.ratio_focus;
        flow.break_ratio = settings.ratio_break;
        let previous_state = flow.state;

        let audio = SoundscapePlayer::new(
            settings.soundscape,
            settings.soundscape_volume,
            settings.soundscape_auto_play,
        );
        let stats = storage::load_user_stats();
        let anchor = storage::load_context_anchor();
        let impulse = ImpulseState::new();
        let ramdump = RamDumpState::new();

        let state = Rc::new(RefCell::new(AppState {
            flow,
            audio,
            previous_state,
            settings,
            stats,
            anchor,
            impulse,
            ramdump,
            last_update: Instant::now(),
            last_anchor_check: Instant::now(),
            last_stats_check: Instant::now(),
        }));

        // Apply Custom CSS
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(
            "
            .timer-display {
                font-size: 48px;
                font-weight: 800;
                font-family: 'JetBrains Mono', 'Fira Code', monospace;
                letter-spacing: 2px;
                margin: 2px 0;
            }
            .badge-micro {
                background-color: alpha(#e5c07b, 0.22);
                color: #e5c07b;
                border-radius: 9999px;
                padding: 4px 14px;
                font-weight: bold;
                font-size: 12px;
            }
            .badge-focus {
                background-color: alpha(#98c379, 0.22);
                color: #98c379;
                border-radius: 9999px;
                padding: 4px 14px;
                font-weight: bold;
                font-size: 12px;
            }
            .badge-break {
                background-color: alpha(#e06c75, 0.22);
                color: #e06c75;
                border-radius: 9999px;
                padding: 4px 14px;
                font-weight: bold;
                font-size: 12px;
            }
            .badge-idle {
                background-color: alpha(currentColor, 0.08);
                color: alpha(currentColor, 0.65);
                border-radius: 9999px;
                padding: 4px 14px;
                font-weight: bold;
                font-size: 12px;
            }
            .hero-card {
                padding: 16px 16px;
                margin: 0 0 6px 0;
                border-radius: 14px;
            }
            .stat-pill {
                background-color: alpha(currentColor, 0.05);
                border-radius: 10px;
                padding: 10px 8px;
            }
            ",
        );
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Tether — ADHD Flow & Save-State")
            .default_width(740)
            .default_height(860)
            .build();

        let header_bar = HeaderBar::new();
        let view_stack = ViewStack::new();

        let view_switcher = ViewSwitcher::builder()
            .stack(&view_stack)
            .policy(libadwaita::ViewSwitcherPolicy::Wide)
            .build();
        header_bar.set_title_widget(Some(&view_switcher));

        // Left Header: Soundscape Toggle Button
        let audio_btn = gtk4::Button::builder()
            .icon_name("audio-volume-high-symbolic")
            .tooltip_text("Fokus-lyd (Støygenerator)")
            .build();
        audio_btn.add_css_class("flat");
        header_bar.pack_start(&audio_btn);

        // Status chip (kept in struct for tracking, but omitted from headerbar to keep clean)
        let status_badge = gtk4::Label::builder()
            .label("KLAR")
            .css_classes(["badge-idle"])
            .build();

        // Right Header: Neovim Quick Action
        let nvim_status = gtk4::Button::builder()
            .label("🟢 NVIM")
            .tooltip_text("Fokuser eller åpne Neovim")
            .build();
        nvim_status.add_css_class("flat");
        header_bar.pack_end(&nvim_status);

        // --- PAGE 1: Flowmodoro ---
        let flow_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();
        let flow_clamp = Clamp::builder()
            .maximum_size(600)
            .tightening_threshold(420)
            .build();
        let flow_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        flow_box.set_margin_top(8);
        flow_box.set_margin_bottom(12);
        flow_box.set_margin_start(16);
        flow_box.set_margin_end(16);

        // Hero Timer Card
        let hero_card = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        hero_card.add_css_class("card");
        hero_card.add_css_class("hero-card");
        hero_card.set_halign(gtk4::Align::Fill);

        let hero_badge = gtk4::Label::builder()
            .label("KLAR")
            .css_classes(["badge-idle"])
            .halign(gtk4::Align::Center)
            .build();
        hero_card.append(&hero_badge);

        let timer_label = gtk4::Label::builder()
            .label("00:00")
            .css_classes(["timer-display"])
            .halign(gtk4::Align::Center)
            .build();
        hero_card.append(&timer_label);

        let sub_info_label = gtk4::Label::builder()
            .label("Klar for fokusøkt. Jobb i flyt så lenge du orker.")
            .css_classes(["dim-label", "caption"])
            .halign(gtk4::Align::Center)
            .wrap(true)
            .justify(gtk4::Justification::Center)
            .build();
        hero_card.append(&sub_info_label);

        // Action Buttons Row
        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        btn_row.set_halign(gtk4::Align::Center);
        btn_row.set_margin_top(12);

        let main_action_btn = gtk4::Button::builder()
            .label("▶ Start Fokus")
            .css_classes(["pill", "suggested-action"])
            .width_request(140)
            .build();
        btn_row.append(&main_action_btn);

        let micro_btn = gtk4::Button::builder()
            .label("⚡ 5m Mikro-start")
            .tooltip_text("Start en uforpliktende 5-minutters økt for å bryte oppstartssperren")
            .css_classes(["pill"])
            .width_request(140)
            .build();
        btn_row.append(&micro_btn);

        let reset_btn = gtk4::Button::builder()
            .label("Nullstill")
            .css_classes(["flat", "pill"])
            .build();
        reset_btn.set_visible(false);
        btn_row.append(&reset_btn);

        hero_card.append(&btn_row);

        let hint_label = gtk4::Label::builder()
            .label("Mellomrom = Toggle Flowmodoro  •  Nettverkssynkronisert med Neovim")
            .css_classes(["dim-label", "caption"])
            .halign(gtk4::Align::Center)
            .margin_top(8)
            .build();
        hero_card.append(&hint_label);

        flow_box.append(&hero_card);

        // Ratio & Stats summary row
        let summary_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        summary_box.set_halign(gtk4::Align::Center);
        let ratio_label = gtk4::Label::builder()
            .label("Forhold: 45m fokus : 15m pause")
            .css_classes(["caption", "dim-label"])
            .build();
        let today_stats_label = gtk4::Label::builder()
            .label("Fokus i dag: 0 minutter")
            .css_classes(["caption", "accent"])
            .build();
        summary_box.append(&ratio_label);
        summary_box.append(&today_stats_label);
        flow_box.append(&summary_box);

        // Group 2: Context Anchor
        let anchor_group = PreferencesGroup::builder()
            .title("⚓ Kontekst-anker")
            .description("Husker nøyaktig hvor du slapp i Neovim")
            .build();

        let anchor_row = ActionRow::builder()
            .title("Ingen aktiv kontekst ennå")
            .subtitle("Åpne en kodefil i Neovim for å forankre flyten")
            .build();

        let anchor_resume_btn = gtk4::Button::builder()
            .label("Gjenoppta i Neovim 🚀")
            .css_classes(["suggested-action", "pill"])
            .valign(gtk4::Align::Center)
            .build();
        anchor_row.add_suffix(&anchor_resume_btn);
        anchor_group.add(&anchor_row);
        flow_box.append(&anchor_group);

        // Group 3: Momentum & Flyt-XP
        let momentum_group = PreferencesGroup::builder()
            .title("🔥 Momentum og Flyt-XP")
            .description("Positiv framdrift – ingen brutte streaks")
            .build();

        let momentum_tier_row = ActionRow::builder()
            .title("⚡ Gnist (0 XP)")
            .subtitle("Ny uke, blanke ark! Ta en mikro-start for å tenne gnisten")
            .build();
        momentum_group.add(&momentum_tier_row);

        let momentum_progress = gtk4::ProgressBar::builder()
            .fraction(0.0)
            .margin_start(12)
            .margin_end(12)
            .margin_bottom(8)
            .build();
        momentum_group.add(&momentum_progress);

        // 3 Stat Pills Row
        let stat_pill_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        stat_pill_row.set_homogeneous(true);
        stat_pill_row.set_margin_start(12);
        stat_pill_row.set_margin_end(12);
        stat_pill_row.set_margin_bottom(6);

        let pill1 = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        pill1.add_css_class("stat-pill");
        let stat_flow_val = gtk4::Label::builder().label("0m").css_classes(["title-4"]).build();
        let pill1_lbl = gtk4::Label::builder().label("Ukens flyt").css_classes(["caption", "dim-label"]).build();
        pill1.append(&stat_flow_val);
        pill1.append(&pill1_lbl);
        stat_pill_row.append(&pill1);

        let pill2 = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        pill2.add_css_class("stat-pill");
        let stat_sessions_val = gtk4::Label::builder().label("0").css_classes(["title-4"]).build();
        let pill2_lbl = gtk4::Label::builder().label("Fullførte økter").css_classes(["caption", "dim-label"]).build();
        pill2.append(&stat_sessions_val);
        pill2.append(&pill2_lbl);
        stat_pill_row.append(&pill2);

        let pill3 = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        pill3.add_css_class("stat-pill");
        let stat_impulses_val = gtk4::Label::builder().label("0").css_classes(["title-4"]).build();
        let pill3_lbl = gtk4::Label::builder().label("Parkerte impulser").css_classes(["caption", "dim-label"]).build();
        pill3.append(&stat_impulses_val);
        pill3.append(&pill3_lbl);
        stat_pill_row.append(&pill3);

        momentum_group.add(&stat_pill_row);
        flow_box.append(&momentum_group);

        flow_clamp.set_child(Some(&flow_box));
        flow_scroll.set_child(Some(&flow_clamp));
        let flow_stack_page = view_stack.add_titled(&flow_scroll, Some("flowmodoro"), "Flyt");
        flow_stack_page.set_icon_name(Some("alarm-symbolic"));

        // --- PAGE 2: RAM-dump ---
        let ram_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();
        let ram_clamp = Clamp::builder().maximum_size(600).tightening_threshold(420).build();
        let ram_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        ram_box.set_margin_top(16);
        ram_box.set_margin_bottom(24);
        ram_box.set_margin_start(16);
        ram_box.set_margin_end(16);

        let ram_group = PreferencesGroup::builder()
            .title("Kognitiv save-state")
            .description("Frys tankerekken din før en pause eller avbrytelse")
            .build();

        let ram_now_entry = EntryRow::builder()
            .title("[NÅ] Hva holder du på med?")
            .build();
        ram_group.add(&ram_now_entry);

        let ram_status_entry = EntryRow::builder()
            .title("[STATUS] Hvor stoppet du?")
            .build();
        ram_group.add(&ram_status_entry);

        let ram_next_entry = EntryRow::builder()
            .title("[NESTE STEG] Første handling ved gjenopptakelse")
            .build();
        ram_group.add(&ram_next_entry);

        let save_ram_btn = gtk4::Button::builder()
            .label("Lagre RAM-dump")
            .css_classes(["suggested-action", "pill"])
            .margin_top(10)
            .halign(gtk4::Align::Center)
            .width_request(160)
            .build();
        ram_group.add(&save_ram_btn);
        ram_box.append(&ram_group);

        let last_dump_group = PreferencesGroup::builder()
            .title("Forrige RAM-dump")
            .description("Siste lagrede tilstand")
            .build();
        let last_dump_row = ActionRow::builder()
            .title("Ingen lagret RAM-dump ennå")
            .build();
        last_dump_group.add(&last_dump_row);
        ram_box.append(&last_dump_group);

        ram_clamp.set_child(Some(&ram_box));
        ram_scroll.set_child(Some(&ram_clamp));
        let ram_stack_page = view_stack.add_titled(&ram_scroll, Some("ramdump"), "RAM");
        ram_stack_page.set_icon_name(Some("document-save-symbolic"));

        // --- PAGE 3: Side-quests / Impulser ---
        let imp_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();
        let imp_clamp = Clamp::builder().maximum_size(600).tightening_threshold(420).build();
        let imp_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        imp_box.set_margin_top(16);
        imp_box.set_margin_bottom(24);
        imp_box.set_margin_start(16);
        imp_box.set_margin_end(16);

        let imp_input_group = PreferencesGroup::builder()
            .title("Side-quest / Impulsbuffer")
            .description("Parker strøtanker og kaninhull umiddelbart uten å spore av")
            .build();

        let impulse_entry = EntryRow::builder()
            .title("Fange ny impuls / side-quest...")
            .show_apply_button(true)
            .build();
        imp_input_group.add(&impulse_entry);
        imp_box.append(&imp_input_group);

        let impulses_group = PreferencesGroup::builder()
            .title("Parkerte impulser")
            .description("Ting du kan undersøke i pausen eller etterpå")
            .build();

        let impulses_list_box = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();
        impulses_group.add(&impulses_list_box);
        imp_box.append(&impulses_group);

        imp_clamp.set_child(Some(&imp_box));
        imp_scroll.set_child(Some(&imp_clamp));
        let imp_stack_page = view_stack.add_titled(&imp_scroll, Some("impulses"), "Impulser");
        imp_stack_page.set_icon_name(Some("checkbox-checked-symbolic"));

        // --- PAGE 4: Settings ---
        let settings_page = PreferencesPage::new();

        // Group: Flowmodoro
        let set_flow_group = PreferencesGroup::builder()
            .title("Flowmodoro")
            .description("Forholdstall mellom uavbrutt fokus og opptjent pause")
            .build();

        let focus_ratio_spin = SpinRow::builder()
            .title("Fokus-enhet (minutter)")
            .adjustment(&gtk4::Adjustment::new(initial_settings.ratio_focus as f64, 1.0, 180.0, 1.0, 5.0, 0.0))
            .numeric(true)
            .build();
        set_flow_group.add(&focus_ratio_spin);

        let break_ratio_spin = SpinRow::builder()
            .title("Pause-enhet (minutter)")
            .adjustment(&gtk4::Adjustment::new(initial_settings.ratio_break as f64, 1.0, 60.0, 1.0, 5.0, 0.0))
            .numeric(true)
            .build();
        set_flow_group.add(&break_ratio_spin);
        settings_page.add(&set_flow_group);

        // Group: Audio
        let set_audio_group = PreferencesGroup::builder()
            .title("Fokus-lyd og Støygenerator")
            .description("Prosedyrisk støy generator for stabil kognitiv stimulering")
            .build();

        let sound_models = gtk4::StringList::new(&[
            "Av",
            "Brun støy (dyp ro)",
            "Rosa støy (balansert)",
            "Mildt regn",
        ]);
        let selected_sound_idx = match initial_settings.soundscape {
            SoundscapeType::Off => 0,
            SoundscapeType::BrownNoise => 1,
            SoundscapeType::PinkNoise => 2,
            SoundscapeType::GentleRain => 3,
        };
        let sound_combo = ComboRow::builder()
            .title("Lydtype")
            .model(&sound_models)
            .selected(selected_sound_idx)
            .build();
        set_audio_group.add(&sound_combo);

        let auto_sound_switch = SwitchRow::builder()
            .title("Start lyden automatisk ved fokus")
            .subtitle("Spiller støy under fokus og mikro-start, pauser i pausen")
            .active(initial_settings.soundscape_auto_play)
            .build();
        set_audio_group.add(&auto_sound_switch);
        settings_page.add(&set_audio_group);

        // Group: General
        let set_gen_group = PreferencesGroup::builder()
            .title("Generelt og Neovim")
            .build();

        let nvim_cmd_entry = EntryRow::builder()
            .title("Terminalkommando for Neovim")
            .text(&initial_settings.terminal_command)
            .build();
        set_gen_group.add(&nvim_cmd_entry);

        let auto_launch_switch = SwitchRow::builder()
            .title("Auto-start Tether fra Neovim")
            .active(initial_settings.auto_launch_desktop)
            .build();
        set_gen_group.add(&auto_launch_switch);
        settings_page.add(&set_gen_group);

        let set_stack_page = view_stack.add_titled(&settings_page, Some("settings"), "Oppsett");
        set_stack_page.set_icon_name(Some("preferences-system-symbolic"));

        // Main layout assembly
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.append(&header_bar);
        main_box.append(&view_stack);

        let view_switcher_bar = ViewSwitcherBar::builder()
            .stack(&view_stack)
            .reveal(false)
            .build();
        main_box.append(&view_switcher_bar);

        window.set_content(Some(&main_box));

        let ui = Rc::new(Self {
            state: state.clone(),
            status_badge,
            audio_btn,
            nvim_status,
            timer_label,
            hero_badge,
            sub_info_label,
            main_action_btn,
            micro_btn,
            reset_btn,
            ratio_label,
            today_stats_label,
            anchor_row,
            anchor_resume_btn,
            momentum_tier_row,
            momentum_progress,
            stat_flow_val,
            stat_sessions_val,
            stat_impulses_val,
            ram_now_entry,
            ram_status_entry,
            ram_next_entry,
            last_dump_group,
            last_dump_row,
            impulse_entry,
            impulses_list_box,
        });

        // Event: Main Action button (Toggle)
        {
            let ui_clone = ui.clone();
            ui.main_action_btn.connect_clicked(move |_| {
                ui_clone.toggle_flow();
            });
        }

        // Event: 5m Micro-start button
        {
            let ui_clone = ui.clone();
            ui.micro_btn.connect_clicked(move |_| {
                ui_clone.start_micro();
            });
        }

        // Event: Reset button
        {
            let ui_clone = ui.clone();
            ui.reset_btn.connect_clicked(move |_| {
                ui_clone.reset_flow();
            });
        }

        // Event: Audio toggle button in header
        {
            let ui_clone = ui.clone();
            ui.audio_btn.connect_clicked(move |_| {
                ui_clone.toggle_audio();
            });
        }

        // Event: Neovim header button
        {
            let state_clone = state.clone();
            ui.nvim_status.connect_clicked(move |_| {
                let cmd = state_clone.borrow().settings.terminal_command.clone();
                storage::open_or_focus_neovim(&cmd);
            });
        }

        // Event: Resume in Neovim button
        {
            let state_clone = state.clone();
            ui.anchor_resume_btn.connect_clicked(move |_| {
                let st = state_clone.borrow();
                if let Some(anchor) = &st.anchor {
                    storage::resume_in_neovim(anchor, &st.settings.terminal_command);
                }
            });
        }

        // Event: Save RAM-dump
        {
            let ui_clone = ui.clone();
            save_ram_btn.connect_clicked(move |_| {
                ui_clone.save_ramdump();
            });
        }

        // Event: Capture Impulse
        {
            let ui_clone = ui.clone();
            ui.impulse_entry.connect_apply(move |entry| {
                let text = entry.text().trim().to_string();
                if !text.is_empty() {
                    ui_clone.capture_impulse(&text);
                    entry.set_text("");
                }
            });
        }

        // Event: Settings changes
        {
            let state_clone = state.clone();
            focus_ratio_spin.connect_value_notify(move |spin| {
                let mut st = state_clone.borrow_mut();
                st.settings.ratio_focus = spin.value() as u32;
                st.flow.focus_ratio = st.settings.ratio_focus;
                st.flow.broadcast_state();
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            break_ratio_spin.connect_value_notify(move |spin| {
                let mut st = state_clone.borrow_mut();
                st.settings.ratio_break = spin.value() as u32;
                st.flow.break_ratio = st.settings.ratio_break;
                st.flow.broadcast_state();
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            sound_combo.connect_selected_notify(move |combo| {
                let mut st = state_clone.borrow_mut();
                let new_type = match combo.selected() {
                    1 => SoundscapeType::BrownNoise,
                    2 => SoundscapeType::PinkNoise,
                    3 => SoundscapeType::GentleRain,
                    _ => SoundscapeType::Off,
                };
                st.audio.set_sound(new_type);
                st.settings.soundscape = new_type;
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            auto_sound_switch.connect_active_notify(move |sw| {
                let mut st = state_clone.borrow_mut();
                st.settings.soundscape_auto_play = sw.is_active();
                st.audio.auto_play_on_focus = sw.is_active();
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            nvim_cmd_entry.connect_apply(move |entry| {
                let mut st = state_clone.borrow_mut();
                st.settings.terminal_command = entry.text().to_string();
                let _ = storage::save_settings(&st.settings);
            });
        }

        // Populate initial data
        ui.refresh_impulses_list();
        ui.update_display();

        // 500ms Timer Tick Loop (GTK4 MainContext)
        {
            let ui_clone = ui.clone();
            glib::timeout_add_local(Duration::from_millis(500), move || {
                ui_clone.tick();
                glib::ControlFlow::Continue
            });
        }

        window.present();
        ui
    }

    pub fn tick(&self) {
        let now = Instant::now();
        let dt = {
            let mut st = self.state.borrow_mut();
            let dt = now.duration_since(st.last_update);
            st.last_update = now;
            st.flow.tick(dt);

            let old = st.previous_state;
            let cur = st.flow.state;
            if old != cur {
                st.previous_state = cur;
                st.audio.handle_flow_state_transition(old, cur);

                // Stats update on state transition
                if old == FlowState::Focus && (cur == FlowState::Break || cur == FlowState::Idle) {
                    let focus_mins = st.flow.elapsed_focus_secs / 60;
                    if focus_mins > 0 {
                        st.stats.record_focus_minutes(focus_mins);
                        let _ = storage::save_user_stats(&st.stats);
                    }
                } else if old == FlowState::MicroStart && cur == FlowState::Focus {
                    st.stats.record_micro_start();
                    let _ = storage::save_user_stats(&st.stats);
                }
            }

            // Periodic external checks
            if now.duration_since(st.last_anchor_check) >= Duration::from_secs(2) {
                st.last_anchor_check = now;
                st.anchor = storage::load_context_anchor();
            }
            if now.duration_since(st.last_stats_check) >= Duration::from_secs(2) {
                st.last_stats_check = now;
                st.stats = storage::load_user_stats();
            }
            dt
        };

        let _ = dt;
        self.update_display();
    }

    pub fn update_display(&self) {
        let st = self.state.borrow();
        let lang = st.settings.language;

        // Timer text
        self.timer_label.set_label(&st.flow.current_time_display());

        // Status badges and colors
        let (tag_text, css_class, sub_info) = match st.flow.state {
            FlowState::MicroStart => {
                let rem = crate::flowmodoro::format_time(st.flow.micro_remaining_secs);
                (
                    "⚡ 5 MIN",
                    "badge-micro",
                    match lang {
                        Language::Norwegian => format!("⚡ Bare 5 min for å bryte dørstokkmila ({} gjenstår). Null skyldfølelse!", rem),
                        Language::English => format!("⚡ Just 5 mins to melt friction ({} left). Zero guilt!", rem),
                    },
                )
            }
            FlowState::Focus => {
                let earned = st.flow.calculate_break_time();
                let earned_str = crate::flowmodoro::format_time(earned);
                (
                    "FOKUS",
                    "badge-focus",
                    Strings::sub_focus(lang, &earned_str, st.flow.focus_ratio, st.flow.break_ratio),
                )
            }
            FlowState::Break => (
                "PAUSE",
                "badge-break",
                Strings::sub_break(lang).to_string(),
            ),
            FlowState::Idle => (
                "KLAR",
                "badge-idle",
                Strings::sub_idle(lang).to_string(),
            ),
        };

        self.status_badge.set_label(tag_text);
        self.status_badge.set_css_classes(&[css_class]);
        self.hero_badge.set_label(tag_text);
        self.hero_badge.set_css_classes(&[css_class]);
        self.sub_info_label.set_label(&sub_info);

        // Buttons
        match st.flow.state {
            FlowState::Idle => {
                self.main_action_btn.set_label(Strings::btn_start_focus(lang));
                self.main_action_btn.set_css_classes(&["pill", "suggested-action"]);
                self.micro_btn.set_visible(true);
                self.reset_btn.set_visible(false);
            }
            FlowState::MicroStart => {
                self.main_action_btn.set_label("✕ Avbryt (Null skyldfølelse)");
                self.main_action_btn.set_css_classes(&["pill", "destructive-action"]);
                self.micro_btn.set_visible(false);
                self.reset_btn.set_visible(false);
            }
            FlowState::Focus => {
                self.main_action_btn.set_label(Strings::btn_take_break(lang));
                self.main_action_btn.set_css_classes(&["pill", "destructive-action"]);
                self.micro_btn.set_visible(false);
                self.reset_btn.set_visible(true);
            }
            FlowState::Break => {
                self.main_action_btn.set_label(Strings::btn_end_break(lang));
                self.main_action_btn.set_css_classes(&["pill", "suggested-action"]);
                self.micro_btn.set_visible(false);
                self.reset_btn.set_visible(true);
            }
        }

        // Audio button state in header
        if st.audio.is_playing {
            self.audio_btn.set_icon_name("media-playback-pause-symbolic");
            self.audio_btn.add_css_class("accent");
        } else {
            self.audio_btn.set_icon_name("audio-volume-high-symbolic");
            self.audio_btn.remove_css_class("accent");
        }

        // Neovim quick status
        if storage::is_neovim_running() {
            self.nvim_status.set_label("🟢 NVIM");
        } else {
            self.nvim_status.set_label("🔌 NVIM");
        }

        // Summary row
        self.ratio_label.set_label(&format!("Forhold: {}m : {}m", st.flow.focus_ratio, st.flow.break_ratio));
        let total_mins = st.flow.total_focus_secs / 60;
        self.today_stats_label.set_label(&Strings::stats_focus_today(lang, total_mins));

        // Context Anchor update
        if let Some(anchor) = &st.anchor {
            self.anchor_row.set_title(&format!("📁 {} [{}]", anchor.project_name, anchor.branch));
            let note = if anchor.last_note.is_empty() {
                String::new()
            } else {
                format!("  •  💡 \"{}\"", anchor.last_note)
            };
            self.anchor_row.set_subtitle(&format!("📄 {}:{}{}", anchor.relative_path, anchor.line, note));
            self.anchor_resume_btn.set_visible(true);
        } else {
            self.anchor_row.set_title("Ingen aktiv kontekst ennå");
            self.anchor_row.set_subtitle("Åpne en kodefil i Neovim for å forankre flyten");
            self.anchor_resume_btn.set_visible(false);
        }

        // Momentum / XP update
        let tier = st.stats.tier();
        let tier_title = match lang {
            Language::Norwegian => tier.title_no(),
            Language::English => tier.title_en(),
        };
        self.momentum_tier_row.set_title(&format!("{} ({} XP)", tier_title, st.stats.xp));

        let cur_th = tier.current_threshold();
        let next_th = tier.next_threshold();
        let fraction = if next_th > cur_th {
            ((st.stats.xp.saturating_sub(cur_th)) as f64 / (next_th - cur_th) as f64).clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.momentum_progress.set_fraction(fraction);

        let hrs = st.stats.current_week_mins / 60;
        let mins = st.stats.current_week_mins % 60;
        let flow_str = if hrs > 0 { format!("{}t {}m", hrs, mins) } else { format!("{}m", mins) };
        self.stat_flow_val.set_label(&flow_str);
        self.stat_sessions_val.set_label(&st.stats.current_week_sessions.to_string());
        self.stat_impulses_val.set_label(&st.stats.current_week_impulses_captured.to_string());
    }

    pub fn toggle_flow(&self) {
        let mut st = self.state.borrow_mut();
        st.flow.toggle();
    }

    pub fn start_micro(&self) {
        let mut st = self.state.borrow_mut();
        st.flow.start_micro_start();
    }

    pub fn reset_flow(&self) {
        let mut st = self.state.borrow_mut();
        st.flow.reset();
    }

    pub fn toggle_audio(&self) {
        let mut st = self.state.borrow_mut();
        st.audio.toggle_play();
    }

    pub fn save_ramdump(&self) {
        let mut st = self.state.borrow_mut();
        st.ramdump.now = self.ram_now_entry.text().to_string();
        st.ramdump.status = self.ram_status_entry.text().to_string();
        st.ramdump.next_step = self.ram_next_entry.text().to_string();
        let _ = st.ramdump.save();

        if let Some((time, now, status, next)) = &st.ramdump.last_dump {
            self.last_dump_row.set_title(&format!("Dump kl. {}", time));
            self.last_dump_row.set_subtitle(&format!("NÅ: {} | STATUS: {} | NESTE: {}", now, status, next));
        }
    }

    pub fn capture_impulse(&self, text: &str) {
        {
            let mut st = self.state.borrow_mut();
            st.impulse.input_text = text.to_string();
            let settings = st.settings.clone();
            let _ = st.impulse.capture_with_settings(&settings);
            st.stats.record_impulse();
            let _ = storage::save_user_stats(&st.stats);
        }
        self.refresh_impulses_list();
    }

    pub fn refresh_impulses_list(&self) {
        let st = self.state.borrow();
        let items = storage::load_impulses_with_settings(&st.settings);

        while let Some(child) = self.impulses_list_box.first_child() {
            self.impulses_list_box.remove(&child);
        }

        if items.is_empty() {
            let row = ActionRow::builder()
                .title("Ingen registrerte impulser ennå 🎉")
                .subtitle("Skriv inn noe ovenfor for å parkere det med en gang.")
                .build();
            self.impulses_list_box.append(&row);
        } else {
            for item in items.iter().rev().take(15) {
                let row = ActionRow::builder()
                    .title(item)
                    .build();
                self.impulses_list_box.append(&row);
            }
        }
    }
}
