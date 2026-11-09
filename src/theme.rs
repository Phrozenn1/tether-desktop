use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "ethereal")]
    Ethereal,
    #[serde(rename = "catppuccin_mocha")]
    CatppuccinMocha,
    #[serde(rename = "tokyo_night")]
    TokyoNight,
    #[serde(rename = "nord")]
    Nord,
    #[serde(rename = "gruvbox_dark")]
    GruvboxDark,
    #[serde(rename = "cyberpunk")]
    Cyberpunk,
    #[serde(rename = "rose_pine")]
    RosePine,
}

impl Default for ThemePreset {
    fn default() -> Self {
        ThemePreset::Auto
    }
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub name: &'static str,
    pub bg_base: Color32,
    pub bg_panel: Color32,
    pub bg_card: Color32,
    pub border_subtle: Color32,
    pub accent: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub focus_green: Color32,
    pub break_red: Color32,
    pub idle_color: Color32,
}

fn parse_hex(hex: &str) -> Option<Color32> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        return Some(Color32::from_rgb(r, g, b));
    }
    None
}

impl ThemeColors {
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Auto => {
                if let Some(colors) = Self::try_load_omarchy_colors() {
                    return colors;
                }
                Self::ethereal()
            }
            ThemePreset::Ethereal => Self::ethereal(),
            ThemePreset::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemePreset::TokyoNight => Self::tokyo_night(),
            ThemePreset::Nord => Self::nord(),
            ThemePreset::GruvboxDark => Self::gruvbox_dark(),
            ThemePreset::Cyberpunk => Self::cyberpunk(),
            ThemePreset::RosePine => Self::rose_pine(),
        }
    }

    fn try_load_omarchy_colors() -> Option<Self> {
        #[cfg(target_os = "linux")]
        {
            if let Some(home) = dirs::home_dir() {
                let path = home.join(".local/state/omarchy/current/theme/colors.toml");
                if let Ok(content) = std::fs::read_to_string(path) {
                    let mut bg = None;
                    let mut bg_panel = None;
                    let mut accent = None;
                    let mut text = None;
                    let mut muted = None;
                    let mut green = None;
                    let mut red = None;

                    for line in content.lines() {
                        let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
                        if parts.len() == 2 {
                            let key = parts[0];
                            let val = parts[1].trim_matches('"');
                            match key {
                                "background" => bg = parse_hex(val),
                                "lighter_background" | "dark_background" => bg_panel = parse_hex(val),
                                "accent" => accent = parse_hex(val),
                                "foreground" => text = parse_hex(val),
                                "muted" | "dark_foreground" => muted = parse_hex(val),
                                "green" => green = parse_hex(val),
                                "red" => red = parse_hex(val),
                                _ => {}
                            }
                        }
                    }

                    if let (Some(b), Some(acc)) = (bg, accent) {
                        let g = green.unwrap_or(Color32::from_rgb(120, 225, 155));
                        let r = red.unwrap_or(Color32::from_rgb(237, 91, 90));
                        let txt = text.unwrap_or(Color32::from_rgb(255, 235, 220));
                        let mut_c = muted.unwrap_or(Color32::from_rgba_premultiplied(109, 125, 182, 120));
                        let panel = bg_panel.unwrap_or(Color32::from_rgb(
                            b.r().saturating_add(6),
                            b.g().saturating_add(8),
                            b.b().saturating_add(16),
                        ));
                        let card = Color32::from_rgb(
                            panel.r().saturating_add(8),
                            panel.g().saturating_add(10),
                            panel.b().saturating_add(20),
                        );

                        return Some(Self {
                            name: "Omarchy Auto",
                            bg_base: b,
                            bg_panel: panel,
                            bg_card: card,
                            border_subtle: Color32::from_rgba_premultiplied(mut_c.r(), mut_c.g(), mut_c.b(), 65),
                            accent: acc,
                            text_primary: txt,
                            text_muted: mut_c,
                            focus_green: g,
                            break_red: r,
                            idle_color: acc,
                        });
                    }
                }
            }
        }
        None
    }

    pub fn ethereal() -> Self {
        Self {
            name: "Ethereal",
            bg_base: Color32::from_rgb(6, 11, 30),
            bg_panel: Color32::from_rgb(10, 16, 40),
            bg_card: Color32::from_rgb(16, 24, 56),
            border_subtle: Color32::from_rgba_premultiplied(109, 125, 182, 60),
            accent: Color32::from_rgb(125, 130, 217),
            text_primary: Color32::from_rgb(255, 235, 220),
            text_muted: Color32::from_rgb(109, 125, 182),
            focus_green: Color32::from_rgb(120, 225, 155),
            break_red: Color32::from_rgb(237, 91, 90),
            idle_color: Color32::from_rgb(125, 130, 217),
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha",
            bg_base: Color32::from_rgb(30, 30, 46),
            bg_panel: Color32::from_rgb(24, 24, 37),
            bg_card: Color32::from_rgb(49, 50, 68),
            border_subtle: Color32::from_rgba_premultiplied(180, 190, 254, 50),
            accent: Color32::from_rgb(203, 166, 247),
            text_primary: Color32::from_rgb(205, 214, 244),
            text_muted: Color32::from_rgb(166, 173, 200),
            focus_green: Color32::from_rgb(166, 227, 161),
            break_red: Color32::from_rgb(243, 139, 168),
            idle_color: Color32::from_rgb(203, 166, 247),
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night",
            bg_base: Color32::from_rgb(26, 27, 38),
            bg_panel: Color32::from_rgb(22, 22, 30),
            bg_card: Color32::from_rgb(36, 40, 59),
            border_subtle: Color32::from_rgba_premultiplied(122, 162, 247, 50),
            accent: Color32::from_rgb(122, 162, 247),
            text_primary: Color32::from_rgb(192, 202, 245),
            text_muted: Color32::from_rgb(86, 95, 137),
            focus_green: Color32::from_rgb(158, 206, 106),
            break_red: Color32::from_rgb(247, 118, 142),
            idle_color: Color32::from_rgb(122, 162, 247),
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord",
            bg_base: Color32::from_rgb(46, 52, 64),
            bg_panel: Color32::from_rgb(36, 41, 51),
            bg_card: Color32::from_rgb(59, 66, 82),
            border_subtle: Color32::from_rgba_premultiplied(136, 192, 208, 60),
            accent: Color32::from_rgb(136, 192, 208),
            text_primary: Color32::from_rgb(236, 239, 244),
            text_muted: Color32::from_rgb(216, 222, 233),
            focus_green: Color32::from_rgb(163, 190, 140),
            break_red: Color32::from_rgb(191, 97, 106),
            idle_color: Color32::from_rgb(129, 161, 193),
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            name: "Gruvbox Dark",
            bg_base: Color32::from_rgb(40, 40, 40),
            bg_panel: Color32::from_rgb(29, 32, 33),
            bg_card: Color32::from_rgb(60, 56, 54),
            border_subtle: Color32::from_rgba_premultiplied(250, 189, 47, 50),
            accent: Color32::from_rgb(250, 189, 47),
            text_primary: Color32::from_rgb(235, 219, 178),
            text_muted: Color32::from_rgb(168, 153, 132),
            focus_green: Color32::from_rgb(184, 187, 38),
            break_red: Color32::from_rgb(251, 73, 52),
            idle_color: Color32::from_rgb(250, 189, 47),
        }
    }

    pub fn cyberpunk() -> Self {
        Self {
            name: "Cyberpunk",
            bg_base: Color32::from_rgb(10, 6, 18),
            bg_panel: Color32::from_rgb(18, 9, 36),
            bg_card: Color32::from_rgb(34, 15, 64),
            border_subtle: Color32::from_rgba_premultiplied(0, 255, 255, 60),
            accent: Color32::from_rgb(0, 255, 255),
            text_primary: Color32::from_rgb(255, 255, 255),
            text_muted: Color32::from_rgb(180, 120, 240),
            focus_green: Color32::from_rgb(0, 255, 136),
            break_red: Color32::from_rgb(255, 0, 85),
            idle_color: Color32::from_rgb(0, 255, 255),
        }
    }

    pub fn rose_pine() -> Self {
        Self {
            name: "Rosé Pine",
            bg_base: Color32::from_rgb(25, 23, 36),
            bg_panel: Color32::from_rgb(31, 29, 46),
            bg_card: Color32::from_rgb(38, 35, 58),
            border_subtle: Color32::from_rgba_premultiplied(235, 188, 186, 50),
            accent: Color32::from_rgb(235, 188, 186),
            text_primary: Color32::from_rgb(224, 222, 244),
            text_muted: Color32::from_rgb(144, 140, 170),
            focus_green: Color32::from_rgb(156, 207, 216),
            break_red: Color32::from_rgb(235, 111, 146),
            idle_color: Color32::from_rgb(196, 167, 231),
        }
    }
}
