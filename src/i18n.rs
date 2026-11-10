use crate::storage::Language;

pub struct Strings;

impl Strings {
    pub fn tag_focus(_lang: Language) -> &'static str {
        "FOKUS"
    }

    pub fn tag_micro(_lang: Language) -> &'static str {
        "5 MIN"
    }

    pub fn tag_break(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "PAUSE",
            Language::English => "BREAK",
        }
    }

    pub fn tag_ready(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "KLAR",
            Language::English => "READY",
        }
    }

    pub fn btn_micro_start(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "5m mikrostart",
            Language::English => "5m Micro-start",
        }
    }

    pub fn btn_micro_tooltip(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Start en uforpliktende 5-minutters økt for å bryte oppstartssperren",
            Language::English => "Start a zero-guilt 5-minute session to melt task initiation friction",
        }
    }

    pub fn sec_soundscape(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Fokus-lyd & Støygenerator",
            Language::English => "Focus Sound & Ambient Noise",
        }
    }

    pub fn lbl_soundscape_type(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Lydtype:",
            Language::English => "Sound type:",
        }
    }

    pub fn lbl_soundscape_vol(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Volum:",
            Language::English => "Volume:",
        }
    }

    pub fn chk_soundscape_auto(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Start lyden automatisk ved Fokus og Mikro-start",
            Language::English => "Auto-play sound during Focus and Micro-start",
        }
    }

    pub fn lbl_context_anchor(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Kontekstanker",
            Language::English => "Context Anchor",
        }
    }

    pub fn btn_resume_neovim(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Gjenoppta i Neovim",
            Language::English => "Resume in Neovim",
        }
    }

    pub fn lbl_no_anchor(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Ingen aktiv kodekontekst ennå. Åpne en fil i Neovim for å forankre flyten.",
            Language::English => "No active code context yet. Open a file in Neovim to anchor flow.",
        }
    }

    pub fn lbl_momentum_title(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Momentum",
            Language::English => "Momentum",
        }
    }

    pub fn lbl_weekly_flow(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Ukens flyt",
            Language::English => "Weekly flow",
        }
    }

    pub fn lbl_impulses_shielded(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Parkerte impulser",
            Language::English => "Shielded impulses",
        }
    }

    pub fn lbl_sessions_count(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Fullførte økter",
            Language::English => "Completed sessions",
        }
    }

    pub fn lbl_no_shame_motto(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Ingen tapte streaks her – hvert minutt du legger ned teller.",
            Language::English => "No broken streaks here – every single minute counts.",
        }
    }

    pub fn drag_tooltip(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Dra for å flytte (eller Super + Venstreklikk)",
            Language::English => "Drag to move (or Super + Left click)",
        }
    }

    pub fn time_toggle_tooltip(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Klikk for å veksle fokus/pause (Mellomrom)",
            Language::English => "Click to toggle focus/break (Space)",
        }
    }

    pub fn expand_tooltip(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Åpne/lukk fullt dashboard (Dobbeltklikk virker også)",
            Language::English => "Open/close full dashboard (Double-click also works)",
        }
    }

    pub fn context_menu_open(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Åpne fullt panel",
            Language::English => "Open full dashboard",
        }
    }

    pub fn context_menu_reset(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Nullstill økt",
            Language::English => "Reset session",
        }
    }

    pub fn context_menu_quit(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Avslutt Tether",
            Language::English => "Quit Tether",
        }
    }

    pub fn tab_flowmodoro(_lang: Language) -> &'static str { "Flowmodoro" }
    pub fn tab_ramdump(_lang: Language) -> &'static str { "RAM-dump" }
    pub fn tab_sidequests(_lang: Language) -> &'static str { "Impulser" }
    pub fn tab_yak(_lang: Language) -> &'static str { "Yak-alarm" }
    pub fn tab_settings(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Innstillinger",
            Language::English => "Settings",
        }
    }

    pub fn mini_hud_btn(_lang: Language) -> &'static str { "Mini-HUD" }
    pub fn always_on_top(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Alltid øverst",
            Language::English => "Always on top",
        }
    }

    pub fn focus_title(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "FOKUSØKT (FLYTSONE)",
            Language::English => "FOCUS SESSION (FLOW STATE)",
        }
    }

    pub fn break_title(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "PAUSE (HVILE)",
            Language::English => "BREAK (REST & RECHARGE)",
        }
    }

    pub fn ready_title(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "KLAR TIL NY ØKT",
            Language::English => "READY TO FLOW",
        }
    }

    pub fn sub_focus(lang: Language, earned_str: &str, f: u32, r: u32) -> String {
        match lang {
            Language::Norwegian => format!("Opptjent pause hittil: {} ({}:{} forhold)", earned_str, f, r),
            Language::English => format!("Earned break so far: {} ({}:{} ratio)", earned_str, f, r),
        }
    }

    pub fn sub_break(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Husk å ta en 60s RAM-dump og strekke beina!",
            Language::English => "Remember to take a 60s RAM-dump and stretch your legs!",
        }
    }

    pub fn sub_idle(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Jobb så lenge du har flyt. Pausen beregnes automatisk.",
            Language::English => "Work uninterrupted as long as you have flow. Break is calculated automatically.",
        }
    }

    pub fn btn_start_focus(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Start fokus",
            Language::English => "Start Focus",
        }
    }

    pub fn btn_take_break(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Ta pause",
            Language::English => "Take Break",
        }
    }

    pub fn btn_end_break(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Avslutt pause",
            Language::English => "End Break",
        }
    }

    pub fn btn_reset(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Nullstill",
            Language::English => "Reset",
        }
    }

    pub fn shortcut_hint(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Snarvei: Trykk Mellomrom når som helst for å veksle.",
            Language::English => "Shortcut: Press Space anytime to toggle.",
        }
    }

    pub fn ratio_title(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Forholdstall:",
            Language::English => "Ratio rule:",
        }
    }

    pub fn stats_focus_today(lang: Language, mins: u64) -> String {
        match lang {
            Language::Norwegian => format!("Total fokus i dag: {} minutter", mins),
            Language::English => format!("Total focus today: {} minutes", mins),
        }
    }

    // RAM-dump
    pub fn ramdump_heading(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "60-sekunders RAM-dump",
            Language::English => "60-Second RAM-Dump",
        }
    }

    pub fn ramdump_subtitle(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Frys tilstanden i arbeidsminnet før du forlater pulten, så slipper du oppstartsparalyse etterpå.",
            Language::English => "Freeze working memory state before leaving your desk to eliminate startup paralysis later.",
        }
    }

    pub fn field_now(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "[NÅ] Hva holder du på med akkurat nå?",
            Language::English => "[NOW] What are you working on right now?",
        }
    }

    pub fn field_now_hint(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "f.eks. Skriver ferdig rapport",
            Language::English => "e.g. Finishing project report",
        }
    }

    pub fn field_status(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "[STATUS] Hva mangler / hvor stoppet det?",
            Language::English => "[STATUS] What is left / where did you pause?",
        }
    }

    pub fn field_status_hint(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "f.eks. Må legge til sammendrag",
            Language::English => "e.g. Need to add summary section",
        }
    }

    pub fn field_next(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "[NESTE MEKANISKE STEG] Aller første handling ved retur:",
            Language::English => "[NEXT STEP] Very first micro-action upon return:",
        }
    }

    pub fn field_next_hint(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "f.eks. Åpne kapittel 3",
            Language::English => "e.g. Open chapter 3",
        }
    }

    pub fn btn_save_ramdump(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Lagre tilstand",
            Language::English => "Anchor State",
        }
    }

    pub fn last_dump_heading(lang: Language, time: &str) -> String {
        match lang {
            Language::Norwegian => format!("Forrige forankring (kl. {}):", time),
            Language::English => format!("Previous anchor (at {}):", time),
        }
    }

    pub fn bullet_now(lang: Language, val: &str) -> String {
        match lang {
            Language::Norwegian => format!("• Nå: {}", val),
            Language::English => format!("• Now: {}", val),
        }
    }

    pub fn bullet_status(lang: Language, val: &str) -> String {
        match lang {
            Language::Norwegian => format!("• Status: {}", val),
            Language::English => format!("• Status: {}", val),
        }
    }

    pub fn bullet_next(lang: Language, val: &str) -> String {
        match lang {
            Language::Norwegian => format!("• Neste mekaniske steg: {}", val),
            Language::English => format!("• Next mechanical step: {}", val),
        }
    }

    // Side-Quests
    pub fn sidequests_heading(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Impulsbuffer",
            Language::English => "Impulse Buffer",
        }
    }

    pub fn sidequests_subtitle(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Dukket det opp en tilfeldig tanke? Fang den her slik at du kan holde fokus på kjerneoppgaven.",
            Language::English => "Got a distracting thought? Capture it here to stay anchored to your core mission.",
        }
    }

    pub fn sidequests_input_hint(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Skriv strøtanke her (f.eks. 'Sjekk e-post om flybilletter')...",
            Language::English => "Capture thought here (e.g. 'Check email regarding flights')...",
        }
    }

    pub fn btn_capture(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "+ Fang",
            Language::English => "+ Capture",
        }
    }

    pub fn saved_impulses_heading(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Fangede impulser (venter til etter økten):",
            Language::English => "Captured impulses (waiting for later):",
        }
    }

    pub fn no_impulses(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Ingen lagrede impulser enda. Hodet ditt er rent.",
            Language::English => "No saved impulses yet. Your mind is clear.",
        }
    }

    // Yak-Shaving
    pub fn yak_heading(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Kaninhullsjekk & Yak-shaving",
            Language::English => "Rabbit Hole Check & Yak-Shaving",
        }
    }

    pub fn yak_subtitle(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "En sunn dose realitetssjekk når du merker at du gjør alt annet enn det du egentlig burde gjøre:",
            Language::English => "A healthy reality check when you catch yourself doing everything except what you actually should be doing:",
        }
    }

    pub fn btn_next_jab(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Nytt stikk",
            Language::English => "New Reality Check",
        }
    }

    pub fn btn_back_to_flow(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Tilbake til flyten",
            Language::English => "Return to flow",
        }
    }

    pub fn more_reminders(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Flere påminnelser:",
            Language::English => "More reality checks:",
        }
    }

    pub fn yak_jabs(lang: Language) -> Vec<&'static str> {
        match lang {
            Language::Norwegian => vec![
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
            Language::English => vec![
                "Tweaking your developer tools unfortunately doesn't count as studying or coding.",
                "Sure, your code will definitely run faster once you change the color scheme one more time.",
                "Congratulations! You just spent 40 minutes automating a 3-second task.",
                "The yak you are shaving has been completely naked for over half an hour.",
                "You set out to fix one simple error. Why are you refactoring the entire database layer now?",
                "You are four layers deeper down a rabbit hole than when you started. Breathe and return to the core goal.",
                "This isn't productivity. This is procrastination wearing a tuxedo.",
                "Your brain: 'This is urgent and critical!' Reality: 'You are cleaning keycaps with a Q-tip'.",
                "No, you do not need to write your own web framework before finishing this ticket.",
                "The architecture is plenty fine. Write the actual logic.",
            ],
        }
    }

    // Settings
    pub fn settings_heading(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Innstillinger & integrasjon",
            Language::English => "Settings & Integrations",
        }
    }

    pub fn sec_general(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Generelt & fargetema",
            Language::English => "General & Theme",
        }
    }

    pub fn lbl_language(_lang: Language) -> &'static str {
        "Language / Språk:"
    }

    pub fn lbl_theme(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Fargetema:",
            Language::English => "Theme:",
        }
    }

    pub fn sec_flowmodoro(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Flowmodoro forholdstall",
            Language::English => "Flowmodoro Ratio",
        }
    }

    pub fn sec_sidequests(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Impulsbuffer lagring",
            Language::English => "Impulse Buffer Storage",
        }
    }

    pub fn lbl_filepath(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Filsti:",
            Language::English => "File path:",
        }
    }

    pub fn btn_browse(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Bla gjennom...",
            Language::English => "Browse...",
        }
    }

    pub fn lbl_format(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Filformat:",
            Language::English => "File format:",
        }
    }

    pub fn sec_integration(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Editor og skrivebordsintegrasjon",
            Language::English => "Editor & Desktop Companion",
        }
    }

    pub fn nvim_status_online(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Neovim kjører nå",
            Language::English => "Neovim is running",
        }
    }

    pub fn nvim_status_offline(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Neovim ikke oppdaget",
            Language::English => "Neovim offline",
        }
    }

    pub fn btn_open_nvim(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Gå til Neovim",
            Language::English => "Open / Focus Neovim",
        }
    }

    pub fn lbl_terminal_cmd(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Terminal-kommando for Neovim:",
            Language::English => "Terminal launch command for Neovim:",
        }
    }

    pub fn chk_auto_desktop(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Start automatisk Desktop HUD når Neovim starter",
            Language::English => "Auto-launch Desktop HUD when Neovim starts",
        }
    }

    pub fn sec_monitor(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Skjerm & plassering",
            Language::English => "Display & Placement",
        }
    }

    pub fn lbl_locked_monitor(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Lås vindu til spesifikk skjerm:",
            Language::English => "Lock window to specific display:",
        }
    }

    pub fn opt_auto_monitor(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Auto (Følg aktiv skjerm)",
            Language::English => "Auto (Follow active display)",
        }
    }

    pub fn hint_monitor(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Vinduet forblir forankret i øvre høyre hjørne på den valgte skjermen.",
            Language::English => "The window remains anchored in the top-right corner of the selected display.",
        }
    }

    pub fn btn_move_now(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Flytt til valgt skjerm nå",
            Language::English => "Move to selected display now",
        }
    }

    pub fn lbl_saved_pos(lang: Language, x: i32, y: i32) -> String {
        match lang {
            Language::Norwegian => format!("Husket vindusposisjon: X={}, Y={}", x, y),
            Language::English => format!("Remembered window position: X={}, Y={}", x, y),
        }
    }

    pub fn btn_reset_pos(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Nullstill posisjon til standard hjørne",
            Language::English => "Reset position to default corner",
        }
    }

    pub fn sec_notifications(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Varsler & påminnelser",
            Language::English => "Notifications & Reminders",
        }
    }

    pub fn chk_notifications(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Aktiver skrivebordsvarsler",
            Language::English => "Enable desktop notifications",
        }
    }

    pub fn chk_remind_ramdump(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Minn om 60s RAM-dump når pausen starter",
            Language::English => "Remind 60s RAM-dump when break starts",
        }
    }

    pub fn btn_save(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Lagre innstillinger",
            Language::English => "Save Settings",
        }
    }

    pub fn save_success(lang: Language) -> &'static str {
        match lang {
            Language::Norwegian => "Innstillinger lagret til ~/.config/tether/config.json.",
            Language::English => "Settings saved to ~/.config/tether/config.json.",
        }
    }
}
