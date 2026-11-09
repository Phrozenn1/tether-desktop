use std::cell::RefCell;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem, PredefinedMenuItem},
};

pub struct TetherTray {
    pub tray: TrayIcon,
    pub item_toggle_win: MenuItem,
    pub item_mini_hud: MenuItem,
    pub item_toggle_flow: MenuItem,
    pub item_quit: MenuItem,
    last_status: RefCell<String>,
}

impl TetherTray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();

        let item_toggle_win = MenuItem::new("Vis / Skjul Tether", true, None);
        let item_mini_hud = MenuItem::new("Kompakt Mini-HUD", true, None);
        let sep1 = PredefinedMenuItem::separator();
        let item_toggle_flow = MenuItem::new("Start / Pause Flyt (Space)", true, None);
        let sep2 = PredefinedMenuItem::separator();
        let item_quit = MenuItem::new("Avslutt Tether", true, None);

        let _ = menu.append_items(&[
            &item_toggle_win,
            &item_mini_hud,
            &sep1,
            &item_toggle_flow,
            &sep2,
            &item_quit,
        ]);

        let icon = create_anchor_icon(36, 170, 204); // Default Teal anchor

        let initial_title = "Tether — ADHD Flow & Save-State";
        let tray = TrayIconBuilder::new()
            .with_id("tether")
            .with_title(initial_title)
            .with_tooltip(initial_title)
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build()?;

        Ok(Self {
            tray,
            item_toggle_win,
            item_mini_hud,
            item_toggle_flow,
            item_quit,
            last_status: RefCell::new(initial_title.to_string()),
        })
    }

    pub fn update_status(&self, timer_text: &str, state_label: &str) {
        let text = format!("Tether: {} [{}]", timer_text, state_label);
        let mut last = self.last_status.borrow_mut();
        if *last != text {
            let _ = self.tray.set_tooltip(Some(text.clone()));
            self.tray.set_title(Some(text.clone()));
            *last = text;
        }
    }
}

/// Generates a crisp 32x32 RGBA anchor icon in memory (works on Linux, Windows, macOS)
pub fn create_anchor_icon(r: u8, g: u8, b: u8) -> Icon {
    let width = 32u32;
    let height = 32u32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;

            // Anchor ring at top (center at x=15.5, y=7.5, radius=4.5)
            let ring_dx = x as f32 - 15.5;
            let ring_dy = y as f32 - 7.5;
            let ring_dist = (ring_dx * ring_dx + ring_dy * ring_dy).sqrt();
            let is_ring = ring_dist >= 2.0 && ring_dist <= 5.0;

            // Anchor vertical stock (center line x in [15, 16], y in [7, 24])
            let is_stock = (x == 15 || x == 16) && (y >= 7 && y <= 24);

            // Crossbar (y in [12, 13], x in [9, 22])
            let is_crossbar = (y == 12 || y == 13) && (x >= 9 && x <= 22);

            // Bottom semi-circle arc (center at x=15.5, y=18, radius ~8)
            let arc_dx = x as f32 - 15.5;
            let arc_dy = y as f32 - 18.0;
            let arc_dist = (arc_dx * arc_dx + arc_dy * arc_dy).sqrt();
            let is_arc = arc_dist >= 6.5 && arc_dist <= 9.5 && y >= 18 && y <= 26;

            // Arrow tips at ends of arc
            let is_tips = (y == 20 || y == 21) && (x == 7 || x == 8 || x == 23 || x == 24);

            if is_ring || is_stock || is_crossbar || is_arc || is_tips {
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            } else {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }

    Icon::from_rgba(rgba, width, height).unwrap_or_else(|_| {
        Icon::from_rgba(vec![r, g, b, 255], 1, 1).unwrap()
    })
}
