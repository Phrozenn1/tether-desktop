use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use libadwaita::glib;
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ComboRow, EntryRow, HeaderBar,
    PreferencesGroup, PreferencesPage, SpinRow, SwitchRow, ViewStack, ViewSwitcher,
};

use crate::audio::{SoundscapePlayer, SoundscapeType};
use crate::editor::EditorType;
use crate::flowmodoro::{FlowState, Flowmodoro};
use crate::i18n::Strings;
use crate::impulse::ImpulseState;
use crate::ramdump::RamDumpState;
use crate::storage::{self, ContextAnchor, Language, LanguageSetting, SideQuestFormat, TetherSettings, UserStats};

pub const FULL_WINDOW_WIDTH: i32 = 940;
pub const FULL_WINDOW_HEIGHT: i32 = 840;
pub const FULL_WINDOW_MIN_WIDTH: i32 = 900;
pub const FULL_WINDOW_MIN_HEIGHT: i32 = 480;

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
    pub last_pos_check: Instant,
    pub last_recorded_focus_mins: u64,
    pub is_mini_hud: bool,
    pub suppress_pos_ticks: u32,
    pub asked_editor_minimize_prompt: bool,
    pub last_theme_mtime: Option<std::time::SystemTime>,
}

pub struct TetherUi {
    pub state: Rc<RefCell<AppState>>,
    pub window: gtk4::ApplicationWindow,
    pub main_box: gtk4::Box,
    pub mini_hud_box: gtk4::Box,
    pub css_provider: gtk4::CssProvider,

    // Dynamic header widgets
    pub status_badge: gtk4::Label,
    pub audio_btn: gtk4::Button,
    pub editor_btn: gtk4::Button,
    pub pill_mode_btn: gtk4::Button,

    // Dynamic flowmodoro widgets
    pub timer_label: gtk4::Label,
    pub hero_badge: gtk4::Label,
    pub sub_info_label: gtk4::Label,
    pub main_action_btn: gtk4::Button,
    pub micro_btn: gtk4::Button,
    pub reset_btn: gtk4::Button,
    pub helper_text: gtk4::Label,
    pub ratio_label: gtk4::Label,
    pub today_stats_label: gtk4::Label,

    // Mini-HUD widgets
    pub mini_timer: gtk4::Label,
    pub mini_badge: gtk4::Label,
    pub mini_toggle_btn: gtk4::Button,
    pub mini_micro_btn: gtk4::Button,
    pub mini_audio_btn: gtk4::Button,
    pub mini_expand_btn: gtk4::Button,

    // Flow soundscape widgets
    pub flow_sound_dropdown: gtk4::DropDown,
    pub flow_audio_btn: gtk4::Button,
    pub flow_vol_btn: gtk4::Button,
    pub flow_vol_scale: gtk4::Scale,
    pub flow_vol_label: gtk4::Label,
    pub flow_sound_row: ActionRow,
    pub sound_combo: ComboRow,
    pub set_vol_scale: gtk4::Scale,

    // Context anchor widgets
    pub anchor_group: PreferencesGroup,
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
    pub set_file_row: ActionRow,
    pub set_format_combo: ComboRow,
    pub set_cleanup_combo: ComboRow,
    pub editor_combo: ComboRow,
    pub nvim_mgr_combo: ComboRow,
    pub ext_status_row: ActionRow,
    pub install_ext_btn: gtk4::Button,
    pub editor_minimize_combo: ComboRow,
    pub pos_row: ActionRow,
    pub tray: Option<Rc<crate::tray::TetherTray>>,
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

        // Apply Theme CSS (Omarchy live integration)
        let colors = crate::theme::load_omarchy_colors();
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(&crate::theme::generate_css(&colors));
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let initial_theme_mtime = crate::theme::get_colors_mtime();

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
            last_pos_check: Instant::now(),
            last_recorded_focus_mins: 0,
            is_mini_hud: false,
            suppress_pos_ticks: 0,
            asked_editor_minimize_prompt: false,
            last_theme_mtime: initial_theme_mtime,
        }));

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Tether — ADHD Flow & Save-State")
            .default_width(FULL_WINDOW_WIDTH)
            .default_height(FULL_WINDOW_HEIGHT)
            .build();
        window.set_size_request(FULL_WINDOW_MIN_WIDTH, FULL_WINDOW_MIN_HEIGHT);

        let header_bar = HeaderBar::new();
        let view_stack = ViewStack::new();
        view_stack.set_vexpand(true);
        view_stack.set_hexpand(true);

        let view_switcher = ViewSwitcher::builder()
            .stack(&view_stack)
            .policy(libadwaita::ViewSwitcherPolicy::Wide)
            .build();
        header_bar.set_title_widget(Some(&view_switcher));

        // Left Header: Soundscape Toggle Button
        let audio_btn = gtk4::Button::builder()
            .icon_name("audio-volume-high-symbolic")
            .tooltip_text("Fokus-lyd (Støygenerator)")
            .css_classes(["flat"])
            .build();
        header_bar.pack_start(&audio_btn);

        // Status chip
        let status_badge = gtk4::Label::builder()
            .label("KLAR")
            .css_classes(["badge-idle"])
            .build();

        // Right Header: Minimize to Pill Button (Mini-HUD)
        let pill_mode_btn = gtk4::Button::builder()
            .icon_name("window-minimize-symbolic")
            .tooltip_text("Minimer til pille (Mini-HUD) [Esc]")
            .css_classes(["flat"])
            .build();
        header_bar.pack_end(&pill_mode_btn);

        // Right Header: Editor Quick Action
        let editor_btn = gtk4::Button::builder()
            .icon_name("utilities-terminal-symbolic")
            .tooltip_text("Editor")
            .css_classes(["flat"])
            .build();
        header_bar.pack_end(&editor_btn);

        // --- PAGE 1: Flowmodoro ---
        let flow_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .propagate_natural_height(true)
            .vexpand(true)
            .hexpand(true)
            .build();

        let flow_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        flow_box.set_margin_top(14);
        flow_box.set_margin_bottom(16);
        flow_box.set_margin_start(16);
        flow_box.set_margin_end(16);
        flow_box.set_hexpand(true);
        flow_box.set_vexpand(true);

        // Hero Timer Card
        let hero_card = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        hero_card.add_css_class("card");
        hero_card.add_css_class("hero-card");
        hero_card.set_hexpand(true);
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

        let helper_text = gtk4::Label::builder()
            .label("Mellomrom = Start/pause flyt  •  ADHD Flowmodoro")
            .css_classes(["dim-label", "caption"])
            .halign(gtk4::Align::Center)
            .margin_top(8)
            .build();
        hero_card.append(&helper_text);

        flow_box.append(&hero_card);

        // Stats summary row
        let ratio_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        ratio_box.set_halign(gtk4::Align::Center);
        ratio_box.set_margin_bottom(4);

        let ratio_label = gtk4::Label::builder()
            .label("Forhold: 45m : 15m")
            .css_classes(["caption", "dim-label"])
            .build();
        let today_stats_label = gtk4::Label::builder()
            .label("Total focus today: 0 minutes")
            .css_classes(["caption", "accent"])
            .build();
        ratio_box.append(&ratio_label);
        ratio_box.append(&today_stats_label);
        flow_box.append(&ratio_box);

        // Soundscape Control (between Hero and Context Anchor)
        let flow_sound_group = PreferencesGroup::builder()
            .build();

        let flow_sound_row = ActionRow::builder()
            .title("Fokus-lyd")
            .subtitle("Prosedyrisk støy for dyp konsentrasjon")
            .build();

        let sound_icon = gtk4::Image::from_icon_name("audio-x-generic-symbolic");
        flow_sound_row.add_prefix(&sound_icon);

        let sound_labels: Vec<&'static str> = SoundscapeType::ALL.iter().map(|s| s.label_no()).collect();
        let flow_sound_dropdown = gtk4::DropDown::from_strings(&sound_labels);
        let initial_sound_idx = initial_settings.soundscape.to_index();
        flow_sound_dropdown.set_selected(initial_sound_idx);
        flow_sound_dropdown.set_valign(gtk4::Align::Center);
        flow_sound_row.add_suffix(&flow_sound_dropdown);

        // Volume Popover Button
        let initial_vol_pct = (initial_settings.soundscape_volume * 100.0).clamp(0.0, 100.0) as f64;
        let flow_vol_btn = gtk4::Button::builder()
            .icon_name(Self::volume_icon_name(initial_vol_pct))
            .tooltip_text("Lydstyrke (relativt til musikk/system)")
            .valign(gtk4::Align::Center)
            .css_classes(["flat", "circular"])
            .build();

        let flow_vol_popover = gtk4::Popover::builder()
            .autohide(true)
            .position(gtk4::PositionType::Bottom)
            .has_arrow(false)
            .build();

        let popover_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .margin_top(10)
            .margin_bottom(10)
            .margin_start(12)
            .margin_end(12)
            .width_request(260)
            .build();

        let popover_header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .build();

        let vol_title = gtk4::Label::builder()
            .label("Lydstyrke")
            .css_classes(["heading"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        let flow_vol_label = gtk4::Label::builder()
            .label(&format!("{:.0}%", initial_vol_pct))
            .css_classes(["numeric", "caption"])
            .halign(gtk4::Align::End)
            .build();

        popover_header.append(&vol_title);
        popover_header.append(&flow_vol_label);
        popover_box.append(&popover_header);

        let slider_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .build();

        let icon_low = gtk4::Image::from_icon_name("audio-volume-low-symbolic");
        icon_low.set_opacity(0.6);
        let icon_high = gtk4::Image::from_icon_name("audio-volume-high-symbolic");
        icon_high.set_opacity(0.6);

        let flow_vol_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        flow_vol_scale.set_value(initial_vol_pct);
        flow_vol_scale.set_draw_value(false);
        flow_vol_scale.set_width_request(160);
        flow_vol_scale.set_hexpand(true);

        slider_box.append(&icon_low);
        slider_box.append(&flow_vol_scale);
        slider_box.append(&icon_high);
        popover_box.append(&slider_box);

        let hint_label = gtk4::Label::builder()
            .label("Relativt til musikk og systemlyd")
            .css_classes(["caption", "dim-label"])
            .halign(gtk4::Align::Start)
            .build();
        popover_box.append(&hint_label);

        // Presets: Subtil (musikk/tinnitus), Balanse, Full
        let presets_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .homogeneous(true)
            .build();

        let btn_p15 = gtk4::Button::builder()
            .label("15%")
            .tooltip_text("15% Subtil (optimal for tinnitus-maskering bak musikk)")
            .css_classes(["flat", "caption"])
            .build();
        let btn_p35 = gtk4::Button::builder()
            .label("35%")
            .tooltip_text("35% Balansert fokuslyd")
            .css_classes(["flat", "caption"])
            .build();
        let btn_p70 = gtk4::Button::builder()
            .label("70%")
            .tooltip_text("70% Tydelig maskering for dyp konsentrasjon")
            .css_classes(["flat", "caption"])
            .build();

        presets_box.append(&btn_p15);
        presets_box.append(&btn_p35);
        presets_box.append(&btn_p70);
        popover_box.append(&presets_box);

        flow_vol_popover.set_child(Some(&popover_box));
        flow_vol_popover.set_parent(&flow_sound_row);

        // Click to toggle popover under focus-sound box with right edges aligned
        {
            let flow_vol_popover_clone = flow_vol_popover.clone();
            let flow_sound_row_clone = flow_sound_row.clone();
            flow_vol_btn.connect_clicked(move |_| {
                if flow_vol_popover_clone.is_visible() {
                    flow_vol_popover_clone.popdown();
                } else {
                    let row_w = flow_sound_row_clone.width();
                    let row_h = flow_sound_row_clone.height();
                    let (_, nat) = flow_vol_popover_clone.preferred_size();
                    let pop_w = nat.width().max(300);
                    // Shift left to align flush with card edge, and down to add breathing room
                    let offset_y = 8;
                    let rect = gtk4::gdk::Rectangle::new(row_w - pop_w, row_h + offset_y, pop_w, 1);
                    flow_vol_popover_clone.set_pointing_to(Some(&rect));
                    flow_vol_popover_clone.popup();
                }
            });
        }

        let flow_audio_btn = gtk4::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .tooltip_text("Spill av fokus-lyd")
            .valign(gtk4::Align::Center)
            .css_classes(["flat", "circular"])
            .build();
        flow_sound_row.add_suffix(&flow_audio_btn);
        flow_sound_row.add_suffix(&flow_vol_btn);

        flow_sound_group.add(&flow_sound_row);
        flow_box.append(&flow_sound_group);

        // Context Anchor Group
        let anchor_group = PreferencesGroup::builder()
            .title("⚓ Kontekst-anker")
            .description("Husker nøyaktig hvor du slapp i koden eller prosjektet")
            .build();

        let anchor_row = ActionRow::builder()
            .title("Ingen aktiv kontekst ennå")
            .subtitle("Åpne et prosjekt for å forankre flyten")
            .build();
        let anchor_resume_btn = gtk4::Button::builder()
            .label("Gjenoppta 🚀")
            .css_classes(["suggested-action", "pill"])
            .valign(gtk4::Align::Center)
            .build();
        anchor_resume_btn.set_visible(false);
        anchor_row.add_suffix(&anchor_resume_btn);
        anchor_group.add(&anchor_row);
        flow_box.append(&anchor_group);

        // Momentum / XP Group
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

        flow_scroll.set_child(Some(&flow_box));
        let flow_stack_page = view_stack.add_titled(&flow_scroll, Some("flowmodoro"), "Flyt");
        flow_stack_page.set_icon_name(Some("alarm-symbolic"));

        // --- PAGE 2: RAM-Dump ---
        let ram_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .propagate_natural_height(true)
            .vexpand(true)
            .hexpand(true)
            .build();

        let ram_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        ram_box.set_margin_top(16);
        ram_box.set_margin_bottom(24);
        ram_box.set_margin_start(16);
        ram_box.set_margin_end(16);
        ram_box.set_hexpand(true);
        ram_box.set_vexpand(true);

        let ram_group = PreferencesGroup::builder()
            .title("Hjernetømming / RAM-Dump")
            .description("Tøm arbeidsminnet før avbrudd, møter eller dagsavslutning")
            .build();

        let ram_now_entry = EntryRow::builder()
            .title("[NÅ] Hva holder du på med akkurat nå?")
            .build();
        ram_group.add(&ram_now_entry);

        let ram_status_entry = EntryRow::builder()
            .title("[STATUS] Hva fungerer, og hva er blokkert?")
            .build();
        ram_group.add(&ram_status_entry);

        let ram_next_entry = EntryRow::builder()
            .title("[NESTE] Det første lille steget når du kommer tilbake:")
            .build();
        ram_group.add(&ram_next_entry);

        let save_ram_btn = gtk4::Button::builder()
            .label("Lagre RAM-Dump 💾")
            .css_classes(["pill", "suggested-action"])
            .halign(gtk4::Align::End)
            .margin_top(12)
            .build();
        ram_group.add(&save_ram_btn);

        ram_box.append(&ram_group);

        let last_dump_group = PreferencesGroup::builder()
            .title("Siste lagrede tilstand")
            .build();
        let last_dump_row = ActionRow::builder()
            .title("Ingen dump lagret ennå")
            .subtitle("Fyll ut feltene ovenfor og trykk Lagre for å sikre arbeidsminnet")
            .build();
        last_dump_group.add(&last_dump_row);
        ram_box.append(&last_dump_group);

        ram_scroll.set_child(Some(&ram_box));
        let ram_stack_page = view_stack.add_titled(&ram_scroll, Some("ramdump"), "RAM-dump");
        ram_stack_page.set_icon_name(Some("document-save-symbolic"));

        // --- PAGE 3: Side-quests / Impulser ---
        let imp_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .propagate_natural_height(true)
            .vexpand(true)
            .hexpand(true)
            .build();

        let imp_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        imp_box.set_margin_top(16);
        imp_box.set_margin_bottom(24);
        imp_box.set_margin_start(16);
        imp_box.set_margin_end(16);
        imp_box.set_hexpand(true);
        imp_box.set_vexpand(true);

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

        imp_scroll.set_child(Some(&imp_box));
        let imp_stack_page = view_stack.add_titled(&imp_scroll, Some("impulses"), "Impulser");
        imp_stack_page.set_icon_name(Some("checkbox-checked-symbolic"));

        // --- PAGE 4: Settings / Oppsett ---
        let settings_page = PreferencesPage::new();
        settings_page.set_vexpand(true);
        settings_page.set_hexpand(true);

        // Group: Language & Locale
        let initial_lang = initial_settings.language.resolve();
        let set_general_group = PreferencesGroup::builder()
            .title(match initial_lang { Language::Norwegian => "Språk og utseende", _ => "Language and Appearance" })
            .description(match initial_lang { Language::Norwegian => "Velg språk for Tether (auto-oppdages fra systemet)", _ => "Choose interface language (auto-detects from system)" })
            .build();

        let lang_items = [
            "Auto (Følg system / System default)",
            "Norsk (Bokmål)",
            "English",
        ];
        let lang_models = gtk4::StringList::new(&lang_items);
        let selected_lang_idx = match initial_settings.language {
            LanguageSetting::Auto => 0,
            LanguageSetting::Norwegian => 1,
            LanguageSetting::English => 2,
        };
        let lang_combo = ComboRow::builder()
            .title(match initial_lang { Language::Norwegian => "Språk / Language", _ => "Language" })
            .model(&lang_models)
            .selected(selected_lang_idx)
            .build();
        set_general_group.add(&lang_combo);
        settings_page.add(&set_general_group);

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

        let sound_labels: Vec<&'static str> = SoundscapeType::ALL.iter().map(|s| s.label_no()).collect();
        let sound_models = gtk4::StringList::new(&sound_labels);
        let selected_sound_idx = initial_settings.soundscape.to_index();
        let sound_combo = ComboRow::builder()
            .title("Lydtype")
            .model(&sound_models)
            .selected(selected_sound_idx)
            .build();
        set_audio_group.add(&sound_combo);

        let set_vol_row = ActionRow::builder()
            .title("Lydstyrke")
            .subtitle("Forholdstall relativt til musikk og systemlyd")
            .build();
        let set_vol_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        set_vol_scale.set_value(initial_vol_pct);
        set_vol_scale.set_draw_value(true);
        set_vol_scale.set_value_pos(gtk4::PositionType::Right);
        set_vol_scale.set_width_request(160);
        set_vol_scale.set_valign(gtk4::Align::Center);
        set_vol_row.add_suffix(&set_vol_scale);
        set_audio_group.add(&set_vol_row);

        let auto_sound_switch = SwitchRow::builder()
            .title("Start lyden automatisk ved fokus")
            .subtitle("Spiller støy under fokus og mikro-start, pauser i pausen")
            .active(initial_settings.soundscape_auto_play)
            .build();
        set_audio_group.add(&auto_sound_switch);
        settings_page.add(&set_audio_group);

        // Group: Impulse Storage
        let set_storage_group = PreferencesGroup::builder()
            .title("Impuls-lagring")
            .description("Hvor side-quests og fangede impulser lagres på disken")
            .build();
        let set_file_row = ActionRow::builder()
            .title("Filsti for impulser")
            .subtitle(&initial_settings.side_quests_file)
            .build();
        let set_pick_btn = gtk4::Button::builder()
            .label("Bla gjennom...")
            .icon_name("document-open-symbolic")
            .valign(gtk4::Align::Center)
            .build();
        set_file_row.add_suffix(&set_pick_btn);
        set_storage_group.add(&set_file_row);

        let set_format_models = gtk4::StringList::new(&[
            "Markdown Avkrysningsboks (- [ ])",
            "Markdown Punktliste (-)",
            "Todo.txt format",
            "Strukturert JSON (.json)",
        ]);
        let set_selected_format_idx = match initial_settings.side_quests_format {
            SideQuestFormat::MarkdownCheckbox => 0,
            SideQuestFormat::MarkdownBullet => 1,
            SideQuestFormat::TodoTxt => 2,
            SideQuestFormat::Json => 3,
        };
        let set_format_combo = ComboRow::builder()
            .title("Filformat")
            .model(&set_format_models)
            .selected(set_selected_format_idx)
            .build();
        set_storage_group.add(&set_format_combo);

        let set_cleanup_models = gtk4::StringList::new(&[
            "1 time",
            "1 dag",
            "1 uke",
            "Behold (aldri slette)",
        ]);
        let selected_cleanup_idx = match initial_settings.impulse_cleanup {
            crate::storage::CompletedCleanup::OneHour => 0,
            crate::storage::CompletedCleanup::OneDay => 1,
            crate::storage::CompletedCleanup::OneWeek => 2,
            crate::storage::CompletedCleanup::Keep => 3,
        };
        let set_cleanup_combo = ComboRow::builder()
            .title("Fjerning av fullførte impulser")
            .subtitle("Sletter automatisk avkryssede impulser etter valgt tid")
            .model(&set_cleanup_models)
            .selected(selected_cleanup_idx)
            .build();
        set_storage_group.add(&set_cleanup_combo);
        settings_page.add(&set_storage_group);

        // Group: Editor / Utviklingsmiljø (IDE)
        let set_editor_group = PreferencesGroup::builder()
            .title("Editor og Kodekontekst")
            .description("Sømløs integrasjon med din favoritteditor eller IDE")
            .build();

        let available_editors = EditorType::available_choices();
        let editor_names: Vec<&'static str> = available_editors.iter().map(|e| e.display_name()).collect();
        let editor_models = gtk4::StringList::new(&editor_names);
        let selected_editor_idx = available_editors
            .iter()
            .position(|&e| e == initial_settings.editor_type)
            .unwrap_or(0) as u32;

        let editor_combo = ComboRow::builder()
            .title("Foretrukket editor / IDE")
            .model(&editor_models)
            .selected(selected_editor_idx)
            .build();
        set_editor_group.add(&editor_combo);

        let nvim_mgr_models = gtk4::StringList::new(&[
            "lazy.nvim (~/.config/nvim/lua/plugins/)",
            "Innebygd packpath (Krever null oppsett)",
            "vim-plug (init.vim / init.lua)",
            "Manuell (Vis konfigurasjon)",
        ]);
        let nvim_mgr_idx = match initial_settings.nvim_plugin_manager {
            crate::editor::NvimPluginManager::Lazy => 0,
            crate::editor::NvimPluginManager::Packpath => 1,
            crate::editor::NvimPluginManager::VimPlug => 2,
            crate::editor::NvimPluginManager::Manual => 3,
        };
        let nvim_mgr_combo = ComboRow::builder()
            .title("Pakkemanager for Neovim")
            .model(&nvim_mgr_models)
            .selected(nvim_mgr_idx)
            .build();
        set_editor_group.add(&nvim_mgr_combo);

        let (is_ext_installed, ext_sub) = crate::editor::check_nvim_plugin_status(initial_settings.nvim_plugin_manager);
        let ext_status_row = ActionRow::builder()
            .title("Tether editor-utvidelse")
            .subtitle(&ext_sub)
            .build();
        let install_ext_btn = gtk4::Button::builder()
            .label(if is_ext_installed { "Reinstaller ⚡" } else { "Installer utvidelse ⚡" })
            .valign(gtk4::Align::Center)
            .build();
        if !is_ext_installed {
            install_ext_btn.add_css_class("suggested-action");
        }
        ext_status_row.add_suffix(&install_ext_btn);
        set_editor_group.add(&ext_status_row);

        let minimize_models = gtk4::StringList::new(&[
            "Minimer til system-tray (skjul vindu)",
            "Minimer til Mini-HUD (lite hjørnevindu)",
            "Ikke minimer (behold fullt vindu)",
        ]);
        let minimize_idx = match initial_settings.editor_minimize_action {
            Some(storage::EditorMinimizeAction::Tray) => 0,
            Some(storage::EditorMinimizeAction::MiniHud) => 1,
            Some(storage::EditorMinimizeAction::KeepOpen) => 2,
            None => 0,
        };
        let editor_minimize_combo = ComboRow::builder()
            .title("Minimering (standard for andre editorer)")
            .subtitle("Standard handling når en ukjent eller ukonfigurert editor fokuseres")
            .model(&minimize_models)
            .selected(minimize_idx)
            .build();
        set_editor_group.add(&editor_minimize_combo);

        // Separate minimerings-innstillinger per oppdagede editor
        let detected = crate::editor::EditorType::detected_editors();
        for ed in &detected {
            let ed_models = gtk4::StringList::new(&[
                "Minimer til system-tray (skjul vindu)",
                "Minimer til Mini-HUD (lite hjørnevindu)",
                "Ikke minimer (behold fullt vindu)",
            ]);
            let act = initial_settings.get_minimize_action_for_editor(*ed);
            let idx = match act {
                storage::EditorMinimizeAction::Tray => 0,
                storage::EditorMinimizeAction::MiniHud => 1,
                storage::EditorMinimizeAction::KeepOpen => 2,
            };
            let row = ComboRow::builder()
                .title(&format!("Minimering for {}", ed.display_name()))
                .subtitle(&format!("Hva som skjer med Tether når du fokuserer {}", ed.display_name()))
                .model(&ed_models)
                .selected(idx)
                .build();
            let state_clone = state.clone();
            let ed_val = *ed;
            row.connect_selected_notify(move |combo| {
                let act = match combo.selected() {
                    0 => storage::EditorMinimizeAction::Tray,
                    1 => storage::EditorMinimizeAction::MiniHud,
                    _ => storage::EditorMinimizeAction::KeepOpen,
                };
                let mut st = state_clone.borrow_mut();
                st.settings.set_minimize_action_for_editor(ed_val, act);
                let _ = storage::save_settings(&st.settings);
            });
            set_editor_group.add(&row);
        }

        let nvim_cmd_entry = EntryRow::builder()
            .title("Terminalkommando (for terminal-editorer som Neovim/Helix)")
            .text(&initial_settings.terminal_command)
            .build();
        set_editor_group.add(&nvim_cmd_entry);

        let auto_launch_switch = SwitchRow::builder()
            .title("Auto-start Tether fra editoren")
            .active(initial_settings.auto_launch_desktop)
            .build();
        set_editor_group.add(&auto_launch_switch);
        settings_page.add(&set_editor_group);

        // Group: Window & Mini-HUD Position
        let set_pos_group = PreferencesGroup::builder()
            .title("Vindu og Mini-HUD")
            .description("Posisjon og oppførsel for det kompakte Mini-HUD-vinduet")
            .build();

        let placement_models = gtk4::StringList::new(&[
            "Fritt flytende (Husker hvor du drar den)",
            "Øverst til høyre",
            "Nederst til høyre",
            "Øverst til venstre",
            "Nederst til venstre",
            "Øverst sentrert",
            "Nederst sentrert",
        ]);
        let placement_idx = match initial_settings.mini_hud_placement {
            storage::MiniHudPlacement::Floating => 0,
            storage::MiniHudPlacement::TopRight => 1,
            storage::MiniHudPlacement::BottomRight => 2,
            storage::MiniHudPlacement::TopLeft => 3,
            storage::MiniHudPlacement::BottomLeft => 4,
            storage::MiniHudPlacement::TopCenter => 5,
            storage::MiniHudPlacement::BottomCenter => 6,
        };
        let placement_combo = ComboRow::builder()
            .title("Plassering av Mini-HUD")
            .model(&placement_models)
            .selected(placement_idx)
            .build();
        set_pos_group.add(&placement_combo);

        let initial_pos_sub = match initial_settings.custom_pos_mini {
            Some((x, y)) => format!("Lagret flytende posisjon: X: {}, Y: {}", x, y),
            None => "Standard posisjon (sentrert / følger skjerm)".to_string(),
        };
        let pos_row = ActionRow::builder()
            .title("Lagret flytende posisjon")
            .subtitle(&initial_pos_sub)
            .build();
        let reset_pos_btn = gtk4::Button::builder()
            .label("Nullstill posisjon")
            .valign(gtk4::Align::Center)
            .build();
        pos_row.add_suffix(&reset_pos_btn);
        set_pos_group.add(&pos_row);
        settings_page.add(&set_pos_group);

        let set_stack_page = view_stack.add_titled(&settings_page, Some("settings"), "Oppsett");
        set_stack_page.set_icon_name(Some("preferences-system-symbolic"));

        // Main layout assembly (clean, no view_switcher_bar at bottom)
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.set_vexpand(true);
        main_box.set_hexpand(true);
        main_box.set_size_request(FULL_WINDOW_MIN_WIDTH, FULL_WINDOW_MIN_HEIGHT);
        main_box.append(&header_bar);
        main_box.append(&view_stack);

        // --- MINI-HUD Compact Bar Layout ---
        let mini_hud_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        mini_hud_box.add_css_class("mini-hud-bar");
        mini_hud_box.set_size_request(320, 44);
        mini_hud_box.set_halign(gtk4::Align::Fill);
        mini_hud_box.set_valign(gtk4::Align::Fill);
        mini_hud_box.set_hexpand(true);
        mini_hud_box.set_vexpand(true);
        mini_hud_box.set_margin_top(2);
        mini_hud_box.set_margin_bottom(2);
        mini_hud_box.set_margin_start(10);
        mini_hud_box.set_margin_end(10);

        let mini_anchor = gtk4::Label::builder()
            .label("⚓")
            .tooltip_text("Tether Mini-HUD")
            .valign(gtk4::Align::Center)
            .build();
        mini_hud_box.append(&mini_anchor);

        let mini_badge = gtk4::Label::builder()
            .label("KLAR")
            .valign(gtk4::Align::Center)
            .css_classes(["badge-idle"])
            .build();
        mini_hud_box.append(&mini_badge);

        let mini_timer = gtk4::Label::builder()
            .label("00:00")
            .valign(gtk4::Align::Center)
            .css_classes(["mini-hud-timer"])
            .build();
        mini_hud_box.append(&mini_timer);

        let mini_toggle_btn = gtk4::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .tooltip_text("Start / Pause (Flowmodoro) [Mellomrom]")
            .valign(gtk4::Align::Center)
            .css_classes(["flat"])
            .build();
        mini_hud_box.append(&mini_toggle_btn);

        let mini_micro_btn = gtk4::Button::builder()
            .icon_name("weather-storm-symbolic")
            .tooltip_text("⚡ 5m Mikro-start")
            .valign(gtk4::Align::Center)
            .css_classes(["flat"])
            .build();
        mini_hud_box.append(&mini_micro_btn);

        let mini_audio_btn = gtk4::Button::builder()
            .icon_name("audio-x-generic-symbolic")
            .tooltip_text("Start / Stopp fokus-lyd")
            .valign(gtk4::Align::Center)
            .css_classes(["flat", "mini-audio-btn-stopped"])
            .build();
        mini_hud_box.append(&mini_audio_btn);

        let mini_expand_btn = gtk4::Button::builder()
            .icon_name("view-fullscreen-symbolic")
            .tooltip_text("Utvid til fullt Tether-vindu [Esc]")
            .valign(gtk4::Align::Center)
            .css_classes(["flat"])
            .build();
        mini_hud_box.append(&mini_expand_btn);

        window.set_child(Some(&main_box));

        let ui = Rc::new(Self {
            state: state.clone(),
            window: window.clone(),
            main_box,
            mini_hud_box,
            css_provider,
            status_badge,
            audio_btn,
            editor_btn,
            pill_mode_btn,
            timer_label,
            hero_badge,
            sub_info_label,
            main_action_btn,
            micro_btn,
            reset_btn,
            helper_text,
            ratio_label,
            today_stats_label,
            mini_timer,
            mini_badge,
            mini_toggle_btn,
            mini_micro_btn,
            mini_audio_btn,
            mini_expand_btn,
            flow_sound_dropdown: flow_sound_dropdown.clone(),
            flow_audio_btn: flow_audio_btn.clone(),
            flow_vol_btn: flow_vol_btn.clone(),
            flow_vol_scale: flow_vol_scale.clone(),
            flow_vol_label: flow_vol_label.clone(),
            flow_sound_row: flow_sound_row.clone(),
            sound_combo: sound_combo.clone(),
            set_vol_scale: set_vol_scale.clone(),
            anchor_group,
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
            set_file_row,
            set_format_combo: set_format_combo.clone(),
            set_cleanup_combo: set_cleanup_combo.clone(),
            editor_combo: editor_combo.clone(),
            nvim_mgr_combo: nvim_mgr_combo.clone(),
            ext_status_row: ext_status_row.clone(),
            install_ext_btn: install_ext_btn.clone(),
            editor_minimize_combo: editor_minimize_combo.clone(),
            pos_row: pos_row.clone(),
            tray: crate::tray::TetherTray::new().ok().map(Rc::new),
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

        // Event: Audio header button
        {
            let ui_clone = ui.clone();
            ui.audio_btn.connect_clicked(move |_| {
                ui_clone.toggle_audio();
            });
        }

        // Event: Pill mode button in headerbar
        {
            let ui_clone = ui.clone();
            ui.pill_mode_btn.connect_clicked(move |_| {
                ui_clone.set_mini_hud_mode(true);
            });
        }

        // Event: Expand button in Mini-HUD
        {
            let ui_clone = ui.clone();
            ui.mini_expand_btn.connect_clicked(move |_| {
                ui_clone.set_mini_hud_mode(false);
            });
        }

        // Event: Mini toggle flow
        {
            let ui_clone = ui.clone();
            ui.mini_toggle_btn.connect_clicked(move |_| {
                ui_clone.toggle_flow();
            });
        }

        // Event: Mini micro-start
        {
            let ui_clone = ui.clone();
            ui.mini_micro_btn.connect_clicked(move |_| {
                ui_clone.start_micro();
            });
        }

        // Event: Mini audio toggle
        {
            let ui_clone = ui.clone();
            ui.mini_audio_btn.connect_clicked(move |_| {
                ui_clone.toggle_audio();
            });
        }

        // Event: Mini timer click to toggle
        {
            let ui_clone = ui.clone();
            let click = gtk4::GestureClick::new();
            click.connect_released(move |_, _, _, _| {
                ui_clone.toggle_flow();
            });
            ui.mini_timer.add_controller(click);
        }

        // Event: File Picker in Settings
        {
            let ui_clone = ui.clone();
            set_pick_btn.connect_clicked(move |_| {
                ui_clone.select_impulse_file();
            });
        }

        // Event: Format Combo in Settings
        {
            let state_clone = state.clone();
            let ui_clone = ui.clone();
            set_format_combo.connect_selected_notify(move |combo| {
                let sel = combo.selected();
                let fmt = match sel {
                    0 => SideQuestFormat::MarkdownCheckbox,
                    1 => SideQuestFormat::MarkdownBullet,
                    2 => SideQuestFormat::TodoTxt,
                    3 => SideQuestFormat::Json,
                    _ => SideQuestFormat::MarkdownCheckbox,
                };
                {
                    let mut st = state_clone.borrow_mut();
                    if st.settings.side_quests_format != fmt {
                        st.settings.side_quests_format = fmt;
                        let _ = storage::save_settings(&st.settings);
                    }
                }
                ui_clone.refresh_impulses_list();
            });
        }

        // Event: Cleanup Combo in Settings
        {
            let state_clone = state.clone();
            let ui_clone = ui.clone();
            set_cleanup_combo.connect_selected_notify(move |combo| {
                let sel = combo.selected();
                let cleanup = match sel {
                    0 => crate::storage::CompletedCleanup::OneHour,
                    1 => crate::storage::CompletedCleanup::OneDay,
                    2 => crate::storage::CompletedCleanup::OneWeek,
                    3 => crate::storage::CompletedCleanup::Keep,
                    _ => crate::storage::CompletedCleanup::OneDay,
                };
                {
                    let mut st = state_clone.borrow_mut();
                    if st.settings.impulse_cleanup != cleanup {
                        st.settings.impulse_cleanup = cleanup;
                        let _ = storage::save_settings(&st.settings);
                    }
                }
                ui_clone.refresh_impulses_list();
            });
        }

        // Event: Editor header button
        {
            let state_clone = state.clone();
            ui.editor_btn.connect_clicked(move |_| {
                let st = state_clone.borrow();
                let editor = st.settings.editor_type;
                editor.open_or_focus(&st.settings.terminal_command);
            });
        }

        // Event: Resume in editor button
        {
            let state_clone = state.clone();
            ui.anchor_resume_btn.connect_clicked(move |_| {
                let st = state_clone.borrow();
                if let Some(anchor) = &st.anchor {
                    st.settings.editor_type.resume(anchor, &st.settings.terminal_command);
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

        // Event: Impulse entry apply / Enter
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

        // Event: Language selection
        {
            let state_clone = state.clone();
            lang_combo.connect_selected_notify(move |combo| {
                let mut st = state_clone.borrow_mut();
                st.settings.language = match combo.selected() {
                    1 => LanguageSetting::Norwegian,
                    2 => LanguageSetting::English,
                    _ => LanguageSetting::Auto,
                };
                let _ = storage::save_settings(&st.settings);
            });
        }

        // Settings events
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
        // Event: Flow Sound Dropdown
        {
            let state_clone = state.clone();
            let sound_combo_clone = sound_combo.clone();
            flow_sound_dropdown.connect_selected_notify(move |dd| {
                let sel = dd.selected();
                let new_type = SoundscapeType::from_index(sel);
                {
                    let mut st = state_clone.borrow_mut();
                    if st.settings.soundscape != new_type {
                        st.settings.soundscape = new_type;
                        st.audio.set_sound(new_type);
                        let _ = storage::save_settings(&st.settings);
                    }
                }
                if sound_combo_clone.selected() != sel {
                    sound_combo_clone.set_selected(sel);
                }
            });
        }

        // Event: Flow Audio Play/Pause Button
        {
            let ui_clone = ui.clone();
            flow_audio_btn.connect_clicked(move |_| {
                ui_clone.toggle_audio();
            });
        }

        // Event: Presets for Volume Popover
        {
            let flow_vol_scale_clone = flow_vol_scale.clone();
            btn_p15.connect_clicked(move |_| {
                flow_vol_scale_clone.set_value(15.0);
            });
        }
        {
            let flow_vol_scale_clone = flow_vol_scale.clone();
            btn_p35.connect_clicked(move |_| {
                flow_vol_scale_clone.set_value(35.0);
            });
        }
        {
            let flow_vol_scale_clone = flow_vol_scale.clone();
            btn_p70.connect_clicked(move |_| {
                flow_vol_scale_clone.set_value(70.0);
            });
        }

        // Event: Flow Volume Scale
        {
            let state_clone = state.clone();
            let set_vol_scale_clone = set_vol_scale.clone();
            let flow_vol_btn_clone = flow_vol_btn.clone();
            let flow_vol_label_clone = flow_vol_label.clone();
            flow_vol_scale.connect_value_changed(move |scale| {
                let val = scale.value();
                let vol_frac = (val / 100.0).clamp(0.0, 1.0) as f32;
                flow_vol_label_clone.set_label(&format!("{:.0}%", val));
                flow_vol_btn_clone.set_icon_name(Self::volume_icon_name(val));
                {
                    let mut st = state_clone.borrow_mut();
                    if (st.settings.soundscape_volume - vol_frac).abs() > 0.005 {
                        st.settings.soundscape_volume = vol_frac;
                        st.audio.set_volume(vol_frac);
                        let _ = storage::save_settings(&st.settings);
                    }
                }
                if (set_vol_scale_clone.value() - val).abs() > 0.5 {
                    set_vol_scale_clone.set_value(val);
                }
            });
        }

        // Event: Settings Volume Scale
        {
            let state_clone = state.clone();
            let flow_vol_scale_clone = flow_vol_scale.clone();
            let flow_vol_btn_clone = flow_vol_btn.clone();
            let flow_vol_label_clone = flow_vol_label.clone();
            set_vol_scale.connect_value_changed(move |scale| {
                let val = scale.value();
                let vol_frac = (val / 100.0).clamp(0.0, 1.0) as f32;
                flow_vol_label_clone.set_label(&format!("{:.0}%", val));
                flow_vol_btn_clone.set_icon_name(Self::volume_icon_name(val));
                {
                    let mut st = state_clone.borrow_mut();
                    if (st.settings.soundscape_volume - vol_frac).abs() > 0.005 {
                        st.settings.soundscape_volume = vol_frac;
                        st.audio.set_volume(vol_frac);
                        let _ = storage::save_settings(&st.settings);
                    }
                }
                if (flow_vol_scale_clone.value() - val).abs() > 0.5 {
                    flow_vol_scale_clone.set_value(val);
                }
            });
        }

        // Event: Settings Sound Combo (Keep in sync with flow sound dropdown)
        {
            let state_clone = state.clone();
            let flow_sound_dropdown_clone = flow_sound_dropdown.clone();
            sound_combo.connect_selected_notify(move |combo| {
                let sel = combo.selected();
                let new_type = SoundscapeType::from_index(sel);
                {
                    let mut st = state_clone.borrow_mut();
                    if st.settings.soundscape != new_type {
                        st.settings.soundscape = new_type;
                        st.audio.set_sound(new_type);
                        let _ = storage::save_settings(&st.settings);
                    }
                }
                if flow_sound_dropdown_clone.selected() != sel {
                    flow_sound_dropdown_clone.set_selected(sel);
                }
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
            let available_editors_clone = available_editors.clone();
            editor_combo.connect_selected_notify(move |combo| {
                let idx = combo.selected() as usize;
                if idx < available_editors_clone.len() {
                    let mut st = state_clone.borrow_mut();
                    st.settings.editor_type = available_editors_clone[idx];
                    let _ = storage::save_settings(&st.settings);
                }
            });
        }
        {
            let state_clone = state.clone();
            let nvim_mgr_combo_clone = nvim_mgr_combo.clone();
            let ext_status_row_clone = ext_status_row.clone();
            let install_ext_btn_clone = install_ext_btn.clone();
            let available_editors = EditorType::available_choices();
            editor_combo.connect_selected_notify(move |combo| {
                let idx = combo.selected() as usize;
                if let Some(&editor) = available_editors.get(idx) {
                    let mut st = state_clone.borrow_mut();
                    st.settings.editor_type = editor;
                    let _ = storage::save_settings(&st.settings);

                    let resolved = editor.resolve();
                    if resolved == EditorType::Neovim || resolved == EditorType::Neovide {
                        nvim_mgr_combo_clone.set_visible(true);
                        let (installed, sub) = crate::editor::check_nvim_plugin_status(st.settings.nvim_plugin_manager);
                        ext_status_row_clone.set_subtitle(&sub);
                        if installed {
                            install_ext_btn_clone.set_label("Reinstaller ⚡");
                            install_ext_btn_clone.remove_css_class("suggested-action");
                        } else {
                            install_ext_btn_clone.set_label("Installer utvidelse ⚡");
                            install_ext_btn_clone.add_css_class("suggested-action");
                        }
                    } else if resolved == EditorType::None {
                        nvim_mgr_combo_clone.set_visible(false);
                        ext_status_row_clone.set_subtitle("Ingen editor valgt");
                    } else {
                        nvim_mgr_combo_clone.set_visible(false);
                        let (installed, sub) = crate::editor::check_vscode_extension_status(resolved);
                        ext_status_row_clone.set_subtitle(&sub);
                        if installed {
                            install_ext_btn_clone.set_label("Reinstaller ⚡");
                            install_ext_btn_clone.remove_css_class("suggested-action");
                        } else {
                            install_ext_btn_clone.set_label("Installer utvidelse ⚡");
                            install_ext_btn_clone.add_css_class("suggested-action");
                        }
                    }
                }
            });
        }
        {
            let state_clone = state.clone();
            let ext_status_row_clone = ext_status_row.clone();
            let install_ext_btn_clone = install_ext_btn.clone();
            nvim_mgr_combo.connect_selected_notify(move |combo| {
                let mgr = match combo.selected() {
                    0 => crate::editor::NvimPluginManager::Lazy,
                    1 => crate::editor::NvimPluginManager::Packpath,
                    2 => crate::editor::NvimPluginManager::VimPlug,
                    _ => crate::editor::NvimPluginManager::Manual,
                };
                {
                    let mut st = state_clone.borrow_mut();
                    st.settings.nvim_plugin_manager = mgr;
                    let _ = storage::save_settings(&st.settings);
                }
                let (installed, sub) = crate::editor::check_nvim_plugin_status(mgr);
                ext_status_row_clone.set_subtitle(&sub);
                if installed {
                    install_ext_btn_clone.set_label("Reinstaller ⚡");
                    install_ext_btn_clone.remove_css_class("suggested-action");
                } else {
                    install_ext_btn_clone.set_label("Installer utvidelse ⚡");
                    install_ext_btn_clone.add_css_class("suggested-action");
                }
            });
        }
        {
            let state_clone = state.clone();
            let ext_status_row_clone = ext_status_row.clone();
            let install_ext_btn_clone = install_ext_btn.clone();
            let window_clone = window.clone();
            install_ext_btn.connect_clicked(move |_| {
                let st = state_clone.borrow();
                let editor = st.settings.editor_type.resolve();
                let mgr = st.settings.nvim_plugin_manager;
                drop(st);

                let res = match editor {
                    EditorType::Neovim | EditorType::Neovide => {
                        crate::editor::install_nvim_plugin(mgr)
                    }
                    _ => {
                        crate::editor::install_vscode_extension(editor)
                    }
                };

                match res {
                    Ok(msg) => {
                        let alert = libadwaita::AlertDialog::builder()
                            .heading("Utvidelse installert!")
                            .body(&msg)
                            .build();
                        alert.add_response("ok", "OK");
                        alert.choose(Some(&window_clone), None::<&gtk4::gio::Cancellable>, |_| {});

                        let (installed, sub) = crate::editor::check_nvim_plugin_status(mgr);
                        ext_status_row_clone.set_subtitle(&sub);
                        if installed {
                            install_ext_btn_clone.set_label("Reinstaller ⚡");
                            install_ext_btn_clone.remove_css_class("suggested-action");
                        }
                    }
                    Err(err) => {
                        let alert = libadwaita::AlertDialog::builder()
                            .heading("Feil ved installasjon")
                            .body(&err)
                            .build();
                        alert.add_response("ok", "OK");
                        alert.choose(Some(&window_clone), None::<&gtk4::gio::Cancellable>, |_| {});
                    }
                }
            });
        }
        {
            let state_clone = state.clone();
            editor_minimize_combo.connect_selected_notify(move |combo| {
                let act = match combo.selected() {
                    0 => storage::EditorMinimizeAction::Tray,
                    1 => storage::EditorMinimizeAction::MiniHud,
                    _ => storage::EditorMinimizeAction::KeepOpen,
                };
                let mut st = state_clone.borrow_mut();
                st.settings.editor_minimize_action = Some(act);
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
        {
            let state_clone = state.clone();
            auto_launch_switch.connect_active_notify(move |sw| {
                let mut st = state_clone.borrow_mut();
                st.settings.auto_launch_desktop = sw.is_active();
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            let pos_row_clone = pos_row.clone();
            reset_pos_btn.connect_clicked(move |_| {
                let mut st = state_clone.borrow_mut();
                st.settings.custom_pos_mini = None;
                let _ = storage::save_settings(&st.settings);
                pos_row_clone.set_subtitle("Standard posisjon (sentrert / følger skjerm)");
            });
        }
        {
            let state_clone = state.clone();
            placement_combo.connect_selected_notify(move |combo| {
                let place = match combo.selected() {
                    1 => storage::MiniHudPlacement::TopRight,
                    2 => storage::MiniHudPlacement::BottomRight,
                    3 => storage::MiniHudPlacement::TopLeft,
                    4 => storage::MiniHudPlacement::BottomLeft,
                    5 => storage::MiniHudPlacement::TopCenter,
                    6 => storage::MiniHudPlacement::BottomCenter,
                    _ => storage::MiniHudPlacement::Floating,
                };
                let mut st = state_clone.borrow_mut();
                st.settings.mini_hud_placement = place;
                let _ = storage::save_settings(&st.settings);
                if st.is_mini_hud {
                    if let Some((x, y)) = place.calculate_pos(&st.settings) {
                        storage::move_window_to_pos(x, y);
                    }
                }
            });
        }

        // Keyboard shortcuts (Escape toggles between Mini-HUD and Full window)
        let key_controller = gtk4::EventControllerKey::new();
        {
            let ui_clone = ui.clone();
            key_controller.connect_key_pressed(move |_, keyval, _, _| {
                if keyval == gtk4::gdk::Key::Escape {
                    let is_mini = ui_clone.state.borrow().is_mini_hud;
                    ui_clone.set_mini_hud_mode(!is_mini);
                    gtk4::glib::Propagation::Stop
                } else {
                    gtk4::glib::Propagation::Proceed
                }
            });
        }
        ui.window.add_controller(key_controller);

        // Populate initial impulses
        ui.refresh_impulses_list();

        // 500ms Timer loop for UI updates, IPC sync, and live theme reloading
        {
            let ui_clone = ui.clone();
            glib::timeout_add_local(Duration::from_millis(500), move || {
                ui_clone.update_ui_tick();
                glib::ControlFlow::Continue
            });
        }

        // Cross-platform System Tray Polling loop (Linux, Windows, macOS)
        if let Some(tray_ref) = &ui.tray {
            let tray_ref = tray_ref.clone();
            let ui_clone = ui.clone();
            glib::timeout_add_local(Duration::from_millis(150), move || {
                while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                    match event {
                        tray_icon::TrayIconEvent::Click { button: tray_icon::MouseButton::Left, .. } => {
                            let is_vis = ui_clone.window.is_visible();
                            if is_vis {
                                ui_clone.window.set_visible(false);
                            } else {
                                ui_clone.window.set_visible(true);
                                ui_clone.window.present();
                            }
                        }
                        _ => {}
                    }
                }

                while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                    let id = event.id;
                    if id == tray_ref.item_toggle_win.id() {
                        let is_vis = ui_clone.window.is_visible();
                        if is_vis {
                            ui_clone.window.set_visible(false);
                        } else {
                            ui_clone.window.set_visible(true);
                            ui_clone.window.present();
                        }
                    } else if id == tray_ref.item_mini_hud.id() {
                        ui_clone.window.set_visible(true);
                        ui_clone.set_mini_hud_mode(true);
                        ui_clone.window.present();
                    } else if id == tray_ref.item_toggle_flow.id() {
                        ui_clone.toggle_flow();
                    } else if id == tray_ref.item_quit.id() {
                        if let Some(app) = ui_clone.window.application() {
                            app.quit();
                        } else {
                            std::process::exit(0);
                        }
                    }
                }

                glib::ControlFlow::Continue
            });
        }

        ui.window.present();
        ui
    }

    pub fn set_mini_hud_mode(&self, is_mini: bool) {
        if is_mini {
            // Save full window position before shrinking (validated & clamped)
            if let Some((cur_x, cur_y)) = storage::get_tether_window_pos() {
                let locked = self.state.borrow().settings.locked_monitor.clone();
                let clamped = storage::clamp_full_window_pos(cur_x, cur_y, FULL_WINDOW_WIDTH, FULL_WINDOW_HEIGHT, &locked);
                let mut st = self.state.borrow_mut();
                st.settings.custom_pos_full = Some(clamped);
                let _ = storage::save_settings(&st.settings);
            }
            {
                let mut st = self.state.borrow_mut();
                st.is_mini_hud = true;
                st.suppress_pos_ticks = 4; // suppress recording while compositor resizes
            }

            self.window.set_decorated(false);
            self.window.set_child(Some(&self.mini_hud_box));
            self.window.set_size_request(320, 44);
            self.window.set_default_size(320, 44);
            #[cfg(target_os = "linux")]
            {
                storage::ensure_pinned(true);
                storage::resize_window(320, 44);
                let target_pos = {
                    let st = self.state.borrow();
                    st.settings.mini_hud_placement.calculate_pos(&st.settings)
                };
                if let Some((x, y)) = target_pos {
                    storage::move_window_to_pos(x, y);
                }
            }
        } else {
            // Save mini-HUD position before expanding
            if let Some((cur_x, cur_y)) = storage::get_tether_window_pos() {
                let mut st = self.state.borrow_mut();
                if st.settings.mini_hud_placement == storage::MiniHudPlacement::Floating {
                    st.settings.custom_pos_mini = Some((cur_x, cur_y));
                    let _ = storage::save_settings(&st.settings);
                    self.pos_row.set_subtitle(&format!("Lagret flytende posisjon: X: {}, Y: {}", cur_x, cur_y));
                }
            }
            {
                let mut st = self.state.borrow_mut();
                st.is_mini_hud = false;
                st.suppress_pos_ticks = 4;
            }

            #[cfg(target_os = "linux")]
            {
                storage::ensure_pinned(false);
                let (target_x, target_y) = {
                    let st = self.state.borrow();
                    if let Some((x, y)) = st.settings.custom_pos_full {
                        storage::clamp_full_window_pos(x, y, FULL_WINDOW_WIDTH, FULL_WINDOW_HEIGHT, &st.settings.locked_monitor)
                    } else {
                        storage::get_centered_full_pos(FULL_WINDOW_WIDTH, FULL_WINDOW_HEIGHT, &st.settings.locked_monitor)
                    }
                };
                // Move and resize window BEFORE setting full content so it doesn't jump or expand off-screen
                storage::move_window_to_pos(target_x, target_y);
                storage::resize_window(FULL_WINDOW_WIDTH, FULL_WINDOW_HEIGHT);
            }

            self.window.set_size_request(FULL_WINDOW_MIN_WIDTH, FULL_WINDOW_MIN_HEIGHT);
            self.window.set_decorated(true);
            self.window.set_child(Some(&self.main_box));
            self.window.set_default_size(FULL_WINDOW_WIDTH, FULL_WINDOW_HEIGHT);

            #[cfg(target_os = "linux")]
            {
                let (target_x, target_y) = {
                    let st = self.state.borrow();
                    if let Some((x, y)) = st.settings.custom_pos_full {
                        storage::clamp_full_window_pos(x, y, FULL_WINDOW_WIDTH, FULL_WINDOW_HEIGHT, &st.settings.locked_monitor)
                    } else {
                        storage::get_centered_full_pos(FULL_WINDOW_WIDTH, FULL_WINDOW_HEIGHT, &st.settings.locked_monitor)
                    }
                };
                storage::move_window_to_pos(target_x, target_y);
            }
        }
    }

    pub fn prompt_editor_minimize(&self) {
        {
            let mut st = self.state.borrow_mut();
            if st.asked_editor_minimize_prompt {
                return;
            }
            st.asked_editor_minimize_prompt = true;
        }

        let alert = libadwaita::AlertDialog::builder()
            .heading("Minimere Tether når du koder?")
            .body("Tether oppdaget at du koder i en integrert editor.\n\nVil du at Tether skal minimeres til system-tray (skjult i bakgrunnen) eller krympes til det lille Mini-HUD-vinduet i hjørnet?\n\n(Du kan alltids hente appen tilbake via tray-ikonet eller endre dette i Oppsett).")
            .build();

        alert.add_response("tray", "Minimer til Tray 📥");
        alert.add_response("mini", "Minimer til Mini-HUD 🗕");
        alert.add_response("keep", "Behold åpen 🗖");

        alert.set_response_appearance("tray", libadwaita::ResponseAppearance::Suggested);
        alert.set_default_response(Some("tray"));
        alert.set_close_response("keep");

        let state = self.state.clone();
        let editor_minimize_combo = self.editor_minimize_combo.clone();
        let window = self.window.clone();
        let mini_hud_box = self.mini_hud_box.clone();

        alert.choose(None::<&gtk4::Window>, None::<&gtk4::gio::Cancellable>, move |response| {
            let act = match response.as_str() {
                "tray" => storage::EditorMinimizeAction::Tray,
                "mini" => storage::EditorMinimizeAction::MiniHud,
                _ => storage::EditorMinimizeAction::KeepOpen,
            };
            {
                let mut st = state.borrow_mut();
                st.settings.editor_minimize_action = Some(act);
                let _ = storage::save_settings(&st.settings);
            }
            match act {
                storage::EditorMinimizeAction::Tray => {
                    editor_minimize_combo.set_selected(0);
                    window.set_visible(false);
                }
                storage::EditorMinimizeAction::MiniHud => {
                    editor_minimize_combo.set_selected(1);
                    {
                        let mut st = state.borrow_mut();
                        st.is_mini_hud = true;
                        st.suppress_pos_ticks = 4;
                    }
                    window.set_decorated(false);
                    window.set_child(Some(&mini_hud_box));
                    window.set_size_request(320, 44);
                    window.set_default_size(320, 44);
                    #[cfg(target_os = "linux")]
                    {
                        storage::ensure_pinned(true);
                        storage::resize_window(320, 44);
                        let target_pos = {
                            let st = state.borrow();
                            st.settings.mini_hud_placement.calculate_pos(&st.settings)
                        };
                        if let Some((x, y)) = target_pos {
                            storage::move_window_to_pos(x, y);
                        }
                    }
                }
                storage::EditorMinimizeAction::KeepOpen => {
                    editor_minimize_combo.set_selected(2);
                }
            }
        });
    }

    pub fn select_impulse_file(&self) {
        if let Some(path) = storage::pick_impulse_file() {
            let path_str = path.to_string_lossy().to_string();
            let mut detected_format = None;
            if path_str.ends_with(".json") {
                detected_format = Some(SideQuestFormat::Json);
            } else if path_str.ends_with(".txt") {
                detected_format = Some(SideQuestFormat::TodoTxt);
            } else if path_str.ends_with(".md") {
                detected_format = Some(SideQuestFormat::MarkdownCheckbox);
            }

            {
                let mut st = self.state.borrow_mut();
                st.settings.side_quests_file = path_str.clone();
                if let Some(fmt) = detected_format {
                    st.settings.side_quests_format = fmt;
                }
                let _ = storage::save_settings(&st.settings);
            }
            self.set_file_row.set_subtitle(&path_str);
            if let Some(fmt) = detected_format {
                let idx = match fmt {
                    SideQuestFormat::MarkdownCheckbox => 0,
                    SideQuestFormat::MarkdownBullet => 1,
                    SideQuestFormat::TodoTxt => 2,
                    SideQuestFormat::Json => 3,
                };
                self.set_format_combo.set_selected(idx);
            }
            self.refresh_impulses_list();
        }
    }

    pub fn update_ui_tick(&self) {
        let now = Instant::now();

        // Check for state updates
        {
            let mut st = self.state.borrow_mut();
            let dt = now.duration_since(st.last_update);
            st.last_update = now;
            st.flow.tick(dt);

            let old = st.previous_state;
            let cur = st.flow.state;
            if old != cur {
                st.previous_state = cur;
                st.audio.handle_flow_state_transition(old, cur);

                // Stats update on state transition (unngå duplisering ved gjenopptatt fokus)
                if old == FlowState::Focus && (cur == FlowState::Break || cur == FlowState::Idle) {
                    let total_mins = st.flow.elapsed_focus_secs / 60;
                    let newly_mins = total_mins.saturating_sub(st.last_recorded_focus_mins);
                    if newly_mins > 0 {
                        st.stats.record_focus_minutes(newly_mins);
                        st.last_recorded_focus_mins = total_mins;
                        let _ = storage::save_user_stats(&st.stats);
                    }
                } else if old == FlowState::MicroStart && cur == FlowState::Focus {
                    st.stats.record_micro_start();
                    let _ = storage::save_user_stats(&st.stats);
                }

                // Maksimer/vis hovedvinduet automatisk når man tar pause så man enkelt kan ta RAM-dump eller skrive impulser
                if cur == FlowState::Break {
                    drop(st);
                    self.set_mini_hud_mode(false);
                    self.window.set_visible(true);
                    self.window.present();
                    self.ram_now_entry.grab_focus();
                    return;
                }
            }

            // Check anchor periodically (every 1s)
            if now.duration_since(st.last_anchor_check) >= Duration::from_secs(1) {
                st.anchor = storage::load_context_anchor();
                st.last_anchor_check = now;
            }

            // Check stats periodically (every 2s)
            if now.duration_since(st.last_stats_check) >= Duration::from_secs(2) {
                st.stats = storage::load_user_stats();
                st.last_stats_check = now;
            }

            // Auto-minimize check when editor is focused (separat per oppdaget editor)
            if !st.is_mini_hud && self.window.is_visible() && st.suppress_pos_ticks == 0 {
                if let Some(focused_editor) = crate::editor::get_focused_editor() {
                    let action = st.settings.get_minimize_action_for_editor(focused_editor);
                    match action {
                        storage::EditorMinimizeAction::Tray => {
                            drop(st);
                            self.window.set_visible(false);
                            return;
                        }
                        storage::EditorMinimizeAction::MiniHud => {
                            drop(st);
                            self.set_mini_hud_mode(true);
                            return;
                        }
                        storage::EditorMinimizeAction::KeepOpen => {}
                    }
                }
            }

            // Track window position periodically (every 800ms)
            if now.duration_since(st.last_pos_check) >= Duration::from_millis(800) {
                st.last_pos_check = now;
                if st.suppress_pos_ticks > 0 {
                    st.suppress_pos_ticks -= 1;
                } else {
                    let is_mini = st.is_mini_hud;
                    let locked_mon = st.settings.locked_monitor.clone();
                    drop(st);
                    if let Some((cur_x, cur_y)) = storage::get_tether_window_pos() {
                        let mut st = self.state.borrow_mut();
                        if is_mini {
                            if st.settings.mini_hud_placement == storage::MiniHudPlacement::Floating {
                                let should_save = match st.settings.custom_pos_mini {
                                    Some((old_x, old_y)) => (cur_x - old_x).abs() > 4 || (cur_y - old_y).abs() > 4,
                                    None => true,
                                };
                                if should_save {
                                    st.settings.custom_pos_mini = Some((cur_x, cur_y));
                                    let _ = storage::save_settings(&st.settings);
                                    self.pos_row.set_subtitle(&format!("Lagret flytende posisjon: X: {}, Y: {}", cur_x, cur_y));
                                }
                            }
                        } else {
                            let (clamped_x, clamped_y) = storage::clamp_full_window_pos(cur_x, cur_y, FULL_WINDOW_WIDTH, FULL_WINDOW_HEIGHT, &locked_mon);
                            if cur_y >= 30 && (clamped_x - cur_x).abs() <= 50 && (clamped_y - cur_y).abs() <= 50 {
                                let should_save = match st.settings.custom_pos_full {
                                    Some((old_x, old_y)) => (clamped_x - old_x).abs() > 4 || (clamped_y - old_y).abs() > 4,
                                    None => true,
                                };
                                if should_save {
                                    st.settings.custom_pos_full = Some((clamped_x, clamped_y));
                                    let _ = storage::save_settings(&st.settings);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update Dynamic UI Labels
        let st = self.state.borrow();
        let lang = st.settings.language.resolve();
        let timer_text = st.flow.current_time_display();
        self.timer_label.set_label(&timer_text);
        self.mini_timer.set_label(&timer_text);

        let (tag_text, css_class, sub_info) = match st.flow.state {
            FlowState::MicroStart => {
                let rem = crate::flowmodoro::format_time(st.flow.micro_remaining_secs);
                (
                    Strings::tag_micro(lang),
                    "badge-micro",
                    match lang {
                        Language::Norwegian => format!("⚡ Bare 5 min for å bryte dørstokkmila ({} gjenstår). Null skyldfølelse!", rem),
                        _ => format!("⚡ Just 5 mins to melt friction ({} left). Zero guilt!", rem),
                    },
                )
            }
            FlowState::Focus => {
                let earned = st.flow.calculate_break_time();
                let earned_str = crate::flowmodoro::format_time(earned);
                let sub = if st.flow.banked_break_secs > 0 {
                    let banked_str = crate::flowmodoro::format_time(st.flow.banked_break_secs);
                    match lang {
                        Language::Norwegian => format!("Opptjent pause hittil: {} (inkl. {} spart fra forrige pause) ({}:{} forhold)", earned_str, banked_str, st.flow.focus_ratio, st.flow.break_ratio),
                        _ => format!("Earned break so far: {} (incl. {} saved from last break) ({}:{} ratio)", earned_str, banked_str, st.flow.focus_ratio, st.flow.break_ratio),
                    }
                } else {
                    Strings::sub_focus(lang, &earned_str, st.flow.focus_ratio, st.flow.break_ratio)
                };
                (
                    Strings::tag_focus(lang),
                    "badge-focus",
                    sub,
                )
            }
            FlowState::Break => {
                let rem_str = crate::flowmodoro::format_time(st.flow.remaining_break_secs);
                let sub = match lang {
                    Language::Norwegian => format!("Pause pågår: {} gjenstår. Klikk 'Avslutt pause' for å fortsette fokusøkt og spare resten!", rem_str),
                    _ => format!("Break in progress: {} left. Click 'End break' to resume focus and save the rest!", rem_str),
                };
                (
                    Strings::tag_break(lang),
                    "badge-break",
                    sub,
                )
            }
            FlowState::Idle => (
                Strings::tag_ready(lang),
                "badge-idle",
                Strings::sub_idle(lang).to_string(),
            ),
        };

        self.status_badge.set_label(tag_text);
        self.status_badge.set_css_classes(&[css_class]);
        self.hero_badge.set_label(tag_text);
        self.hero_badge.set_css_classes(&[css_class]);
        self.sub_info_label.set_label(&sub_info);

        // Sync Mini-HUD state
        self.mini_badge.set_label(tag_text);
        self.mini_badge.set_css_classes(&[css_class]);
        let is_running = st.flow.is_active();
        if is_running {
            self.mini_toggle_btn.set_icon_name("media-playback-pause-symbolic");
            self.mini_toggle_btn.set_tooltip_text(Some(match lang {
                Language::Norwegian => "Pause flyt [Mellomrom]",
                _ => "Pause flow [Space]",
            }));
        } else {
            self.mini_toggle_btn.set_icon_name("media-playback-start-symbolic");
            self.mini_toggle_btn.set_tooltip_text(Some(match lang {
                Language::Norwegian => "Start fokusøkt [Mellomrom]",
                _ => "Start focus [Space]",
            }));
        }

        // Buttons
        match st.flow.state {
            FlowState::Idle => {
                self.main_action_btn.set_label(Strings::btn_start_focus(lang));
                self.main_action_btn.set_css_classes(&["pill", "suggested-action"]);
                self.micro_btn.set_visible(true);
                let has_residual = st.flow.elapsed_focus_secs > 0 || st.flow.banked_break_secs > 0;
                self.reset_btn.set_visible(has_residual);
            }
            FlowState::MicroStart => {
                self.main_action_btn.set_label(match lang {
                    Language::Norwegian => "✕ Avbryt (Null skyldfølelse)",
                    _ => "✕ Cancel (Zero Guilt)",
                });
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

        // Audio button state in header, Mini-HUD and Flow tab
        if st.audio.is_playing {
            self.audio_btn.set_icon_name("media-playback-pause-symbolic");
            self.audio_btn.add_css_class("accent");
            self.mini_audio_btn.remove_css_class("mini-audio-btn-stopped");
            self.mini_audio_btn.add_css_class("mini-audio-btn-playing");
            self.mini_audio_btn.set_tooltip_text(Some("Stopp fokus-lyd"));

            self.flow_audio_btn.set_icon_name("media-playback-pause-symbolic");
            self.flow_audio_btn.set_tooltip_text(Some("Pause fokus-lyd"));
            self.flow_audio_btn.add_css_class("accent");
            let sound_name = st.audio.current_type.label_no();
            self.flow_sound_row.set_subtitle(&format!("Spiller nå: {}", sound_name));
        } else {
            self.audio_btn.set_icon_name("audio-volume-high-symbolic");
            self.audio_btn.remove_css_class("accent");
            self.mini_audio_btn.remove_css_class("mini-audio-btn-playing");
            self.mini_audio_btn.add_css_class("mini-audio-btn-stopped");
            self.mini_audio_btn.set_tooltip_text(Some("Start fokus-lyd"));

            self.flow_audio_btn.set_icon_name("media-playback-start-symbolic");
            self.flow_audio_btn.set_tooltip_text(Some("Spill av fokus-lyd"));
            self.flow_audio_btn.remove_css_class("accent");
            self.flow_sound_row.set_subtitle("Prosedyrisk støy for dyp konsentrasjon");
        }

        // Editor integration state & UI visibility
        let editor = st.settings.editor_type.resolve_context(st.anchor.as_ref());
        if editor == EditorType::None {
            self.editor_btn.set_visible(false);
            if st.anchor.is_none() {
                self.anchor_group.set_visible(false);
            } else {
                self.anchor_group.set_visible(true);
            }
            self.helper_text.set_label(match lang {
                Language::Norwegian => "Mellomrom = Start/pause flyt  •  ADHD Flowmodoro",
                _ => "Space = Start/pause flow  •  ADHD Flowmodoro",
            });
        } else {
            self.editor_btn.set_visible(true);
            self.anchor_group.set_visible(true);
            let badge = editor.short_badge();
            let is_running = editor.is_running();
            let conn_status = if is_running {
                match lang { Language::Norwegian => "Tilkoblet 🟢", _ => "Connected 🟢" }
            } else {
                match lang { Language::Norwegian => "Frakoblet 🔌", _ => "Offline 🔌" }
            };
            if is_running {
                self.editor_btn.set_label(&format!("🟢 {}", badge));
            } else {
                self.editor_btn.set_label(&format!("🔌 {}", badge));
            }
            self.editor_btn.set_tooltip_text(Some(&format!("{} ({})", editor.display_name(), conn_status)));
            self.helper_text.set_label(&match lang {
                Language::Norwegian => format!("Mellomrom = Start/pause flyt  •  Integrert med {}", editor.display_name()),
                _ => format!("Space = Start/pause flow  •  Integrated with {}", editor.display_name()),
            });
        }

        // Summary row
        self.ratio_label.set_label(&match lang {
            Language::Norwegian => format!("Forhold: {}m : {}m", st.flow.focus_ratio, st.flow.break_ratio),
            _ => format!("Ratio: {}m : {}m", st.flow.focus_ratio, st.flow.break_ratio),
        });
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
            self.anchor_resume_btn.set_label(&editor.resume_button_label(lang));
            self.anchor_resume_btn.set_visible(true);
        } else {
            self.anchor_row.set_title(match lang {
                Language::Norwegian => "Ingen aktiv kontekst ennå",
                _ => "No active context yet",
            });
            if editor == EditorType::None {
                self.anchor_row.set_subtitle(match lang {
                    Language::Norwegian => "Ingen aktiv forankring",
                    _ => "No active anchor",
                });
            } else {
                self.anchor_row.set_subtitle(&match lang {
                    Language::Norwegian => format!("Åpne en kodefil i {} for å forankre flyten", editor.display_name()),
                    _ => format!("Open a code file in {} to anchor flow", editor.display_name()),
                });
            }
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

        if let Some((x, y)) = st.settings.custom_pos_mini {
            self.pos_row.set_subtitle(&format!("Lagret Mini-HUD posisjon: X: {}, Y: {}", x, y));
        } else {
            self.pos_row.set_subtitle("Standard posisjon (sentrert / følger skjerm)");
        }

        if let Some(tray) = &self.tray {
            tray.update_status(&timer_text, tag_text);
        }

        // Live Omarchy Theme check
        let cur_mtime = crate::theme::get_colors_mtime();
        let need_reload = {
            cur_mtime.is_some() && cur_mtime != st.last_theme_mtime
        };
        drop(st);

        if need_reload {
            let colors = crate::theme::load_omarchy_colors();
            self.css_provider.load_from_data(&crate::theme::generate_css(&colors));
            let mut st = self.state.borrow_mut();
            st.last_theme_mtime = cur_mtime;
        }
    }

    pub fn toggle_flow(&self) {
        let (_old_state, new_state) = {
            let mut st = self.state.borrow_mut();
            let old_state = st.flow.state;
            st.flow.toggle();
            let new_state = st.flow.state;
            st.previous_state = new_state;
            st.audio.handle_flow_state_transition(old_state, new_state);
            (old_state, new_state)
        };
        if new_state == FlowState::Break {
            self.set_mini_hud_mode(false);
            self.window.set_visible(true);
            self.window.present();
            self.ram_now_entry.grab_focus();
        }
    }

    pub fn start_micro(&self) {
        let mut st = self.state.borrow_mut();
        let old_state = st.flow.state;
        st.flow.start_micro_start();
        st.last_recorded_focus_mins = 0;
        let new_state = st.flow.state;
        st.previous_state = new_state;
        st.audio.handle_flow_state_transition(old_state, new_state);
    }

    pub fn reset_flow(&self) {
        let mut st = self.state.borrow_mut();
        st.flow.reset();
        st.last_recorded_focus_mins = 0;
        st.previous_state = FlowState::Idle;
    }

    pub fn toggle_audio(&self) {
        let (is_playing, current_type) = {
            let mut st = self.state.borrow_mut();
            if !st.audio.is_playing && st.audio.current_type == SoundscapeType::Off {
                st.audio.set_sound(SoundscapeType::BrownNoise);
                st.settings.soundscape = SoundscapeType::BrownNoise;
                let _ = storage::save_settings(&st.settings);
            }
            st.audio.toggle_play();
            (st.audio.is_playing, st.audio.current_type)
        };
        let sound_idx = current_type.to_index();
        if self.flow_sound_dropdown.selected() != sound_idx {
            self.flow_sound_dropdown.set_selected(sound_idx);
        }
        if self.sound_combo.selected() != sound_idx {
            self.sound_combo.set_selected(sound_idx);
        }

        if is_playing {
            self.mini_audio_btn.remove_css_class("mini-audio-btn-stopped");
            self.mini_audio_btn.add_css_class("mini-audio-btn-playing");
            self.mini_audio_btn.set_tooltip_text(Some("Stopp fokus-lyd"));
            self.flow_audio_btn.set_icon_name("media-playback-pause-symbolic");
            self.flow_audio_btn.set_tooltip_text(Some("Pause fokus-lyd"));
            self.flow_audio_btn.add_css_class("accent");
            let sound_name = current_type.label_no();
            self.flow_sound_row.set_subtitle(&format!("Spiller nå: {}", sound_name));
        } else {
            self.mini_audio_btn.remove_css_class("mini-audio-btn-playing");
            self.mini_audio_btn.add_css_class("mini-audio-btn-stopped");
            self.mini_audio_btn.set_tooltip_text(Some("Start fokus-lyd"));
            self.flow_audio_btn.set_icon_name("media-playback-start-symbolic");
            self.flow_audio_btn.set_tooltip_text(Some("Spill av fokus-lyd"));
            self.flow_audio_btn.remove_css_class("accent");
            self.flow_sound_row.set_subtitle("Prosedyrisk støy for dyp konsentrasjon");
        }
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
        let items = storage::load_structured_impulses_with_cleanup(&st.settings);

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
            for (original_idx, item) in items.iter().enumerate().rev().take(30) {
                let row = ActionRow::builder().build();
                let check_btn = gtk4::CheckButton::builder()
                    .active(item.done)
                    .valign(gtk4::Align::Center)
                    .build();

                if item.done {
                    let escaped = glib::markup_escape_text(&item.text);
                    row.set_title(&format!("<span strikethrough=\"true\" alpha=\"60%\">{}</span>", escaped));
                    if let Some(comp) = &item.completed_at {
                        row.set_subtitle(&format!("Fullført {}", comp));
                    } else {
                        row.set_subtitle("Fullført ✓");
                    }
                } else {
                    row.set_title(&glib::markup_escape_text(&item.text));
                    if let Some(created) = &item.created_at {
                        row.set_subtitle(&format!("Parkert {}", created));
                    }
                }

                let state_clone = self.state.clone();
                let target_idx = original_idx;
                let row_weak = row.downgrade();
                let text = item.text.clone();
                let created_at = item.created_at.clone();

                check_btn.connect_toggled(move |btn| {
                    let is_done = btn.is_active();
                    let settings = state_clone.borrow().settings.clone();
                    let _ = storage::toggle_impulse_done(&settings, target_idx, is_done);
                    if let Some(r) = row_weak.upgrade() {
                        if is_done {
                            let escaped = glib::markup_escape_text(&text);
                            r.set_title(&format!("<span strikethrough=\"true\" alpha=\"60%\">{}</span>", escaped));
                            let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
                            r.set_subtitle(&format!("Fullført {}", now_str));
                        } else {
                            r.set_title(&glib::markup_escape_text(&text));
                            if let Some(created) = &created_at {
                                r.set_subtitle(&format!("Parkert {}", created));
                            } else {
                                r.set_subtitle("");
                            }
                        }
                    }
                });

                row.add_prefix(&check_btn);
                self.impulses_list_box.append(&row);
            }
        }
    }

    pub fn volume_icon_name(vol_pct: f64) -> &'static str {
        if vol_pct <= 0.01 {
            "audio-volume-muted-symbolic"
        } else if vol_pct < 33.0 {
            "audio-volume-low-symbolic"
        } else if vol_pct < 67.0 {
            "audio-volume-medium-symbolic"
        } else {
            "audio-volume-high-symbolic"
        }
    }
}
