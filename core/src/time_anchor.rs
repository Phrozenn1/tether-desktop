use chrono::Timelike;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeAnchorEvent {
    AmbientTimeCue { clock_time: String },
    HardStopWarning15m { time_str: String, label: String },
    HardStopWarning5m { time_str: String, label: String },
    HardStopReached { time_str: String, label: String },
}

#[derive(Debug, Default)]
pub struct TimeAnchorState {
    pub last_cue_hm: Option<(u32, u32)>,
    pub pulse_ticks_remaining: u32,
    pub warned_15m_key: Option<String>,
    pub warned_5m_key: Option<String>,
    pub warned_0m_key: Option<String>,
}

/// Parses `"HH:MM"` or `"H:MM"` into validated `(hour, minute)`.
pub fn parse_hh_mm(s: &str) -> Option<(u32, u32)> {
    let trimmed = s.trim();
    let (h_str, m_str) = trimmed.split_once(':')?;
    if m_str.len() != 2 {
        return None;
    }
    let hour: u32 = h_str.parse().ok()?;
    let min: u32 = m_str.parse().ok()?;
    if hour < 24 && min < 60 {
        Some((hour, min))
    } else {
        None
    }
}

/// Signed seconds from `(now_h, now_m, now_s)` until `target_str` on the same day.
pub fn seconds_until_target(now_h: u32, now_m: u32, now_s: u32, target_str: &str) -> Option<i64> {
    let (th, tm) = parse_hh_mm(target_str)?;
    let target_secs = (th * 3600 + tm * 60) as i64;
    let now_secs = (now_h * 3600 + now_m * 60 + now_s) as i64;
    Some(target_secs - now_secs)
}

/// Signed minutes (rounded up when positive) from `(now_hour, now_min, now_sec)` until `target_str`.
pub fn minutes_until_target(now_hour: u32, now_min: u32, now_sec: u32, target_str: &str) -> Option<i64> {
    let secs = seconds_until_target(now_hour, now_min, now_sec, target_str)?;
    if secs > 0 {
        Some((secs + 59) / 60)
    } else {
        Some(secs / 60)
    }
}

pub fn format_hard_stop_countdown(secs_left: i64, is_no: bool) -> String {
    if secs_left <= 0 {
        return if is_no {
            "Nådd nå".to_string()
        } else {
            "Reached now".to_string()
        };
    }

    let mins_total = (secs_left + 59) / 60;
    let hrs = mins_total / 60;
    let mins = mins_total % 60;

    if hrs > 0 {
        if is_no {
            format!("om {}t {}m", hrs, mins)
        } else {
            format!("in {}h {}m", hrs, mins)
        }
    } else if is_no {
        format!("om {} min", mins_total)
    } else {
        format!("in {} min", mins_total)
    }
}

impl TimeAnchorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_tick(
        &mut self,
        is_focusing: bool,
        interval_mins: u32,
        hard_stop_enabled: bool,
        hard_stop_time: &str,
        hard_stop_label: &str,
    ) -> Vec<TimeAnchorEvent> {
        let now = chrono::Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        self.check_tick_at(
            &today,
            now.hour(),
            now.minute(),
            now.second(),
            is_focusing,
            interval_mins,
            hard_stop_enabled,
            hard_stop_time,
            hard_stop_label,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn check_tick_at(
        &mut self,
        today_date: &str,
        hour: u32,
        min: u32,
        sec: u32,
        is_focusing: bool,
        interval_mins: u32,
        hard_stop_enabled: bool,
        hard_stop_time: &str,
        hard_stop_label: &str,
    ) -> Vec<TimeAnchorEvent> {
        if self.pulse_ticks_remaining > 0 {
            self.pulse_ticks_remaining -= 1;
        }

        let mut events = Vec::new();

        // 1. Ambient Time Cues during active focus
        if is_focusing && interval_mins > 0 {
            let matches_interval = match interval_mins {
                30 => (min == 0 || min == 30) && sec < 3,
                60 => min == 0 && sec < 3,
                _ => false,
            };

            if matches_interval && self.last_cue_hm != Some((hour, min)) {
                self.last_cue_hm = Some((hour, min));
                self.pulse_ticks_remaining = 12;
                events.push(TimeAnchorEvent::AmbientTimeCue {
                    clock_time: format!("{:02}:{:02}", hour, min),
                });
            }
        }

        // 2. Hard Stop / Appointment Anchor warnings
        if hard_stop_enabled {
            if let Some((th, tm)) = parse_hh_mm(hard_stop_time) {
                let secs_left = (th * 3600 + tm * 60) as i64 - (hour * 3600 + min * 60 + sec) as i64;
                let time_str = format!("{:02}:{:02}", th, tm);
                let key = format!("{}-{}", today_date, time_str);
                let label = hard_stop_label.trim().to_string();

                if secs_left <= 15 * 60 && secs_left > 14 * 60 && self.warned_15m_key.as_deref() != Some(&key) {
                    self.warned_15m_key = Some(key.clone());
                    self.pulse_ticks_remaining = 12;
                    events.push(TimeAnchorEvent::HardStopWarning15m {
                        time_str: time_str.clone(),
                        label: label.clone(),
                    });
                }

                if secs_left <= 5 * 60 && secs_left > 4 * 60 && self.warned_5m_key.as_deref() != Some(&key) {
                    self.warned_5m_key = Some(key.clone());
                    self.pulse_ticks_remaining = 16;
                    events.push(TimeAnchorEvent::HardStopWarning5m {
                        time_str: time_str.clone(),
                        label: label.clone(),
                    });
                }

                if secs_left <= 0 && secs_left >= -60 && self.warned_0m_key.as_deref() != Some(&key) {
                    self.warned_0m_key = Some(key);
                    self.pulse_ticks_remaining = 20;
                    events.push(TimeAnchorEvent::HardStopReached {
                        time_str,
                        label,
                    });
                }
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hh_mm() {
        assert_eq!(parse_hh_mm("14:00"), Some((14, 0)));
        assert_eq!(parse_hh_mm("9:30"), Some((9, 30)));
        assert_eq!(parse_hh_mm("23:59"), Some((23, 59)));
        assert_eq!(parse_hh_mm("24:00"), None);
        assert_eq!(parse_hh_mm("14:60"), None);
        assert_eq!(parse_hh_mm("14:0"), None);
        assert_eq!(parse_hh_mm("abc"), None);
    }

    #[test]
    fn test_seconds_and_minutes_until_target() {
        assert_eq!(seconds_until_target(13, 45, 0, "14:00"), Some(900));
        assert_eq!(minutes_until_target(13, 45, 0, "14:00"), Some(15));
        assert_eq!(seconds_until_target(14, 1, 0, "14:00"), Some(-60));
    }

    #[test]
    fn test_ambient_cue_deduplication() {
        let mut state = TimeAnchorState::new();
        let ev1 = state.check_tick_at("2026-11-14", 13, 30, 0, true, 30, false, "", "");
        assert_eq!(
            ev1,
            vec![TimeAnchorEvent::AmbientTimeCue {
                clock_time: "13:30".to_string()
            }]
        );
        assert_eq!(state.pulse_ticks_remaining, 12);

        // Next tick within same minute does not duplicate
        let ev2 = state.check_tick_at("2026-11-14", 13, 30, 1, true, 30, false, "", "");
        assert!(ev2.is_empty());

        // Not focusing -> no cue
        let ev3 = state.check_tick_at("2026-11-14", 14, 0, 0, false, 30, false, "", "");
        assert!(ev3.is_empty());
    }

    #[test]
    fn test_hard_stop_warnings_and_reached() {
        let mut state = TimeAnchorState::new();
        let ev15 = state.check_tick_at("2026-11-14", 13, 45, 0, true, 0, true, "14:00", "Forelesning");
        assert_eq!(
            ev15,
            vec![TimeAnchorEvent::HardStopWarning15m {
                time_str: "14:00".to_string(),
                label: "Forelesning".to_string(),
            }]
        );

        let ev5 = state.check_tick_at("2026-11-14", 13, 55, 0, true, 0, true, "14:00", "Forelesning");
        assert_eq!(
            ev5,
            vec![TimeAnchorEvent::HardStopWarning5m {
                time_str: "14:00".to_string(),
                label: "Forelesning".to_string(),
            }]
        );

        let ev0 = state.check_tick_at("2026-11-14", 14, 0, 0, true, 0, true, "14:00", "Forelesning");
        assert_eq!(
            ev0,
            vec![TimeAnchorEvent::HardStopReached {
                time_str: "14:00".to_string(),
                label: "Forelesning".to_string(),
            }]
        );

        // Repeated check at 14:00:05 does not re-fire
        let ev_repeat = state.check_tick_at("2026-11-14", 14, 0, 5, true, 0, true, "14:00", "Forelesning");
        assert!(ev_repeat.is_empty());
    }
}
