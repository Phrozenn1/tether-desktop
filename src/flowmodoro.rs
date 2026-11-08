use std::time::Duration;
use crate::storage::{self, SharedFlowState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    Idle,
    Focus,
    Break,
}

pub struct Flowmodoro {
    pub state: FlowState,
    pub focus_ratio: u32,
    pub break_ratio: u32,
    pub elapsed_focus_secs: u64,
    pub remaining_break_secs: u64,
    pub initial_break_secs: u64,
    pub total_focus_secs: u64,
    pub start_epoch: u64,
    pub break_duration_sec: u64,
    pub seq: u64,
    pub last_synced_seq: u64,
    accumulator: Duration,
}

impl Default for Flowmodoro {
    fn default() -> Self {
        Self {
            state: FlowState::Idle,
            focus_ratio: 45,
            break_ratio: 15,
            elapsed_focus_secs: 0,
            remaining_break_secs: 0,
            initial_break_secs: 0,
            total_focus_secs: 0,
            start_epoch: 0,
            break_duration_sec: 0,
            seq: 0,
            last_synced_seq: 0,
            accumulator: Duration::ZERO,
        }
    }
}

fn now_epoch() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}

impl Flowmodoro {
    pub fn new() -> Self {
        let mut f = Self::default();
        // Sjekk om det allerede finnes en aktiv økt startet i Neovim
        f.check_external_sync();
        f
    }

    pub fn is_active(&self) -> bool {
        self.state != FlowState::Idle
    }

    pub fn broadcast_state(&self) {
        let state_str = match self.state {
            FlowState::Idle => "idle",
            FlowState::Focus => "focus",
            FlowState::Break => "break",
        };

        let shared = SharedFlowState {
            state: state_str.to_string(),
            start_epoch: self.start_epoch,
            break_duration_sec: self.break_duration_sec,
            elapsed_focus_sec: self.elapsed_focus_secs,
            remaining_break_sec: self.remaining_break_secs,
            focus_ratio: self.focus_ratio,
            break_ratio: self.break_ratio,
            writer: "desktop".to_string(),
            seq: self.seq,
            timestamp: now_epoch(),
        };

        let _ = storage::save_shared_state(&shared);
    }

    pub fn check_external_sync(&mut self) {
        if let Some(shared) = storage::load_shared_state() {
            if shared.writer == "nvim" && shared.seq != self.last_synced_seq {
                self.last_synced_seq = shared.seq;
                self.seq = shared.seq;
                self.focus_ratio = shared.focus_ratio.max(1);
                self.break_ratio = shared.break_ratio.max(1);

                match shared.state.as_str() {
                    "focus" => {
                        self.state = FlowState::Focus;
                        self.start_epoch = shared.start_epoch;
                        let now = now_epoch();
                        self.elapsed_focus_secs = if now >= self.start_epoch {
                            now - self.start_epoch
                        } else {
                            shared.elapsed_focus_sec
                        };
                    }
                    "break" => {
                        self.state = FlowState::Break;
                        self.start_epoch = shared.start_epoch;
                        self.break_duration_sec = shared.break_duration_sec.max(shared.remaining_break_sec);
                        self.initial_break_secs = self.break_duration_sec;
                        let now = now_epoch();
                        let elapsed = if now >= self.start_epoch {
                            now - self.start_epoch
                        } else {
                            0
                        };
                        self.remaining_break_secs = self.break_duration_sec.saturating_sub(elapsed);
                    }
                    _ => {
                        self.state = FlowState::Idle;
                        self.elapsed_focus_secs = 0;
                        self.remaining_break_secs = 0;
                        self.start_epoch = 0;
                    }
                }
            }
        }
    }

    pub fn start_focus(&mut self) {
        self.state = FlowState::Focus;
        self.elapsed_focus_secs = 0;
        self.accumulator = Duration::ZERO;
        self.start_epoch = now_epoch();
        self.seq += 1;
        self.broadcast_state();

        notify("Fokusøkt startet 🚀", "Jobb uforstyrret så lenge du har god flyt!");
    }

    pub fn calculate_break_time(&self) -> u64 {
        let f_base = self.focus_ratio.max(1) as f64;
        let b_base = self.break_ratio.max(1) as f64;
        let earned = (self.elapsed_focus_secs as f64) * (b_base / f_base);
        (earned as u64).max(30)
    }

    pub fn start_break(&mut self) {
        if self.elapsed_focus_secs < 10 {
            self.reset();
            return;
        }

        let break_secs = self.calculate_break_time();
        let focus_fmt = format_time(self.elapsed_focus_secs);
        let break_fmt = format_time(break_secs);

        self.state = FlowState::Break;
        self.remaining_break_secs = break_secs;
        self.initial_break_secs = break_secs;
        self.break_duration_sec = break_secs;
        self.start_epoch = now_epoch();
        self.accumulator = Duration::ZERO;
        self.seq += 1;
        self.broadcast_state();

        notify(
            "Fokusøkt fullført! ☕",
            &format!(
                "Fokus: {}.\nOpptjent pause (45:15): {}.\nHusk en 60s RAM-dump!",
                focus_fmt, break_fmt
            ),
        );
    }

    pub fn toggle(&mut self) {
        match self.state {
            FlowState::Idle => self.start_focus(),
            FlowState::Focus => self.start_break(),
            FlowState::Break => self.reset(),
        }
    }

    pub fn reset(&mut self) {
        self.state = FlowState::Idle;
        self.elapsed_focus_secs = 0;
        self.remaining_break_secs = 0;
        self.start_epoch = 0;
        self.break_duration_sec = 0;
        self.accumulator = Duration::ZERO;
        self.seq += 1;
        self.broadcast_state();
    }

    pub fn tick(&mut self, dt: Duration) {
        // Sjekk alltid om Neovim har oppdatert status
        self.check_external_sync();

        if self.state == FlowState::Idle {
            return;
        }

        self.accumulator += dt;
        while self.accumulator >= Duration::from_secs(1) {
            self.accumulator -= Duration::from_secs(1);

            match self.state {
                FlowState::Focus => {
                    self.elapsed_focus_secs += 1;
                    self.total_focus_secs += 1;
                }
                FlowState::Break => {
                    if self.remaining_break_secs > 0 {
                        self.remaining_break_secs -= 1;
                    }
                    if self.remaining_break_secs == 0 {
                        self.state = FlowState::Idle;
                        self.broadcast_state();
                        notify(
                            "Pausen er over! 🚀",
                            "Hjernen er uthvilt og klar for neste fokusøkt.",
                        );
                        break;
                    }
                }
                FlowState::Idle => break,
            }
        }
    }

    pub fn current_time_display(&self) -> String {
        match self.state {
            FlowState::Focus => format_time(self.elapsed_focus_secs),
            FlowState::Break => format_time(self.remaining_break_secs),
            FlowState::Idle => "00:00".to_string(),
        }
    }

    pub fn progress_pct(&self) -> f32 {
        match self.state {
            FlowState::Idle => 0.0,
            FlowState::Focus => {
                let target = (self.focus_ratio as f32 * 60.0).max(60.0);
                (self.elapsed_focus_secs as f32 / target).clamp(0.0, 1.0)
            }
            FlowState::Break => {
                if self.initial_break_secs == 0 {
                    0.0
                } else {
                    (self.remaining_break_secs as f32 / self.initial_break_secs as f32).clamp(0.0, 1.0)
                }
            }
        }
    }
}

pub fn format_time(seconds: u64) -> String {
    let hrs = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hrs > 0 {
        format!("{:02}:{:02}:{:02}", hrs, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

fn notify(title: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .appname("Tether")
        .summary(title)
        .body(body)
        .timeout(notify_rust::Timeout::Milliseconds(6000))
        .show();
}
