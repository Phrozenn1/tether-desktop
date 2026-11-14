use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use chrono::Timelike;
use gtk4::prelude::*;
use libadwaita::glib;
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, Clamp, ComboRow, EntryRow, HeaderBar,
    PreferencesGroup, PreferencesPage, SpinRow, SwitchRow, ToolbarView, ViewStack, ViewSwitcher, WindowTitle,
};

use crate::audio::{SoundscapePlayer, SoundscapeType};
use crate::editor::EditorType;
use crate::flowmodoro::{FlowState, Flowmodoro};
use crate::i18n::Strings;
use crate::impulse::ImpulseState;
use crate::ramdump::RamDumpState;
use crate::storage::{self, ContextAnchor, Language, LanguageSetting, SideQuestFormat, TetherSettings, UserStats};

pub const FULL_WINDOW_WIDTH: i32 = 940;
pub const FULL_WINDOW_HEIGHT: i32 = 890;
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
    pub speedbump: crate::speedbump::SpeedbumpState,
    pub speedbump_active_prompt: bool,
    pub dopamine: crate::dopamine::DopamineAccount,
    pub time_anchor: crate::time_anchor::TimeAnchorState,
    pub where_was_i_dismissed: bool,
    pub shutdown_window_open: bool,
    pub last_update: Instant,
    pub last_anchor_check: Instant,
    pub last_stats_check: Instant,
    pub last_pos_check: Instant,
    pub last_recorded_focus_mins: u64,
    pub is_mini_hud: bool,
    pub suppress_pos_ticks: u32,
    pub asked_editor_minimize_prompt: bool,
    pub last_theme_mtime: Option<std::time::SystemTime>,
    pub editing_ramdump_idx: Option<usize>,
}

pub struct TetherUi {
    pub state: Rc<RefCell<AppState>>,
    pub window: gtk4::ApplicationWindow,
    pub main_box: gtk4::Box,
    pub flow_box: gtk4::Box,
    pub hero_card: gtk4::Box,
    pub mini_hud_box: gtk4::Box,
    pub css_provider: gtk4::CssProvider,

    // Dynamic header widgets
    pub status_badge: gtk4::Label,
    pub audio_btn: gtk4::Button,
    pub dopamine_btn: gtk4::Button,
    pub shutdown_btn: gtk4::Button,
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
    pub anchor_where_btn: gtk4::Button,
    pub anchor_resume_btn: gtk4::Button,

    // "Where was I?" welcome card widgets
    pub where_was_i_group: PreferencesGroup,
    pub where_state_row: ActionRow,
    pub where_git_row: ActionRow,
    pub where_next_row: ActionRow,
    pub where_start_btn: gtk4::Button,

    // Time-blindness & Hard Stop widgets
    pub hard_stop_row: ActionRow,
    pub hard_stop_entry: gtk4::Entry,
    pub hard_stop_label_entry: gtk4::Entry,
    pub hard_stop_toggle_btn: gtk4::Button,

    // Momentum widgets
    pub momentum_tier_row: ActionRow,
    pub momentum_progress: gtk4::ProgressBar,
    pub stat_flow_val: gtk4::Label,
    pub stat_sessions_val: gtk4::Label,
    pub stat_impulses_val: gtk4::Label,
    pub stat_dopamine_val: gtk4::Label,

    // RAM-dump widgets
    pub ram_now_entry: EntryRow,
    pub ram_status_entry: EntryRow,
    pub ram_next_entry: EntryRow,
    pub autofill_ram_btn: gtk4::Button,
    pub save_ram_btn: gtk4::Button,
    pub cancel_edit_ram_btn: gtk4::Button,
    pub last_dump_group: PreferencesGroup,
    pub last_dump_row: ActionRow,
    pub ramdumps_list_box: gtk4::ListBox,

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
        let initial_lang = initial_settings.language.resolve();
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
        let speedbump = crate::speedbump::SpeedbumpState::new();

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
        let dopamine = crate::dopamine::load_dopamine_account();
        let initial_dopamine_balance = dopamine.balance;

        let state = Rc::new(RefCell::new(AppState {
            flow,
            audio,
            previous_state,
            settings,
            stats,
            anchor,
            impulse,
            ramdump,
            speedbump,
            speedbump_active_prompt: false,
            dopamine,
            time_anchor: crate::time_anchor::TimeAnchorState::new(),
            where_was_i_dismissed: false,
            shutdown_window_open: false,
            last_update: Instant::now(),
            last_anchor_check: Instant::now(),
            last_stats_check: Instant::now(),
            last_pos_check: Instant::now(),
            last_recorded_focus_mins: 0,
            is_mini_hud: false,
            suppress_pos_ticks: 0,
            asked_editor_minimize_prompt: false,
            last_theme_mtime: initial_theme_mtime,
            editing_ramdump_idx: None,
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

        // Left Header: Dopamine Account / Rewards Button
        let dopamine_btn = gtk4::Button::builder()
            .label(&format!("{} p", initial_dopamine_balance))
            .tooltip_text(match initial_lang {
                Language::Norwegian => "Dopaminkonto & Belønningsmeny",
                _ => "Dopamine Account & Rewards",
            })
            .css_classes(["flat", "pill"])
            .build();
        header_bar.pack_start(&dopamine_btn);

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

        // Right Header: Shutdown Ritual Button
        let shutdown_btn = gtk4::Button::builder()
            .icon_name("weather-clear-night-symbolic")
            .tooltip_text(match initial_lang {
                Language::Norwegian => "Dagsavslutning (Tøm hodet og avslutt arbeidsdagen)",
                _ => "Shutdown Ritual (Park working memory for the day)",
            })
            .css_classes(["flat"])
            .build();
        header_bar.pack_end(&shutdown_btn);

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

        // "Where was I?" Context Recovery Card (shown after absence >= 4h or manually)
        let where_was_i_group = PreferencesGroup::builder()
            .title(match initial_lang {
                Language::Norwegian => "Hvor var jeg?",
                _ => "Where was I?",
            })
            .description(match initial_lang {
                Language::Norwegian => "Rask oversikt etter fravær",
                _ => "Quick context recovery",
            })
            .visible(false)
            .build();

        let where_state_row = ActionRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Siste tilstand",
                _ => "Last state",
            })
            .build();
        where_state_row.add_prefix(&gtk4::Image::from_icon_name("document-save-symbolic"));
        let where_dismiss_btn = gtk4::Button::builder()
            .icon_name("window-close-symbolic")
            .tooltip_text(match initial_lang {
                Language::Norwegian => "Lukk oversikt",
                _ => "Dismiss card",
            })
            .css_classes(["flat", "circular"])
            .valign(gtk4::Align::Center)
            .build();
        where_state_row.add_suffix(&where_dismiss_btn);
        where_was_i_group.add(&where_state_row);

        let where_git_row = ActionRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Git og kodekontekst",
                _ => "Git & code context",
            })
            .build();
        where_git_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        where_was_i_group.add(&where_git_row);

        let where_next_row = ActionRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Foreslått første mikro-steg",
                _ => "Suggested first micro-step",
            })
            .build();
        where_next_row.add_prefix(&gtk4::Image::from_icon_name("weather-storm-symbolic"));
        let where_start_btn = gtk4::Button::builder()
            .label(match initial_lang {
                Language::Norwegian => "Start 5m & åpne",
                _ => "Start 5m & open",
            })
            .css_classes(["suggested-action", "pill"])
            .valign(gtk4::Align::Center)
            .build();
        where_next_row.add_suffix(&where_start_btn);
        where_was_i_group.add(&where_next_row);

        flow_box.append(&where_was_i_group);

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
            .label(match initial_lang {
                Language::Norwegian => "Start fokus",
                _ => "Start Focus",
            })
            .css_classes(["pill", "suggested-action"])
            .width_request(140)
            .build();
        btn_row.append(&main_action_btn);

        let micro_btn = gtk4::Button::builder()
            .label(match initial_lang {
                Language::Norwegian => "5m mikrostart",
                _ => "5m Micro-start",
            })
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
            .title(match initial_lang {
                Language::Norwegian => "Fokuslyd",
                _ => "Focus Soundscape",
            })
            .description(match initial_lang {
                Language::Norwegian => "Akustisk støymaskering og beroligende naturlyder",
                _ => "Acoustic noise masking and soothing nature soundscapes",
            })
            .build();

        let flow_sound_row = ActionRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Lydkulisse",
                _ => "Soundscape",
            })
            .subtitle(match initial_lang {
                Language::Norwegian => "Spilles automatisk av under fokus",
                _ => "Plays automatically during focus",
            })
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

        // Time-Blindness Anchor & Hard Stop Group
        let time_anchor_group = PreferencesGroup::builder()
            .title(match initial_lang {
                Language::Norwegian => "Tidsanker & Hard Stop",
                _ => "Time Anchor & Hard Stop",
            })
            .description(match initial_lang {
                Language::Norwegian => "Sett et fast stopptidspunkt så du ikke glemmer avtaler under hyperfokus",
                _ => "Set a firm stop time so you never miss appointments during hyperfocus",
            })
            .build();

        let hard_stop_row = ActionRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Hard Stop-klokkeslett",
                _ => "Hard Stop time",
            })
            .subtitle(match initial_lang {
                Language::Norwegian => "Ingen Hard Stop satt (varsler 15m og 5m før)",
                _ => "No Hard Stop set (alerts 15m and 5m before)",
            })
            .build();
        hard_stop_row.add_prefix(&gtk4::Image::from_icon_name("alarm-symbolic"));

        let hard_stop_controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        hard_stop_controls.set_valign(gtk4::Align::Center);

        let hard_stop_entry = gtk4::Entry::builder()
            .placeholder_text("14:00")
            .text(&initial_settings.hard_stop_time)
            .width_chars(6)
            .max_length(5)
            .valign(gtk4::Align::Center)
            .build();
        hard_stop_controls.append(&hard_stop_entry);

        let hard_stop_label_entry = gtk4::Entry::builder()
            .placeholder_text(match initial_lang {
                Language::Norwegian => "Møte / Avtale",
                _ => "Meeting / Event",
            })
            .text(&initial_settings.hard_stop_label)
            .width_chars(14)
            .valign(gtk4::Align::Center)
            .build();
        hard_stop_controls.append(&hard_stop_label_entry);

        let hard_stop_toggle_btn = gtk4::Button::builder()
            .label(if initial_settings.hard_stop_enabled {
                match initial_lang {
                    Language::Norwegian => "Avbryt",
                    _ => "Clear",
                }
            } else {
                match initial_lang {
                    Language::Norwegian => "Aktiver",
                    _ => "Set",
                }
            })
            .css_classes(if initial_settings.hard_stop_enabled {
                &["pill", "destructive-action"][..]
            } else {
                &["pill"][..]
            })
            .valign(gtk4::Align::Center)
            .build();
        hard_stop_controls.append(&hard_stop_toggle_btn);

        hard_stop_row.add_suffix(&hard_stop_controls);
        time_anchor_group.add(&hard_stop_row);
        flow_box.append(&time_anchor_group);

        // Context Anchor Group
        let anchor_group = PreferencesGroup::builder()
            .title(match initial_lang {
                Language::Norwegian => "Kontekstanker",
                _ => "Context Anchor",
            })
            .description(match initial_lang {
                Language::Norwegian => "Husker nøyaktig hvor du slapp i koden eller prosjektet",
                _ => "Remembers exactly where you left off in code or project",
            })
            .build();

        let anchor_row = ActionRow::builder()
            .title("Ingen aktiv kontekst ennå")
            .subtitle("Åpne et prosjekt for å forankre flyten")
            .build();
        let anchor_where_btn = gtk4::Button::builder()
            .label(match initial_lang {
                Language::Norwegian => "Hvor var jeg?",
                _ => "Where was I?",
            })
            .tooltip_text(match initial_lang {
                Language::Norwegian => "Vis sammendrag av siste økt, Git-status og neste steg",
                _ => "Show summary of last session, Git status, and next step",
            })
            .css_classes(["pill"])
            .valign(gtk4::Align::Center)
            .build();
        let anchor_resume_btn = gtk4::Button::builder()
            .label(match initial_lang {
                Language::Norwegian => "Gjenoppta",
                _ => "Resume",
            })
            .css_classes(["suggested-action", "pill"])
            .valign(gtk4::Align::Center)
            .build();
        anchor_resume_btn.set_visible(false);
        anchor_row.add_suffix(&anchor_where_btn);
        anchor_row.add_suffix(&anchor_resume_btn);
        anchor_group.add(&anchor_row);
        flow_box.append(&anchor_group);

        // Momentum / XP Group
        let momentum_group = PreferencesGroup::builder()
            .title(match initial_lang {
                Language::Norwegian => "Momentum og flyt-XP",
                _ => "Momentum and Flow XP",
            })
            .description(match initial_lang {
                Language::Norwegian => "Positiv framdrift – ingen brutte streaks",
                _ => "Positive progression – no broken streaks",
            })
            .build();

        let momentum_tier_row = ActionRow::builder()
            .title("Gnist (0 XP)")
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

        let pill4 = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        pill4.add_css_class("stat-pill");
        let stat_dopamine_val = gtk4::Label::builder()
            .label(&format!("{} p", initial_dopamine_balance))
            .css_classes(["title-4"])
            .build();
        let pill4_lbl = gtk4::Label::builder()
            .label(match initial_lang {
                Language::Norwegian => "Dopaminpoeng",
                _ => "Dopamine points",
            })
            .css_classes(["caption", "dim-label"])
            .build();
        pill4.append(&stat_dopamine_val);
        pill4.append(&pill4_lbl);
        stat_pill_row.append(&pill4);

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

        let ram_btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        ram_btn_row.set_halign(gtk4::Align::End);
        ram_btn_row.set_margin_top(12);

        let cancel_edit_ram_btn = gtk4::Button::builder()
            .label(match initial_lang {
                Language::Norwegian => "Avbryt redigering",
                _ => "Cancel Edit",
            })
            .css_classes(["pill"])
            .visible(false)
            .build();
        ram_btn_row.append(&cancel_edit_ram_btn);

        let autofill_ram_btn = gtk4::Button::builder()
            .label(match initial_lang {
                Language::Norwegian => "✨ Fyll ut automatisk (Kontekst / AI)",
                _ => "✨ Auto-fill (Context / AI)",
            })
            .tooltip_text(match initial_lang {
                Language::Norwegian => "Analyserer aktivt dokument, vindu eller prosjekt",
                _ => "Analyzes active document, window or project",
            })
            .css_classes(["pill"])
            .build();
        ram_btn_row.append(&autofill_ram_btn);

        let save_ram_btn = gtk4::Button::builder()
            .label(match initial_lang {
                Language::Norwegian => "Lagre arbeidsminne",
                _ => "Save Working Memory",
            })
            .css_classes(["pill", "suggested-action"])
            .build();
        ram_btn_row.append(&save_ram_btn);
        ram_group.add(&ram_btn_row);

        ram_box.append(&ram_group);

        let last_dump_group = PreferencesGroup::builder()
            .title(match initial_lang {
                Language::Norwegian => "Lagrede RAM-dumps (Rediger / Slett)",
                _ => "Saved RAM Dumps (Edit / Delete)",
            })
            .build();
        let last_dump_row = ActionRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Ingen dump lagret ennå",
                _ => "No dump saved yet",
            })
            .subtitle(match initial_lang {
                Language::Norwegian => "Fyll ut feltene ovenfor og trykk Lagre for å sikre arbeidsminnet",
                _ => "Fill in the fields above and click Save to secure working memory",
            })
            .build();
        last_dump_group.add(&last_dump_row);

        let ramdumps_list_box = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(["boxed-list"])
            .margin_top(8)
            .build();
        last_dump_group.add(&ramdumps_list_box);
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

        // Group: Speedbump (Soft Interceptor)
        let set_speedbump_group = PreferencesGroup::builder()
            .title(match initial_lang {
                Language::Norwegian => "Fartsdump (Avsporingsvern)",
                _ => "Speedbump (Distraction Shield)",
            })
            .description(match initial_lang {
                Language::Norwegian => "Pustepause og impulsfangst hvis du åpner kaninhull under fokus",
                _ => "Gentle pause and impulse capture if rabbit holes open during focus",
            })
            .build();

        let speedbump_switch = SwitchRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Aktiver fartsdump under fokus",
                _ => "Enable speedbump during focus",
            })
            .subtitle(match initial_lang {
                Language::Norwegian => "Viser en rolig påminnelse om aktivt vindu matcher kjente distraksjoner",
                _ => "Shows a gentle prompt if active window matches known distractions",
            })
            .active(initial_settings.speedbump_enabled)
            .build();
        set_speedbump_group.add(&speedbump_switch);

        let speedbump_cooldown_spin = SpinRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Nedkjølingstid (sekunder)",
                _ => "Cooldown (seconds)",
            })
            .subtitle(match initial_lang {
                Language::Norwegian => "Minimum tid mellom påminnelser for samme avsporing",
                _ => "Minimum interval between prompts for the same distraction",
            })
            .adjustment(&gtk4::Adjustment::new(initial_settings.speedbump_cooldown_secs as f64, 10.0, 600.0, 5.0, 30.0, 0.0))
            .numeric(true)
            .build();
        set_speedbump_group.add(&speedbump_cooldown_spin);

        let speedbump_triggers_entry = EntryRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Kaninhull-nøkkelord (kommaseparert)",
                _ => "Rabbit hole keywords (comma-separated)",
            })
            .text(&initial_settings.speedbump_triggers.join(", "))
            .build();
        set_speedbump_group.add(&speedbump_triggers_entry);

        settings_page.add(&set_speedbump_group);

        // Group: Time-Blindness Cues & AI Assist
        let set_time_anchor_group = PreferencesGroup::builder()
            .title(match initial_lang {
                Language::Norwegian => "Tidsblindhets-ankring og AI-assistent",
                _ => "Time-Blindness Cues & AI Assistant",
            })
            .description(match initial_lang {
                Language::Norwegian => "Myke tidsankre under hyperfokus og smart utfylling av RAM-dump",
                _ => "Gentle time cues during hyperfocus and smart RAM-dump auto-fill",
            })
            .build();

        let time_cue_models = gtk4::StringList::new(&[
            "Av",
            "Hver halvtime (:00 og :30)",
            "Hver hele time (:00)",
        ]);
        let time_cue_idx = match initial_settings.time_cue_interval_mins {
            0 => 0,
            60 => 2,
            _ => 1,
        };
        let time_cue_combo = ComboRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Myke tidsankre under fokus",
                _ => "Ambient time cues during focus",
            })
            .subtitle(match initial_lang {
                Language::Norwegian => "Gir en subtil visuell puls så du merker at tiden går",
                _ => "Shows a subtle visual pulse so you notice time passing",
            })
            .model(&time_cue_models)
            .selected(time_cue_idx)
            .build();
        set_time_anchor_group.add(&time_cue_combo);

        let time_cue_sound_switch = SwitchRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Akustisk vanndråpe-lyd ved tidsanker",
                _ => "Soft acoustic water-droplet chime at time cue",
            })
            .active(initial_settings.time_cue_sound)
            .build();
        set_time_anchor_group.add(&time_cue_sound_switch);

        let ai_mode_models = gtk4::StringList::new(&[
            "Auto (Antigravity / Copilot / CLI / Ollama → Git)",
            "Antigravity CLI (agy)",
            "GitHub Copilot CLI (copilot)",
            "Gemini CLI (gemini)",
            "OpenAI Codex CLI (codex)",
            "OpenCode CLI (opencode)",
            "Claude CLI (claude)",
            "Lokal Ollama (localhost:11434)",
            "Kun Git- og kodeanalyse (100% offline)",
        ]);
        let ai_mode_idx = match initial_settings.ai_assist_mode {
            storage::AiAssistMode::Auto | storage::AiAssistMode::Cli => 0,
            storage::AiAssistMode::Antigravity => 1,
            storage::AiAssistMode::Copilot => 2,
            storage::AiAssistMode::Gemini => 3,
            storage::AiAssistMode::Codex => 4,
            storage::AiAssistMode::OpenCode => 5,
            storage::AiAssistMode::Claude => 6,
            storage::AiAssistMode::Ollama => 7,
            storage::AiAssistMode::GitOnly => 8,
        };
        let detected_tools = crate::context_engine::detect_installed_ai_tools();
        let ai_detected_sub = if detected_tools.is_empty() {
            match initial_lang {
                Language::Norwegian => "Funnet på maskinen: ingen AI-CLI (bruker Git-analyse)".to_string(),
                _ => "Detected on machine: no AI CLI (using Git analysis)".to_string(),
            }
        } else {
            match initial_lang {
                Language::Norwegian => format!("Funnet på maskinen: {}", detected_tools.join(", ")),
                _ => format!("Detected: {}", detected_tools.join(", ")),
            }
        };
        let ai_mode_combo = ComboRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "AI / Git-assistent for RAM-dump",
                _ => "AI / Git assistant for RAM-dump",
            })
            .subtitle(&ai_detected_sub)
            .model(&ai_mode_models)
            .selected(ai_mode_idx)
            .build();
        set_time_anchor_group.add(&ai_mode_combo);

        let ollama_model_entry = EntryRow::builder()
            .title(match initial_lang {
                Language::Norwegian => "Lokal Ollama-modell (localhost:11434)",
                _ => "Local Ollama model (localhost:11434)",
            })
            .text(&initial_settings.ollama_model)
            .show_apply_button(true)
            .build();
        set_time_anchor_group.add(&ollama_model_entry);

        settings_page.add(&set_time_anchor_group);

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

        // Group: Arbeidsprogrammer og Kontekst-ankring
        let set_editor_group = PreferencesGroup::builder()
            .title("Arbeidsprogrammer og Kontekst-ankring")
            .description("Sømløs integrasjon med skriveprogrammer, notater, studieverktøy og kreative apper")
            .build();

        let available_editors = EditorType::available_choices();
        let editor_names: Vec<&'static str> = available_editors.iter().map(|e| e.display_name()).collect();
        let editor_models = gtk4::StringList::new(&editor_names);
        let selected_editor_idx = available_editors
            .iter()
            .position(|&e| e == initial_settings.editor_type)
            .unwrap_or(0) as u32;

        let editor_combo = ComboRow::builder()
            .title("Foretrukket arbeidsprogram")
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

        let ext_status_row = ActionRow::builder()
            .title("Tether app-integrasjon")
            .build();
        let install_ext_btn = gtk4::Button::builder()
            .valign(gtk4::Align::Center)
            .build();
        Self::apply_ext_row_status(
            &ext_status_row,
            &install_ext_btn,
            Some(&nvim_mgr_combo),
            initial_settings.editor_type.resolve(),
            initial_settings.nvim_plugin_manager,
        );
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
            .label("TETHER")
            .tooltip_text("Tether Mini-HUD")
            .valign(gtk4::Align::Center)
            .css_classes(["heading", "dim-label"])
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
            .tooltip_text("5m mikrostart")
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
            flow_box: flow_box.clone(),
            hero_card,
            mini_hud_box,
            css_provider,
            status_badge,
            audio_btn,
            dopamine_btn,
            shutdown_btn,
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
            anchor_where_btn,
            anchor_resume_btn,
            where_was_i_group,
            where_state_row,
            where_git_row,
            where_next_row,
            where_start_btn,
            hard_stop_row,
            hard_stop_entry,
            hard_stop_label_entry,
            hard_stop_toggle_btn,
            momentum_tier_row,
            momentum_progress,
            stat_flow_val,
            stat_sessions_val,
            stat_impulses_val,
            stat_dopamine_val,
            ram_now_entry,
            ram_status_entry,
            ram_next_entry,
            autofill_ram_btn,
            save_ram_btn,
            cancel_edit_ram_btn,
            last_dump_group,
            last_dump_row,
            ramdumps_list_box,
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

        // Event: Dopamine header button (Open Dopamine Menu)
        {
            let ui_clone = ui.clone();
            ui.dopamine_btn.connect_clicked(move |_| {
                ui_clone.open_dopamine_menu();
            });
        }

        // Event: Shutdown header button (Open Shutdown Ritual)
        {
            let ui_clone = ui.clone();
            ui.shutdown_btn.connect_clicked(move |_| {
                ui_clone.open_shutdown_ritual();
            });
        }

        // Event: Where-was-I dismiss button
        {
            let ui_clone = ui.clone();
            where_dismiss_btn.connect_clicked(move |_| {
                ui_clone.state.borrow_mut().where_was_i_dismissed = true;
                ui_clone.where_was_i_group.set_visible(false);
            });
        }

        // Event: Where-was-I Start 5m & open editor
        {
            let ui_clone = ui.clone();
            ui.where_start_btn.connect_clicked(move |_| {
                ui_clone.start_micro();
                {
                    let mut st = ui_clone.state.borrow_mut();
                    st.where_was_i_dismissed = true;
                    if let Some(anchor) = &st.anchor {
                        st.settings.editor_type.resume(anchor, &st.settings.terminal_command);
                    } else {
                        st.settings.editor_type.open_or_focus(&st.settings.terminal_command);
                    }
                }
                ui_clone.where_was_i_group.set_visible(false);
            });
        }

        // Event: Anchor "Where was I?" button
        {
            let ui_clone = ui.clone();
            ui.anchor_where_btn.connect_clicked(move |_| {
                ui_clone.state.borrow_mut().where_was_i_dismissed = false;
                ui_clone.refresh_where_was_i(true);
            });
        }

        // Event: Hard Stop toggle button
        {
            let ui_clone = ui.clone();
            ui.hard_stop_toggle_btn.connect_clicked(move |_| {
                ui_clone.toggle_hard_stop();
            });
        }

        // Event: Time cue interval & sound & AI mode settings
        {
            let state_clone = state.clone();
            time_cue_combo.connect_selected_notify(move |combo| {
                let interval = match combo.selected() {
                    0 => 0,
                    2 => 60,
                    _ => 30,
                };
                let mut st = state_clone.borrow_mut();
                st.settings.time_cue_interval_mins = interval;
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            time_cue_sound_switch.connect_active_notify(move |sw| {
                let mut st = state_clone.borrow_mut();
                st.settings.time_cue_sound = sw.is_active();
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            ai_mode_combo.connect_selected_notify(move |combo| {
                let mode = match combo.selected() {
                    1 => storage::AiAssistMode::Antigravity,
                    2 => storage::AiAssistMode::Copilot,
                    3 => storage::AiAssistMode::Gemini,
                    4 => storage::AiAssistMode::Codex,
                    5 => storage::AiAssistMode::OpenCode,
                    6 => storage::AiAssistMode::Claude,
                    7 => storage::AiAssistMode::Ollama,
                    8 => storage::AiAssistMode::GitOnly,
                    _ => storage::AiAssistMode::Auto,
                };
                let mut st = state_clone.borrow_mut();
                st.settings.ai_assist_mode = mode;
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            ollama_model_entry.connect_apply(move |entry| {
                let mut st = state_clone.borrow_mut();
                let txt = entry.text().trim().to_string();
                if !txt.is_empty() {
                    st.settings.ollama_model = txt;
                    let _ = storage::save_settings(&st.settings);
                }
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

        // Event: Auto-fill RAM-dump from Context / AI
        {
            let ui_clone = ui.clone();
            ui.autofill_ram_btn.connect_clicked(move |_| {
                ui_clone.autofill_ramdump();
            });
        }

        // Event: Save or update RAM-dump
        {
            let ui_clone = ui.clone();
            ui.save_ram_btn.connect_clicked(move |_| {
                ui_clone.save_ramdump();
            });
        }

        // Event: Cancel RAM-dump editing
        {
            let ui_clone = ui.clone();
            ui.cancel_edit_ram_btn.connect_clicked(move |_| {
                ui_clone.cancel_edit_ramdump();
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
            speedbump_switch.connect_active_notify(move |sw| {
                let mut st = state_clone.borrow_mut();
                st.settings.speedbump_enabled = sw.is_active();
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            speedbump_cooldown_spin.connect_value_notify(move |spin| {
                let mut st = state_clone.borrow_mut();
                st.settings.speedbump_cooldown_secs = spin.value().round() as u64;
                let _ = storage::save_settings(&st.settings);
            });
        }
        {
            let state_clone = state.clone();
            speedbump_triggers_entry.connect_changed(move |entry| {
                let text = entry.text();
                let triggers: Vec<String> = text
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                let mut st = state_clone.borrow_mut();
                st.settings.speedbump_triggers = triggers;
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
                    Self::apply_ext_row_status(
                        &ext_status_row_clone,
                        &install_ext_btn_clone,
                        Some(&nvim_mgr_combo_clone),
                        resolved,
                        st.settings.nvim_plugin_manager,
                    );
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
                let resolved = {
                    let mut st = state_clone.borrow_mut();
                    st.settings.nvim_plugin_manager = mgr;
                    let _ = storage::save_settings(&st.settings);
                    st.settings.editor_type.resolve()
                };
                Self::apply_ext_row_status(
                    &ext_status_row_clone,
                    &install_ext_btn_clone,
                    None,
                    resolved,
                    mgr,
                );
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
                    EditorType::Obsidian => {
                        crate::editor::install_obsidian_plugin()
                    }
                    EditorType::VSCode | EditorType::VSCodium | EditorType::Cursor => {
                        crate::editor::install_vscode_extension(editor)
                    }
                    _ => {
                        Ok("Automatisk vindu- og filsporing er allerede aktivert for dette programmet.".to_string())
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

                        Self::apply_ext_row_status(
                            &ext_status_row_clone,
                            &install_ext_btn_clone,
                            None,
                            editor,
                            mgr,
                        );
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

        // Populate initial impulses & context recovery card
        ui.refresh_impulses_list();
        ui.refresh_where_was_i(false);
        ui.refresh_last_ramdump_row();
        ui.refresh_ramdumps_list();

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

        {
            let ui_clone = ui.clone();
            glib::idle_add_local_once(move || {
                if !ui_clone.state.borrow().is_mini_hud {
                    let full_h = ui_clone.natural_full_height();
                    ui_clone.window.set_default_size(FULL_WINDOW_WIDTH, full_h);
                    #[cfg(target_os = "linux")]
                    {
                        storage::resize_window(FULL_WINDOW_WIDTH, full_h);
                    }
                }
            });
        }

        ui
    }

    pub fn natural_full_height(&self) -> i32 {
        let header_h = self
            .main_box
            .first_child()
            .map(|w| w.measure(gtk4::Orientation::Vertical, FULL_WINDOW_WIDTH).1)
            .unwrap_or(48);
        let (_, flow_nat_h, _, _) = self.flow_box.measure(gtk4::Orientation::Vertical, FULL_WINDOW_WIDTH);
        (header_h + flow_nat_h).max(FULL_WINDOW_HEIGHT)
    }

    pub fn set_mini_hud_mode(&self, is_mini: bool) {
        let full_h = self.natural_full_height();
        if is_mini {
            // Save full window position before shrinking (validated & clamped)
            if let Some((cur_x, cur_y)) = storage::get_tether_window_pos() {
                let locked = self.state.borrow().settings.locked_monitor.clone();
                let clamped = storage::clamp_full_window_pos(cur_x, cur_y, FULL_WINDOW_WIDTH, full_h, &locked);
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
                        storage::clamp_full_window_pos(x, y, FULL_WINDOW_WIDTH, full_h, &st.settings.locked_monitor)
                    } else {
                        storage::get_centered_full_pos(FULL_WINDOW_WIDTH, full_h, &st.settings.locked_monitor)
                    }
                };
                // Move and resize window BEFORE setting full content so it doesn't jump or expand off-screen
                storage::move_window_to_pos(target_x, target_y);
                storage::resize_window(FULL_WINDOW_WIDTH, full_h);
            }

            self.window.set_size_request(FULL_WINDOW_MIN_WIDTH, FULL_WINDOW_MIN_HEIGHT);
            self.window.set_decorated(true);
            self.window.set_child(Some(&self.main_box));
            self.window.set_default_size(FULL_WINDOW_WIDTH, full_h);

            #[cfg(target_os = "linux")]
            {
                let (target_x, target_y) = {
                    let st = self.state.borrow();
                    if let Some((x, y)) = st.settings.custom_pos_full {
                        storage::clamp_full_window_pos(x, y, FULL_WINDOW_WIDTH, full_h, &st.settings.locked_monitor)
                    } else {
                        storage::get_centered_full_pos(FULL_WINDOW_WIDTH, full_h, &st.settings.locked_monitor)
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

        alert.add_response("tray", "Minimer til systemfelt");
        alert.add_response("mini", "Minimer til Mini-HUD");
        alert.add_response("keep", "Behold åpen");

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

    pub fn prompt_speedbump(&self, win: crate::speedbump::ActiveWindowInfo, trigger: String) {
        let (lang_str, elapsed_mins) = {
            let st = self.state.borrow();
            let lang = match st.settings.language.resolve() {
                Language::Norwegian => "no",
                _ => "en",
            };
            let mins = st.flow.elapsed_focus_secs / 60;
            (lang, mins)
        };

        let title_clean = if win.title.trim().is_empty() {
            win.class.clone()
        } else {
            win.title.clone()
        };

        // Desktop notification as quiet alert alongside centered popup window
        let notify_title = if lang_str == "no" { "Fartsdump" } else { "Speedbump" };
        let notify_body = if lang_str == "no" {
            format!("Du har vært i flyt i {} minutter. Er \"{}\" et kaninhull?", elapsed_mins, trigger)
        } else {
            format!("You've been in flow for {} minutes. Is \"{}\" a rabbit hole?", elapsed_mins, trigger)
        };
        crate::flowmodoro::notify(notify_title, &notify_body);

        let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("tether"));
        let state = self.state.clone();
        let target_title = title_clean.clone();
        let win_addr = win.address.clone();

        let (tx, rx) = std::sync::mpsc::channel::<String>();

        glib::timeout_add_local(Duration::from_millis(80), move || {
            if let Ok(choice) = rx.try_recv() {
                match choice.as_str() {
                    "impulse" => {
                        let impulse_text = format!("Sjekk: {}", target_title);
                        {
                            let mut st = state.borrow_mut();
                            st.speedbump_active_prompt = false;
                            st.impulse.input_text = impulse_text;
                            let settings = st.settings.clone();
                            let _ = st.impulse.capture_with_settings(&settings);
                            st.stats.record_impulse();
                            st.dopamine.earn_impulse_shield_bonus();
                            let _ = crate::dopamine::save_dopamine_account(&st.dopamine);
                            let _ = storage::save_user_stats(&st.stats);
                        }
                        crate::speedbump::refocus_editor();
                    }
                    "relevant" => {
                        let mut st = state.borrow_mut();
                        st.speedbump_active_prompt = false;
                        st.speedbump.snooze(&win_addr);
                    }
                    "break" => {
                        let mut st = state.borrow_mut();
                        st.speedbump_active_prompt = false;
                        let old_state = st.flow.state;
                        st.flow.start_break();
                        let new_state = st.flow.state;
                        st.previous_state = new_state;
                        st.audio.handle_flow_state_transition(old_state, new_state);
                    }
                    _ => {
                        let mut st = state.borrow_mut();
                        st.speedbump_active_prompt = false;
                    }
                }
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });

        let target_title_for_thread = title_clean;
        let win_addr_for_thread = win.address;

        std::thread::spawn(move || {
            let output = std::process::Command::new(&exe)
                .args([
                    "--speedbump",
                    &elapsed_mins.to_string(),
                    &target_title_for_thread,
                    &trigger,
                    &win_addr_for_thread,
                    lang_str,
                ])
                .output();

            let choice = match output {
                Ok(out) => {
                    let s = String::from_utf8_lossy(&out.stdout);
                    if s.contains("impulse") {
                        "impulse".to_string()
                    } else if s.contains("break") {
                        "break".to_string()
                    } else {
                        "relevant".to_string()
                    }
                }
                Err(_) => "relevant".to_string(),
            };

            let _ = tx.send(choice);
        });
    }

    pub fn open_dopamine_menu(&self) {
        let is_no = {
            let st = self.state.borrow();
            match st.settings.language.resolve() {
                Language::Norwegian => true,
                _ => false,
            }
        };

        let dialog = gtk4::Window::builder()
            .title(if is_no { "Dopaminmeny" } else { "Dopamine Menu" })
            .default_width(540)
            .default_height(580)
            .modal(false)
            .transient_for(&self.window)
            .build();

        let toolbar_view = ToolbarView::new();
        let header_bar = HeaderBar::new();
        let title_widget = WindowTitle::new(
            if is_no { "Dopaminmeny" } else { "Dopamine Menu" },
            if is_no { "Dopaminkonto & Belønninger" } else { "Dopamine Account & Rewards" },
        );
        header_bar.set_title_widget(Some(&title_widget));
        toolbar_view.add_top_bar(&header_bar);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .hexpand(true)
            .build();

        let clamp = Clamp::builder()
            .maximum_size(520)
            .tightening_threshold(460)
            .build();

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content_box.set_margin_top(16);
        content_box.set_margin_bottom(24);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);

        let state = self.state.clone();
        let dopamine_btn = self.dopamine_btn.clone();
        let stat_dopamine_val = self.stat_dopamine_val.clone();
        let content_box_rc = Rc::new(content_box);

        type RenderCallback = Rc<RefCell<Option<Box<dyn Fn()>>>>;
        let render_cell: RenderCallback = Rc::new(RefCell::new(None));
        let render_cell_clone = render_cell.clone();

        let content_box_clone = content_box_rc.clone();
        *render_cell.borrow_mut() = Some(Box::new(move || {
            // Remove previous children
            while let Some(child) = content_box_clone.first_child() {
                content_box_clone.remove(&child);
            }

            let (balance, total_earned, total_spent, rewards) = {
                let st = state.borrow();
                (
                    st.dopamine.balance,
                    st.dopamine.total_earned,
                    st.dopamine.total_spent,
                    st.dopamine.rewards.clone(),
                )
            };

            // Hero Balance Card
            let hero_card = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
            hero_card.add_css_class("card");
            hero_card.add_css_class("hero-card");
            hero_card.set_margin_bottom(8);

            let balance_label = gtk4::Label::builder()
                .label(&format!("{} {}", balance, if is_no { "poeng" } else { "points" }))
                .css_classes(["title-1", "accent"])
                .halign(gtk4::Align::Center)
                .build();
            hero_card.append(&balance_label);

            let stats_subtitle = gtk4::Label::builder()
                .label(&format!(
                    "{} {} p  •  {} {} p",
                    if is_no { "Totalt opptjent:" } else { "Total earned:" },
                    total_earned,
                    if is_no { "Innløst:" } else { "Redeemed:" },
                    total_spent
                ))
                .css_classes(["dim-label", "caption"])
                .halign(gtk4::Align::Center)
                .build();
            hero_card.append(&stats_subtitle);

            let motto_label = gtk4::Label::builder()
                .label(if is_no {
                    "Tjen 1 poeng per 5 minutter fokus, +1 for mikro-start og +1 for hver parkert impuls."
                } else {
                    "Earn 1 point per 5 minutes of focus, +1 for micro-start and +1 for each parked impulse."
                })
                .css_classes(["dim-label", "caption"])
                .halign(gtk4::Align::Center)
                .wrap(true)
                .justify(gtk4::Justification::Center)
                .margin_top(4)
                .build();
            hero_card.append(&motto_label);
            content_box_clone.append(&hero_card);

            // Group rewards by category
            let categories = [
                crate::dopamine::RewardCategory::Quick,
                crate::dopamine::RewardCategory::Break,
                crate::dopamine::RewardCategory::Deep,
                crate::dopamine::RewardCategory::Custom,
            ];

            for cat in categories {
                let items_in_cat: Vec<_> = rewards.iter().filter(|r| r.category == cat).cloned().collect();
                if items_in_cat.is_empty() {
                    continue;
                }

                let group = PreferencesGroup::builder()
                    .title(cat.display_name(is_no))
                    .build();

                for item in items_in_cat {
                    let cost_text = format!("{} {}", item.cost, if is_no { "poeng" } else { "points" });
                    let row = ActionRow::builder()
                        .title(&item.title)
                        .subtitle(&cost_text)
                        .build();

                    let cost_badge = gtk4::Label::builder()
                        .label(&format!("{} p", item.cost))
                        .css_classes(["badge-idle"])
                        .valign(gtk4::Align::Center)
                        .build();
                    row.add_suffix(&cost_badge);

                    let redeem_btn = gtk4::Button::builder()
                        .label(if is_no { "Løs inn" } else { "Redeem" })
                        .css_classes(["pill", "suggested-action"])
                        .valign(gtk4::Align::Center)
                        .sensitive(balance >= item.cost)
                        .build();

                    let state_for_redeem = state.clone();
                    let dop_btn_for_redeem = dopamine_btn.clone();
                    let stat_dop_for_redeem = stat_dopamine_val.clone();
                    let item_id = item.id.clone();
                    let item_title = item.title.clone();
                    let item_cost = item.cost;
                    let render_cell_redeem = render_cell_clone.clone();

                    redeem_btn.connect_clicked(move |_| {
                        let (success, new_balance) = {
                            let mut st = state_for_redeem.borrow_mut();
                            if let Ok(_) = st.dopamine.redeem(&item_id) {
                                let _ = crate::dopamine::save_dopamine_account(&st.dopamine);
                                (true, st.dopamine.balance)
                            } else {
                                (false, st.dopamine.balance)
                            }
                        };

                        if success {
                            dop_btn_for_redeem.set_label(&format!("{} p", new_balance));
                            stat_dop_for_redeem.set_label(&format!("{} p", new_balance));

                            let notify_title = if is_no { "Belønning innløst!" } else { "Reward redeemed!" };
                            let notify_body = if is_no {
                                format!("Nyt: {} ({} poeng). Vel fortjent!", item_title, item_cost)
                            } else {
                                format!("Enjoy: {} ({} points). Well deserved!", item_title, item_cost)
                            };
                            crate::flowmodoro::notify(notify_title, &notify_body);

                            if let Some(ref cb) = *render_cell_redeem.borrow() {
                                cb();
                            }
                        }
                    });

                    row.add_suffix(&redeem_btn);

                    if item.is_custom {
                        let del_btn = gtk4::Button::builder()
                            .icon_name("user-trash-symbolic")
                            .css_classes(["flat", "circular"])
                            .valign(gtk4::Align::Center)
                            .tooltip_text(if is_no { "Slett belønning" } else { "Delete reward" })
                            .build();

                        let state_for_del = state.clone();
                        let custom_id = item.id.clone();
                        let render_cell_del = render_cell_clone.clone();

                        del_btn.connect_clicked(move |_| {
                            {
                                let mut st = state_for_del.borrow_mut();
                                st.dopamine.remove_custom_reward(&custom_id);
                                let _ = crate::dopamine::save_dopamine_account(&st.dopamine);
                            }
                            if let Some(ref cb) = *render_cell_del.borrow() {
                                cb();
                            }
                        });

                        row.add_suffix(&del_btn);
                    }

                    group.add(&row);
                }

                content_box_clone.append(&group);
            }

            // Group: Add Custom Reward
            let add_group = PreferencesGroup::builder()
                .title(if is_no { "Legg til egen belønning" } else { "Add Custom Reward" })
                .build();

            let title_entry = EntryRow::builder()
                .title(if is_no { "Tittel" } else { "Title" })
                .build();
            add_group.add(&title_entry);

            let cost_spin = SpinRow::builder()
                .title(if is_no { "Kostnad (poeng)" } else { "Cost (points)" })
                .adjustment(&gtk4::Adjustment::new(10.0, 1.0, 500.0, 1.0, 5.0, 0.0))
                .build();
            add_group.add(&cost_spin);

            let add_action_row = ActionRow::builder()
                .title(if is_no { "Opprett belønning" } else { "Create Reward" })
                .build();

            let create_btn = gtk4::Button::builder()
                .label(if is_no { "Legg til" } else { "Add" })
                .css_classes(["pill", "suggested-action"])
                .valign(gtk4::Align::Center)
                .build();

            let state_for_add = state.clone();
            let render_cell_add = render_cell_clone.clone();
            let title_entry_clone = title_entry.clone();
            let cost_spin_clone = cost_spin.clone();

            create_btn.connect_clicked(move |_| {
                let text = title_entry_clone.text().to_string();
                if text.trim().is_empty() {
                    return;
                }
                let cost = cost_spin_clone.value() as u32;
                {
                    let mut st = state_for_add.borrow_mut();
                    st.dopamine.add_custom_reward(text, cost, crate::dopamine::RewardCategory::Custom);
                    let _ = crate::dopamine::save_dopamine_account(&st.dopamine);
                }
                if let Some(ref cb) = *render_cell_add.borrow() {
                    cb();
                }
            });

            add_action_row.add_suffix(&create_btn);
            add_group.add(&add_action_row);
            content_box_clone.append(&add_group);
        }));

        if let Some(ref cb) = *render_cell.borrow() {
            cb();
        }

        clamp.set_child(Some(&*content_box_rc));
        scrolled.set_child(Some(&clamp));
        toolbar_view.set_content(Some(&scrolled));
        dialog.set_child(Some(&toolbar_view));
        dialog.present();
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
        if storage::take_shutdown_request() {
            self.set_mini_hud_mode(false);
            self.window.set_visible(true);
            self.window.present();
            self.open_shutdown_ritual();
        }

        let now = Instant::now();

        // Check for state updates
        {
            let mut st = self.state.borrow_mut();
            let dt = now.duration_since(st.last_update);
            st.last_update = now;
            st.flow.tick(dt);

            let old = st.previous_state;
            let cur = st.flow.state;

            // Time-Blindness Ambient Cues & Hard Stop Warnings
            let is_focusing = cur == FlowState::Focus || cur == FlowState::MicroStart;
            let st_ref = &mut *st;
            let time_events = st_ref.time_anchor.check_tick(
                is_focusing,
                st_ref.settings.time_cue_interval_mins,
                st_ref.settings.hard_stop_enabled,
                &st_ref.settings.hard_stop_time,
                &st_ref.settings.hard_stop_label,
            );
            let is_no = st.settings.language.resolve() == Language::Norwegian;
            let notify_on = st.settings.notifications_enabled;
            let cue_sound_on = st.settings.time_cue_sound;
            for ev in time_events {
                match ev {
                    crate::time_anchor::TimeAnchorEvent::AmbientTimeCue { clock_time } => {
                        if cue_sound_on {
                            st.audio.play_time_cue();
                        }
                        let _ = clock_time;
                    }
                    crate::time_anchor::TimeAnchorEvent::HardStopWarning15m { time_str, label } => {
                        if cue_sound_on {
                            st.audio.play_time_cue();
                        }
                        if notify_on {
                            let lbl = if label.is_empty() {
                                if is_no { "Hard Stop".to_string() } else { "Hard Stop".to_string() }
                            } else {
                                label
                            };
                            let (title, body) = if is_no {
                                (
                                    "15 min til Hard Stop",
                                    format!("15 min til {} (kl. {}) – på tide å runde av og ta en RAM-dump.", lbl, time_str),
                                )
                            } else {
                                (
                                    "15 min to Hard Stop",
                                    format!("15 min until {} ({}) – time to wrap up and save a RAM-dump.", lbl, time_str),
                                )
                            };
                            crate::flowmodoro::notify(title, &body);
                        }
                    }
                    crate::time_anchor::TimeAnchorEvent::HardStopWarning5m { time_str, label } => {
                        if cue_sound_on {
                            st.audio.play_focus_chime();
                        }
                        if notify_on {
                            let lbl = if label.is_empty() { "Hard Stop".to_string() } else { label };
                            let (title, body) = if is_no {
                                (
                                    "5 min til Hard Stop!",
                                    format!("5 min til {} (kl. {})! Ta en RAM-dump nå.", lbl, time_str),
                                )
                            } else {
                                (
                                    "5 min to Hard Stop!",
                                    format!("5 min until {} ({})! Save a RAM-dump now.", lbl, time_str),
                                )
                            };
                            crate::flowmodoro::notify(title, &body);
                        }
                    }
                    crate::time_anchor::TimeAnchorEvent::HardStopReached { time_str, label } => {
                        if cue_sound_on {
                            st.audio.play_break_bong();
                        }
                        if notify_on {
                            let lbl = if label.is_empty() { "Hard Stop".to_string() } else { label };
                            let (title, body) = if is_no {
                                (
                                    "Hard Stop nådd!",
                                    format!("Klokken er {} ({}) – på tide å avslutte!", time_str, lbl),
                                )
                            } else {
                                (
                                    "Hard Stop Reached!",
                                    format!("It is {} ({}) – time to stop!", time_str, lbl),
                                )
                            };
                            crate::flowmodoro::notify(title, &body);
                        }
                    }
                }
            }

            // Kontinuerlig opptjening av dopaminpoeng under aktiv fokus (1p per 5 min)
            if cur == FlowState::Focus {
                let earned = st.dopamine.record_focus_millis(
                    dt.as_millis() as u64,
                    crate::dopamine::DEFAULT_MILLIS_PER_DOPAMINE_POINT,
                );
                if earned > 0 {
                    let _ = crate::dopamine::save_dopamine_account(&st.dopamine);
                }
            }

            if old != cur {
                st.previous_state = cur;
                st.audio.handle_flow_state_transition(old, cur);
                if old == FlowState::Focus && cur != FlowState::Focus {
                    st.speedbump.reset_snooze();
                }

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
                    st.dopamine.earn_micro_start_bonus();
                    let _ = crate::dopamine::save_dopamine_account(&st.dopamine);
                    let _ = storage::save_user_stats(&st.stats);
                }

                // Lagre dopaminkonto ved overgang fra fokus til pause/idle
                if old == FlowState::Focus {
                    let _ = crate::dopamine::save_dopamine_account(&st.dopamine);
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

            // Sync universal window/file context & check anchor periodically (every 1s)
            if now.duration_since(st.last_anchor_check) >= Duration::from_secs(1) {
                crate::editor::sync_universal_context_anchor();
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

            // Speedbump check during active focus
            if st.flow.state == FlowState::Focus && !st.speedbump_active_prompt && st.settings.speedbump_enabled {
                if let Some(win) = crate::speedbump::get_active_window() {
                    let trigger_opt = {
                        let AppState { ref mut speedbump, ref settings, .. } = *st;
                        speedbump.should_trigger(
                            &win,
                            settings.speedbump_enabled,
                            settings.speedbump_cooldown_secs,
                            &settings.speedbump_triggers,
                        )
                    };
                    if let Some(trigger) = trigger_opt {
                        st.speedbump_active_prompt = true;
                        drop(st);
                        self.prompt_speedbump(win, trigger);
                        return;
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
                            let full_h = self.natural_full_height();
                            let (clamped_x, clamped_y) = storage::clamp_full_window_pos(cur_x, cur_y, FULL_WINDOW_WIDTH, full_h, &locked_mon);
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
                        Language::Norwegian => format!("Bare 5 min for å bryte dørstokkmila ({} gjenstår). Null skyldfølelse!", rem),
                        _ => format!("Just 5 mins to melt friction ({} left). Zero guilt!", rem),
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
                    Language::Norwegian => "Avbryt (null skyldfølelse)",
                    _ => "Cancel (Zero Guilt)",
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
            self.flow_sound_row.set_subtitle(match lang {
                Language::Norwegian => "Spilles automatisk av under fokus",
                _ => "Plays automatically during focus",
            });
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
                match lang { Language::Norwegian => "Tilkoblet", _ => "Connected" }
            } else {
                match lang { Language::Norwegian => "Frakoblet", _ => "Offline" }
            };
            if is_running {
                self.editor_btn.set_label(&format!("● {}", badge));
            } else {
                self.editor_btn.set_label(&format!("○ {}", badge));
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
            let title_raw = if anchor.branch.trim().is_empty() {
                anchor.project_name.clone()
            } else {
                format!("{} [{}]", anchor.project_name, anchor.branch)
            };
            self.anchor_row.set_title(&glib::markup_escape_text(&title_raw));
            let note = if anchor.last_note.is_empty() {
                String::new()
            } else {
                format!("  •  \"{}\"", anchor.last_note)
            };
            let loc = if anchor.line > 1 {
                format!("{}:{}", anchor.relative_path, anchor.line)
            } else {
                anchor.relative_path.clone()
            };
            self.anchor_row.set_subtitle(&glib::markup_escape_text(&format!("{}{}", loc, note)));
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
                    Language::Norwegian => format!("Åpne et dokument, notat eller prosjekt i {} for å forankre flyten", editor.display_name()),
                    _ => format!("Open a document, note, or project in {} to anchor flow", editor.display_name()),
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
        self.stat_dopamine_val.set_label(&format!("{} p", st.dopamine.balance));
        self.dopamine_btn.set_label(&format!("{} p", st.dopamine.balance));

        if let Some((x, y)) = st.settings.custom_pos_mini {
            self.pos_row.set_subtitle(&format!("Lagret Mini-HUD posisjon: X: {}, Y: {}", x, y));
        } else {
            self.pos_row.set_subtitle("Standard posisjon (sentrert / følger skjerm)");
        }

        if let Some(tray) = &self.tray {
            tray.update_status(&timer_text, tag_text);
        }

        // Visual Time-Blindness Pulse & Hard Stop countdown
        if st.time_anchor.pulse_ticks_remaining > 0 {
            self.hero_card.add_css_class("time-cue-pulse");
            self.mini_hud_box.add_css_class("time-cue-pulse");
        } else {
            self.hero_card.remove_css_class("time-cue-pulse");
            self.mini_hud_box.remove_css_class("time-cue-pulse");
        }
        self.update_hard_stop_ui(&st);

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
            self.flow_sound_row.set_subtitle("Spilles automatisk av under fokus");
        }
    }

    pub fn apply_last_ramdump_row(
        row: &ActionRow,
        last_dump: Option<&(String, String, String, String)>,
        is_no: bool,
    ) {
        if let Some((time, now, status, next)) = last_dump {
            row.set_title(&glib::markup_escape_text(&format!("Dump kl. {}", time)));
            row.set_subtitle(&glib::markup_escape_text(&format!(
                "NÅ: {} | STATUS: {} | NESTE: {}",
                now, status, next
            )));
        } else {
            row.set_title(if is_no { "Ingen dump lagret ennå" } else { "No dump saved yet" });
            row.set_subtitle(if is_no {
                "Fyll ut feltene ovenfor og trykk Lagre for å sikre arbeidsminnet"
            } else {
                "Fill in the fields above and click Save to secure working memory"
            });
        }
    }

    pub fn save_ramdump(&self) {
        let now_val = self.ram_now_entry.text().to_string();
        let status_val = self.ram_status_entry.text().to_string();
        let next_val = self.ram_next_entry.text().to_string();
        if now_val.trim().is_empty() && status_val.trim().is_empty() && next_val.trim().is_empty() {
            return;
        }

        let is_no = {
            let mut st = self.state.borrow_mut();
            let is_no = st.settings.language.resolve() == Language::Norwegian;
            if let Some(idx) = st.editing_ramdump_idx.take() {
                let _ = storage::update_ramdump_at(idx, &now_val, &status_val, &next_val);
                st.ramdump.last_dump = storage::load_last_ramdump();
            } else {
                st.ramdump.now = now_val;
                st.ramdump.status = status_val;
                st.ramdump.next_step = next_val;
                let _ = st.ramdump.save();
            }
            is_no
        };

        self.ram_now_entry.set_text("");
        self.ram_status_entry.set_text("");
        self.ram_next_entry.set_text("");
        self.save_ram_btn.set_label(if is_no { "Lagre arbeidsminne" } else { "Save Working Memory" });
        self.cancel_edit_ram_btn.set_visible(false);

        self.refresh_last_ramdump_row();
        self.refresh_ramdumps_list();
    }

    pub fn cancel_edit_ramdump(&self) {
        let is_no = {
            let mut st = self.state.borrow_mut();
            st.editing_ramdump_idx = None;
            st.settings.language.resolve() == Language::Norwegian
        };
        self.ram_now_entry.set_text("");
        self.ram_status_entry.set_text("");
        self.ram_next_entry.set_text("");
        self.save_ram_btn.set_label(if is_no { "Lagre arbeidsminne" } else { "Save Working Memory" });
        self.cancel_edit_ram_btn.set_visible(false);
    }

    pub fn refresh_last_ramdump_row(&self) {
        let st = self.state.borrow();
        let is_no = st.settings.language.resolve() == Language::Norwegian;
        Self::apply_last_ramdump_row(&self.last_dump_row, st.ramdump.last_dump.as_ref(), is_no);
    }

    pub fn refresh_ramdumps_list(&self) {
        Self::populate_ramdumps_box(
            &self.ramdumps_list_box,
            &self.state,
            &self.ram_now_entry,
            &self.ram_status_entry,
            &self.ram_next_entry,
            &self.save_ram_btn,
            &self.cancel_edit_ram_btn,
            &self.last_dump_row,
        );
    }

    pub fn populate_ramdumps_box(
        list_box: &gtk4::ListBox,
        state: &Rc<RefCell<AppState>>,
        now_entry: &EntryRow,
        status_entry: &EntryRow,
        next_entry: &EntryRow,
        save_btn: &gtk4::Button,
        cancel_btn: &gtk4::Button,
        last_dump_row: &ActionRow,
    ) {
        let is_no = state.borrow().settings.language.resolve() == Language::Norwegian;
        let items = storage::load_all_ramdumps();

        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        if items.is_empty() {
            list_box.set_visible(false);
            return;
        }
        list_box.set_visible(true);

        for (original_idx, item) in items.iter().enumerate().rev().take(20) {
            let title_text = format!("{}  —  {}", item.timestamp, item.now);
            let sub_text = format!("Status: {}  •  Neste: {}", item.status, item.next_step);
            let row = ActionRow::builder()
                .title(glib::markup_escape_text(&title_text))
                .subtitle(glib::markup_escape_text(&sub_text))
                .build();

            let edit_btn = gtk4::Button::builder()
                .icon_name("document-edit-symbolic")
                .tooltip_text(if is_no { "Rediger RAM-dump" } else { "Edit RAM dump" })
                .css_classes(["flat"])
                .valign(gtk4::Align::Center)
                .build();

            let delete_btn = gtk4::Button::builder()
                .icon_name("user-trash-symbolic")
                .tooltip_text(if is_no { "Slett RAM-dump" } else { "Delete RAM dump" })
                .css_classes(["flat", "error"])
                .valign(gtk4::Align::Center)
                .build();

            {
                let st_edit = state.clone();
                let now_e = now_entry.clone();
                let status_e = status_entry.clone();
                let next_e = next_entry.clone();
                let save_b = save_btn.clone();
                let cancel_b = cancel_btn.clone();
                let item_now = item.now.clone();
                let item_status = item.status.clone();
                let item_next = item.next_step.clone();

                edit_btn.connect_clicked(move |_| {
                    let is_no_inner = {
                        let mut st = st_edit.borrow_mut();
                        st.editing_ramdump_idx = Some(original_idx);
                        st.settings.language.resolve() == Language::Norwegian
                    };
                    now_e.set_text(&item_now);
                    status_e.set_text(&item_status);
                    next_e.set_text(&item_next);
                    save_b.set_label(if is_no_inner { "Oppdater RAM-dump" } else { "Update RAM Dump" });
                    cancel_b.set_visible(true);
                    let _ = now_e.grab_focus();
                });
            }

            {
                let st_del = state.clone();
                let box_ref = list_box.clone();
                let now_e = now_entry.clone();
                let status_e = status_entry.clone();
                let next_e = next_entry.clone();
                let save_b = save_btn.clone();
                let cancel_b = cancel_btn.clone();
                let last_row_ref = last_dump_row.clone();

                delete_btn.connect_clicked(move |_| {
                    let _ = storage::delete_ramdump_at(original_idx);
                    let (last_opt, cleared_edit, is_no_inner) = {
                        let mut st = st_del.borrow_mut();
                        st.ramdump.last_dump = storage::load_last_ramdump();
                        let mut cleared = false;
                        if let Some(cur) = st.editing_ramdump_idx {
                            if cur == original_idx {
                                st.editing_ramdump_idx = None;
                                cleared = true;
                            } else if cur > original_idx {
                                st.editing_ramdump_idx = Some(cur - 1);
                            }
                        }
                        let is_no_inner = st.settings.language.resolve() == Language::Norwegian;
                        (st.ramdump.last_dump.clone(), cleared, is_no_inner)
                    };

                    if cleared_edit {
                        now_e.set_text("");
                        status_e.set_text("");
                        next_e.set_text("");
                        save_b.set_label(if is_no_inner { "Lagre arbeidsminne" } else { "Save Working Memory" });
                        cancel_b.set_visible(false);
                    }

                    Self::apply_last_ramdump_row(&last_row_ref, last_opt.as_ref(), is_no_inner);
                    Self::populate_ramdumps_box(
                        &box_ref,
                        &st_del,
                        &now_e,
                        &status_e,
                        &next_e,
                        &save_b,
                        &cancel_b,
                        &last_row_ref,
                    );
                });
            }

            row.add_suffix(&edit_btn);
            row.add_suffix(&delete_btn);
            list_box.append(&row);
        }
    }

    /// Runs RAM-dump context/AI suggestion on a background worker thread so GTK never freezes.
    fn spawn_ramdump_suggestion<F>(
        btn: &gtk4::Button,
        loading_label: &'static str,
        anchor_opt: Option<storage::ContextAnchor>,
        settings: storage::TetherSettings,
        on_done: F,
    ) where
        F: FnOnce(crate::context_engine::RamDumpSuggestion) + 'static,
    {
        let prev_label = btn.label().unwrap_or_default().to_string();
        btn.set_sensitive(false);
        btn.set_label(loading_label);

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let suggestion = crate::context_engine::suggest_ramdump_fields(anchor_opt.as_ref(), &settings);
            let _ = tx.send(suggestion);
        });

        let btn_weak = btn.downgrade();
        let mut on_done_opt = Some(on_done);
        glib::timeout_add_local(Duration::from_millis(80), move || {
            match rx.try_recv() {
                Ok(suggestion) => {
                    if let Some(b) = btn_weak.upgrade() {
                        b.set_sensitive(true);
                        b.set_label(&prev_label);
                    }
                    if let Some(cb) = on_done_opt.take() {
                        cb(suggestion);
                    }
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    if let Some(b) = btn_weak.upgrade() {
                        b.set_sensitive(true);
                        b.set_label(&prev_label);
                    }
                    glib::ControlFlow::Break
                }
            }
        });
    }

    pub fn autofill_ramdump(&self) {
        let (anchor_opt, settings) = {
            let st = self.state.borrow();
            (st.anchor.clone(), st.settings.clone())
        };
        let is_no = settings.language.resolve() == Language::Norwegian;
        let now_e = self.ram_now_entry.clone();
        let status_e = self.ram_status_entry.clone();
        let next_e = self.ram_next_entry.clone();
        let last_row = self.last_dump_row.clone();

        Self::spawn_ramdump_suggestion(
            &self.autofill_ram_btn,
            if is_no { "⏳ Analyserer kontekst..." } else { "⏳ Analyzing context..." },
            anchor_opt,
            settings,
            move |suggestion| {
                now_e.set_text(&suggestion.now);
                status_e.set_text(&suggestion.status);
                next_e.set_text(&suggestion.next_step);
                last_row.set_subtitle(&glib::markup_escape_text(&if is_no {
                    format!("Forslag hentet fra {} – se over og trykk Lagre", suggestion.source_label)
                } else {
                    format!("Drafted from {} – review and click Save", suggestion.source_label)
                }));
            },
        );
    }

    pub fn refresh_where_was_i(&self, force_show: bool) {
        let (settings, dismissed) = {
            let st = self.state.borrow();
            (st.settings.clone(), st.where_was_i_dismissed)
        };
        if dismissed && !force_show {
            self.where_was_i_group.set_visible(false);
            return;
        }
        if let Some(card) = crate::context_engine::build_where_was_i(&settings) {
            if !force_show && !card.is_after_long_absence {
                self.where_was_i_group.set_visible(false);
                return;
            }
            let is_no = settings.language.resolve() == Language::Norwegian;
            self.where_was_i_group.set_title(&if is_no {
                format!("Hvor var jeg? ({})", card.absence_text)
            } else {
                format!("Where was I? ({})", card.absence_text)
            });
            self.where_was_i_group.set_description(Some(&card.headline));

            self.where_state_row.set_title(&glib::markup_escape_text(if card.bullet_last_state.is_empty() {
                if is_no { "Ingen tidligere RAM-dump" } else { "No previous RAM-dump" }
            } else {
                &card.bullet_last_state
            }));
            self.where_state_row.set_subtitle(&glib::markup_escape_text(&card.bullet_impulses));
            self.where_git_row.set_title(&glib::markup_escape_text(&card.bullet_git));
            self.where_next_row.set_title(&glib::markup_escape_text(&card.suggested_first_step));
            self.where_was_i_group.set_visible(true);
        } else {
            self.where_was_i_group.set_visible(false);
        }
    }

    pub fn update_hard_stop_ui(&self, st: &AppState) {
        let is_no = st.settings.language.resolve() == Language::Norwegian;
        if st.settings.hard_stop_enabled {
            let now = chrono::Local::now();
            if let Some(secs_left) = crate::time_anchor::seconds_until_target(
                now.hour(),
                now.minute(),
                now.second(),
                &st.settings.hard_stop_time,
            ) {
                let countdown = crate::time_anchor::format_hard_stop_countdown(secs_left, is_no);
                let label = st.settings.hard_stop_label.trim();
                let desc = if label.is_empty() {
                    if is_no {
                        format!("Aktivt stopp kl. {} ({})", st.settings.hard_stop_time, countdown)
                    } else {
                        format!("Active stop at {} ({})", st.settings.hard_stop_time, countdown)
                    }
                } else if is_no {
                    format!("kl. {} ({}) — {}", st.settings.hard_stop_time, label, countdown)
                } else {
                    format!("at {} ({}) — {}", st.settings.hard_stop_time, label, countdown)
                };
                self.hard_stop_row.set_subtitle(&desc);
            }
            self.hard_stop_toggle_btn.set_label(if is_no { "Avbryt" } else { "Clear" });
            self.hard_stop_toggle_btn.set_css_classes(&["pill", "destructive-action"]);
        } else {
            self.hard_stop_row.set_subtitle(if is_no {
                "Ingen Hard Stop satt (varsler 15m og 5m før)"
            } else {
                "No Hard Stop set (alerts 15m and 5m before)"
            });
            self.hard_stop_toggle_btn.set_label(if is_no { "Aktiver" } else { "Set" });
            self.hard_stop_toggle_btn.set_css_classes(&["pill"]);
        }
    }

    pub fn toggle_hard_stop(&self) {
        let raw_time = self.hard_stop_entry.text().trim().to_string();
        let raw_label = self.hard_stop_label_entry.text().trim().to_string();
        let mut st = self.state.borrow_mut();
        let is_no = st.settings.language.resolve() == Language::Norwegian;

        let should_clear = st.settings.hard_stop_enabled
            && (raw_time.is_empty() || raw_time == st.settings.hard_stop_time);

        if should_clear {
            st.settings.hard_stop_enabled = false;
            st.time_anchor.warned_15m_key = None;
            st.time_anchor.warned_5m_key = None;
            st.time_anchor.warned_0m_key = None;
            let _ = storage::save_settings(&st.settings);
            self.update_hard_stop_ui(&st);
            return;
        }

        if let Some((h, m)) = crate::time_anchor::parse_hh_mm(&raw_time) {
            let normalized = format!("{:02}:{:02}", h, m);
            self.hard_stop_entry.set_text(&normalized);
            st.settings.hard_stop_time = normalized;
            st.settings.hard_stop_label = raw_label;
            st.settings.hard_stop_enabled = true;
            st.time_anchor.warned_15m_key = None;
            st.time_anchor.warned_5m_key = None;
            st.time_anchor.warned_0m_key = None;
            let _ = storage::save_settings(&st.settings);
            self.update_hard_stop_ui(&st);
        } else {
            self.hard_stop_row.set_subtitle(if is_no {
                "Ugyldig klokkeslett – bruk formatet TT:MM (f.eks. 14:30)"
            } else {
                "Invalid time – use HH:MM format (e.g. 14:30)"
            });
        }
    }

    pub fn open_shutdown_ritual(&self) {
        let (is_no, anchor_opt, settings, already_open) = {
            let mut st = self.state.borrow_mut();
            let open = st.shutdown_window_open;
            if !open {
                st.shutdown_window_open = true;
            }
            (
                st.settings.language.resolve() == Language::Norwegian,
                st.anchor.clone(),
                st.settings.clone(),
                open,
            )
        };
        if already_open {
            return;
        }

        let dialog = gtk4::Window::builder()
            .title(if is_no { "Dagsavslutning" } else { "Shutdown Ritual" })
            .default_width(560)
            .default_height(640)
            .modal(false)
            .transient_for(&self.window)
            .build();

        {
            let state_close = self.state.clone();
            dialog.connect_close_request(move |_| {
                state_close.borrow_mut().shutdown_window_open = false;
                glib::Propagation::Proceed
            });
        }

        let toolbar_view = ToolbarView::new();
        let header_bar = HeaderBar::new();
        let title_widget = WindowTitle::new(
            if is_no { "Dagsavslutning" } else { "Shutdown Ritual" },
            if is_no { "Tøm hodet og parker arbeidsminnet" } else { "Clear working memory for the day" },
        );
        header_bar.set_title_widget(Some(&title_widget));
        toolbar_view.add_top_bar(&header_bar);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .hexpand(true)
            .build();

        let clamp = Clamp::builder()
            .maximum_size(520)
            .tightening_threshold(460)
            .build();

        let content_box = Rc::new(gtk4::Box::new(gtk4::Orientation::Vertical, 16));
        content_box.set_margin_top(16);
        content_box.set_margin_bottom(24);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);

        // Step 1: Impulse Triage
        let imp_group = PreferencesGroup::builder()
            .title(if is_no { "1. Tøm dagens impulser" } else { "1. Clear today's impulses" })
            .description(if is_no {
                "Huk av ferdige punkter, slett uviktige, og la resten ligge trygt til i morgen"
            } else {
                "Check off completed items, delete noise, leave the rest safely for tomorrow"
            })
            .build();
        let imp_list = Rc::new(
            gtk4::ListBox::builder()
                .selection_mode(gtk4::SelectionMode::None)
                .css_classes(["boxed-list"])
                .build(),
        );

        type RenderCb = Rc<RefCell<Option<Box<dyn Fn()>>>>;
        let render_imp_cell: RenderCb = Rc::new(RefCell::new(None));
        let render_imp_weak = render_imp_cell.clone();
        let state_imp = self.state.clone();
        let imp_list_clone = imp_list.clone();
        let main_imp_box = self.impulses_list_box.clone();

        *render_imp_cell.borrow_mut() = Some(Box::new(move || {
            while let Some(child) = imp_list_clone.first_child() {
                imp_list_clone.remove(&child);
            }
            let st_settings = state_imp.borrow().settings.clone();
            let items = storage::load_structured_impulses_with_cleanup(&st_settings);
            if items.is_empty() {
                let empty_row = ActionRow::builder()
                    .title(if is_no { "Ingen parkerte impulser" } else { "No parked impulses" })
                    .subtitle(if is_no { "Hodet er helt ryddet" } else { "Inbox is clean" })
                    .build();
                imp_list_clone.append(&empty_row);
                return;
            }

            for (idx, item) in items.iter().enumerate().take(25) {
                let row = ActionRow::builder().build();
                if item.done {
                    row.set_title(&format!(
                        "<span strikethrough=\"true\" alpha=\"60%\">{}</span>",
                        glib::markup_escape_text(&item.text)
                    ));
                } else {
                    row.set_title(&glib::markup_escape_text(&item.text));
                }

                let check = gtk4::CheckButton::builder()
                    .active(item.done)
                    .valign(gtk4::Align::Center)
                    .build();
                let st_toggle = state_imp.clone();
                let r_cb_toggle = render_imp_weak.clone();
                let main_box_toggle = main_imp_box.clone();
                check.connect_toggled(move |btn| {
                    let s = st_toggle.borrow().settings.clone();
                    let _ = storage::toggle_impulse_done(&s, idx, btn.is_active());
                    Self::populate_impulses_box(&main_box_toggle, &st_toggle);
                    if let Some(ref cb) = *r_cb_toggle.borrow() {
                        cb();
                    }
                });
                row.add_prefix(&check);

                let del_btn = gtk4::Button::builder()
                    .icon_name("user-trash-symbolic")
                    .css_classes(["flat", "circular"])
                    .valign(gtk4::Align::Center)
                    .tooltip_text(if is_no { "Slett impuls" } else { "Delete impulse" })
                    .build();
                let st_del = state_imp.clone();
                let r_cb_del = render_imp_weak.clone();
                let main_box_del = main_imp_box.clone();
                del_btn.connect_clicked(move |_| {
                    let s = st_del.borrow().settings.clone();
                    let mut all = storage::load_raw_structured_impulses(&s);
                    if idx < all.len() {
                        all.remove(idx);
                        let _ = storage::save_structured_impulses(&s, &all);
                    }
                    Self::populate_impulses_box(&main_box_del, &st_del);
                    if let Some(ref cb) = *r_cb_del.borrow() {
                        cb();
                    }
                });
                row.add_suffix(&del_btn);
                imp_list_clone.append(&row);
            }
        }));
        if let Some(ref cb) = *render_imp_cell.borrow() {
            cb();
        }
        imp_group.add(&*imp_list);
        content_box.append(&imp_group);

        // Step 2: Evening RAM-dump for tomorrow
        let evening_group = PreferencesGroup::builder()
            .title(if is_no { "2. Kvelds-RAM-dump for morgendagens deg" } else { "2. Evening RAM-dump for tomorrow's you" })
            .description(if is_no {
                "Én setning om hva du jobbet med, og det aller første steget i morgen"
            } else {
                "One sentence on today's work and your very first micro-step tomorrow"
            })
            .build();

        let autofill_row = ActionRow::builder()
            .title(if is_no { "Hent forslag fra aktiv kontekst / AI" } else { "Suggest from active context / AI" })
            .build();
        let shutdown_autofill_btn = gtk4::Button::builder()
            .label(if is_no { "Fyll ut automatisk" } else { "Auto-fill" })
            .css_classes(["pill"])
            .valign(gtk4::Align::Center)
            .build();
        autofill_row.add_suffix(&shutdown_autofill_btn);
        evening_group.add(&autofill_row);

        let shutdown_now_entry = EntryRow::builder()
            .title(if is_no { "[OPPSUMMERING] Hva jobbet du med i dag?" } else { "[SUMMARY] What did you work on today?" })
            .build();
        let shutdown_next_entry = EntryRow::builder()
            .title(if is_no { "[I MORGEN] Det aller første lille steget:" } else { "[TOMORROW] First tiny step:" })
            .build();
        evening_group.add(&shutdown_now_entry);
        evening_group.add(&shutdown_next_entry);

        {
            let now_e = shutdown_now_entry.clone();
            let next_e = shutdown_next_entry.clone();
            let a_opt = anchor_opt.clone();
            let s_clone = settings.clone();
            shutdown_autofill_btn.connect_clicked(move |btn| {
                let loading = if is_no { "⏳ Analyserer..." } else { "⏳ Analyzing..." };
                let now_inner = now_e.clone();
                let next_inner = next_e.clone();
                Self::spawn_ramdump_suggestion(
                    btn,
                    loading,
                    a_opt.clone(),
                    s_clone.clone(),
                    move |sug| {
                        now_inner.set_text(&sug.now);
                        next_inner.set_text(&sug.next_step);
                    },
                );
            });
        }
        content_box.append(&evening_group);

        // Step 3: Park and unplug
        let finish_group = PreferencesGroup::builder()
            .title(if is_no { "3. Koble av" } else { "3. Unplug & Shutdown" })
            .build();
        let finish_btn = gtk4::Button::builder()
            .label(if is_no { "Parkér arbeidsminnet og ta kvelden" } else { "Park working memory & end day" })
            .css_classes(["pill", "suggested-action"])
            .halign(gtk4::Align::Center)
            .margin_top(6)
            .build();
        finish_group.add(&finish_btn);
        content_box.append(&finish_group);

        {
            let state_finish = self.state.clone();
            let content_box_finish = content_box.clone();
            let now_e = shutdown_now_entry.clone();
            let next_e = shutdown_next_entry.clone();
            let dialog_ref = dialog.clone();
            let main_window_ref = self.window.clone();
            let last_dump_row_ref = self.last_dump_row.clone();
            let main_imp_list_ref = self.impulses_list_box.clone();
            let main_ram_list_ref = self.ramdumps_list_box.clone();
            let main_ram_now_ref = self.ram_now_entry.clone();
            let main_ram_status_ref = self.ram_status_entry.clone();
            let main_ram_next_ref = self.ram_next_entry.clone();
            let main_ram_save_ref = self.save_ram_btn.clone();
            let main_ram_cancel_ref = self.cancel_edit_ram_btn.clone();

            finish_btn.connect_clicked(move |_| {
                let summary_now = now_e.text().trim().to_string();
                let tomorrow_step = next_e.text().trim().to_string();

                let (remaining_impulses, last_dump_opt) = {
                    let mut st = state_finish.borrow_mut();

                    // Record any accrued focus minutes before resetting
                    if st.flow.state == FlowState::Focus {
                        let total_mins = st.flow.elapsed_focus_secs / 60;
                        let newly_mins = total_mins.saturating_sub(st.last_recorded_focus_mins);
                        if newly_mins > 0 {
                            st.stats.record_focus_minutes(newly_mins);
                            let _ = storage::save_user_stats(&st.stats);
                        }
                    }

                    st.flow.reset();
                    st.last_recorded_focus_mins = 0;
                    st.previous_state = FlowState::Idle;
                    st.audio.stop();
                    st.audio.play_break_bong();

                    if !summary_now.is_empty() || !tomorrow_step.is_empty() {
                        let _ = storage::append_ramdump(
                            &summary_now,
                            if is_no { "Dagsavslutning (parkert for kvelden)" } else { "Shutdown (parked for the night)" },
                            &tomorrow_step,
                        );
                        st.ramdump.last_dump = storage::load_last_ramdump();
                    }

                    let open_count = storage::load_structured_impulses_with_cleanup(&st.settings)
                        .iter()
                        .filter(|i| !i.done)
                        .count();

                    let rec = storage::ShutdownRecord {
                        timestamp: chrono::Utc::now().timestamp(),
                        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                        tomorrow_first_step: tomorrow_step.clone(),
                        summary_now: summary_now.clone(),
                        remaining_impulses: open_count,
                    };
                    let _ = storage::save_shutdown_record(&rec);

                    (open_count, st.ramdump.last_dump.clone())
                };

                Self::apply_last_ramdump_row(&last_dump_row_ref, last_dump_opt.as_ref(), is_no);
                Self::populate_ramdumps_box(
                    &main_ram_list_ref,
                    &state_finish,
                    &main_ram_now_ref,
                    &main_ram_status_ref,
                    &main_ram_next_ref,
                    &main_ram_save_ref,
                    &main_ram_cancel_ref,
                    &last_dump_row_ref,
                );
                Self::populate_impulses_box(&main_imp_list_ref, &state_finish);

                // Transition dialog to calm confirmation view
                while let Some(child) = content_box_finish.first_child() {
                    content_box_finish.remove(&child);
                }

                let confirm_card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
                confirm_card.add_css_class("card");
                confirm_card.add_css_class("hero-card");

                let heading = gtk4::Label::builder()
                    .label(if is_no { "Arbeidsminnet er trygt parkert." } else { "Working memory safely parked." })
                    .css_classes(["title-2", "accent"])
                    .halign(gtk4::Align::Center)
                    .wrap(true)
                    .build();
                confirm_card.append(&heading);

                let msg = gtk4::Label::builder()
                    .label(&if is_no {
                        format!(
                            "Fokusøkten og fokuslyden er stoppet, og {} åpne impulser ligger trygt til i morgen. Logg av skjermen og ta kvelden!",
                            remaining_impulses
                        )
                    } else {
                        format!(
                            "Focus session and audio stopped, and {} open impulses are safely parked for tomorrow. Step away and enjoy your evening!",
                            remaining_impulses
                        )
                    })
                    .css_classes(["dim-label"])
                    .halign(gtk4::Align::Center)
                    .wrap(true)
                    .justify(gtk4::Justification::Center)
                    .build();
                confirm_card.append(&msg);

                let btns = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
                btns.set_halign(gtk4::Align::Center);
                btns.set_margin_top(10);

                let close_win_btn = gtk4::Button::builder()
                    .label(if is_no { "Lukk vindu" } else { "Close window" })
                    .css_classes(["pill"])
                    .build();
                let state_close_btn = state_finish.clone();
                let dialog_close_ref = dialog_ref.clone();
                close_win_btn.connect_clicked(move |_| {
                    state_close_btn.borrow_mut().shutdown_window_open = false;
                    dialog_close_ref.close();
                });
                btns.append(&close_win_btn);

                let quit_app_btn = gtk4::Button::builder()
                    .label(if is_no { "Avslutt Tether for i dag" } else { "Quit Tether for today" })
                    .css_classes(["pill", "suggested-action"])
                    .build();
                let main_w = main_window_ref.clone();
                quit_app_btn.connect_clicked(move |_| {
                    if let Some(app) = main_w.application() {
                        app.quit();
                    } else {
                        std::process::exit(0);
                    }
                });
                btns.append(&quit_app_btn);

                confirm_card.append(&btns);
                content_box_finish.append(&confirm_card);
            });
        }

        clamp.set_child(Some(&*content_box));
        scrolled.set_child(Some(&clamp));
        toolbar_view.set_content(Some(&scrolled));
        dialog.set_child(Some(&toolbar_view));
        dialog.present();
    }

    pub fn capture_impulse(&self, text: &str) {
        {
            let mut st = self.state.borrow_mut();
            st.impulse.input_text = text.to_string();
            let settings = st.settings.clone();
            let _ = st.impulse.capture_with_settings(&settings);
            st.stats.record_impulse();
            st.dopamine.earn_impulse_shield_bonus();
            let _ = crate::dopamine::save_dopamine_account(&st.dopamine);
            let _ = storage::save_user_stats(&st.stats);
        }
        self.refresh_impulses_list();
    }

    pub fn refresh_impulses_list(&self) {
        Self::populate_impulses_box(&self.impulses_list_box, &self.state);
    }

    /// Populates a GTK ListBox with tracked impulses and syncs completion checkboxes.
    pub fn populate_impulses_box(list_box: &gtk4::ListBox, state: &Rc<RefCell<AppState>>) {
        let settings = state.borrow().settings.clone();
        let items = storage::load_structured_impulses_with_cleanup(&settings);

        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        if items.is_empty() {
            let row = ActionRow::builder()
                .title("Ingen registrerte impulser ennå")
                .subtitle("Skriv inn noe ovenfor for å parkere det med en gang.")
                .build();
            list_box.append(&row);
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
                        row.set_subtitle("Fullført");
                    }
                } else {
                    row.set_title(&glib::markup_escape_text(&item.text));
                    if let Some(created) = &item.created_at {
                        row.set_subtitle(&format!("Parkert {}", created));
                    }
                }

                let state_clone = state.clone();
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
                list_box.append(&row);
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

    pub fn editor_extension_status(
        resolved: EditorType,
        mgr: crate::editor::NvimPluginManager,
    ) -> (bool, String, bool) {
        match resolved {
            EditorType::Neovim | EditorType::Neovide => {
                let (installed, sub) = crate::editor::check_nvim_plugin_status(mgr);
                (installed, sub, true)
            }
            EditorType::Obsidian => {
                let (installed, sub) = crate::editor::check_obsidian_plugin_status();
                (installed, sub, true)
            }
            EditorType::VSCode | EditorType::VSCodium | EditorType::Cursor => {
                let (installed, sub) = crate::editor::check_vscode_extension_status(resolved);
                (installed, sub, true)
            }
            EditorType::None => (false, "Ingen applikasjon valgt".to_string(), false),
            _ => (
                true,
                "Automatisk vindu- og filsporing (ingen plugin nødvendig)".to_string(),
                false,
            ),
        }
    }

    pub fn apply_ext_row_status(
        row: &ActionRow,
        btn: &gtk4::Button,
        nvim_combo: Option<&ComboRow>,
        resolved: EditorType,
        mgr: crate::editor::NvimPluginManager,
    ) {
        if let Some(combo) = nvim_combo {
            combo.set_visible(resolved == EditorType::Neovim || resolved == EditorType::Neovide);
        }
        let (installed, sub, has_plugin) = Self::editor_extension_status(resolved, mgr);
        row.set_subtitle(&sub);
        btn.set_visible(has_plugin);
        if has_plugin {
            if installed {
                btn.set_label("Reinstaller");
                btn.remove_css_class("suggested-action");
            } else {
                btn.set_label("Installer utvidelse");
                btn.add_css_class("suggested-action");
            }
        }
    }
}
