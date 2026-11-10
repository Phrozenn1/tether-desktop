use std::time::Duration;
use crate::storage::{self, SharedFlowState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    Idle,
    MicroStart,
    Focus,
    Break,
}

pub struct Flowmodoro {
    pub state: FlowState,
    pub focus_ratio: u32,
    pub break_ratio: u32,
    pub elapsed_focus_secs: u64,
    pub remaining_break_secs: u64,
    pub micro_remaining_secs: u64,
    pub initial_break_secs: u64,
    pub total_focus_secs: u64,
    pub start_epoch: u64,
    pub break_duration_sec: u64,
    pub banked_break_secs: u64,
    pub focus_at_last_break: u64,
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
            micro_remaining_secs: 300,
            initial_break_secs: 0,
            total_focus_secs: 0,
            start_epoch: 0,
            break_duration_sec: 0,
            banked_break_secs: 0,
            focus_at_last_break: 0,
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
            FlowState::MicroStart => "micro_start",
            FlowState::Focus => "focus",
            FlowState::Break => "break",
        };

        let shared = SharedFlowState {
            state: state_str.to_string(),
            start_epoch: self.start_epoch,
            break_duration_sec: self.break_duration_sec,
            elapsed_focus_sec: self.elapsed_focus_secs,
            remaining_break_sec: self.remaining_break_secs,
            banked_break_sec: self.banked_break_secs,
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
            let is_new_seq = shared.seq != self.last_synced_seq;
            let is_external = shared.writer == "nvim" || shared.writer == "vscode";
            let is_startup_recovery = self.last_synced_seq == 0 && shared.state != "idle" && self.state == FlowState::Idle;

            if (is_external && is_new_seq) || is_startup_recovery {
                self.last_synced_seq = shared.seq;
                self.seq = shared.seq;
                self.focus_ratio = shared.focus_ratio.max(1);
                self.break_ratio = shared.break_ratio.max(1);

                let now = now_epoch();
                match shared.state.as_str() {
                    "micro_start" => {
                        self.state = FlowState::MicroStart;
                        self.start_epoch = shared.start_epoch;
                        let elapsed = if self.start_epoch > 0 && now >= self.start_epoch {
                            now - self.start_epoch
                        } else {
                            0
                        };
                        self.micro_remaining_secs = 300_u64.saturating_sub(elapsed);
                    }
                    "focus" => {
                        self.state = FlowState::Focus;
                        self.start_epoch = shared.start_epoch;
                        self.elapsed_focus_secs = if self.start_epoch > 0 && now >= self.start_epoch {
                            now - self.start_epoch
                        } else {
                            shared.elapsed_focus_sec
                        };
                        self.banked_break_secs = shared.banked_break_sec;
                        self.focus_at_last_break = self.elapsed_focus_secs;
                    }
                    "break" => {
                        self.state = FlowState::Break;
                        self.start_epoch = shared.start_epoch;
                        self.break_duration_sec = shared.break_duration_sec.max(shared.remaining_break_sec);
                        self.initial_break_secs = self.break_duration_sec;
                        let elapsed = if self.start_epoch > 0 && now >= self.start_epoch {
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
                        self.banked_break_secs = 0;
                        self.focus_at_last_break = 0;
                        self.start_epoch = 0;
                    }
                }
            }
        }
    }

    pub fn start_micro_start(&mut self) {
        self.state = FlowState::MicroStart;
        self.elapsed_focus_secs = 0;
        self.banked_break_secs = 0;
        self.focus_at_last_break = 0;
        self.micro_remaining_secs = 300;
        self.accumulator = Duration::ZERO;
        self.start_epoch = now_epoch();
        self.seq += 1;
        self.last_synced_seq = self.seq;
        self.broadcast_state();

        notify(
            "Mikrostart (5 min)",
            "Terskelen er senket! Jobb uforstyrret i 5 minutter – du har lov til å stoppe etterpå.",
        );
    }

    pub fn cancel_micro_start(&mut self) {
        self.reset();
        notify(
            "Bra forsøk!",
            "Helt greit å stoppe her. Null skyldfølelse – prøv igjen senere når du er klar.",
        );
    }

    pub fn start_focus(&mut self) {
        self.state = FlowState::Focus;
        self.elapsed_focus_secs = 0;
        self.banked_break_secs = 0;
        self.focus_at_last_break = 0;
        self.accumulator = Duration::ZERO;
        self.start_epoch = now_epoch();
        self.seq += 1;
        self.last_synced_seq = self.seq;
        self.broadcast_state();

        notify("Fokusøkt startet", "Jobb uforstyrret så lenge du har god flyt!");
    }

    pub fn calculate_break_time(&self) -> u64 {
        let newly_focused = self.elapsed_focus_secs.saturating_sub(self.focus_at_last_break);
        let f_base = self.focus_ratio.max(1) as f64;
        let b_base = self.break_ratio.max(1) as f64;
        let newly_earned = ((newly_focused as f64) * (b_base / f_base)) as u64;
        let total = self.banked_break_secs + newly_earned;
        if total > 0 || self.elapsed_focus_secs >= 10 {
            total.max(30)
        } else {
            0
        }
    }

    pub fn start_break(&mut self) {
        if self.elapsed_focus_secs < 10 && self.banked_break_secs == 0 {
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
        self.banked_break_secs = 0;
        self.focus_at_last_break = self.elapsed_focus_secs;
        self.start_epoch = now_epoch();
        self.accumulator = Duration::ZERO;
        self.seq += 1;
        self.last_synced_seq = self.seq;
        self.broadcast_state();

        notify(
            "Fokusøkt fullført",
            &format!(
                "Fokus: {}.\nOpptjent pause ({}:{}): {}.\nHusk en 60s RAM-dump!",
                focus_fmt, self.focus_ratio, self.break_ratio, break_fmt
            ),
        );
    }

    pub fn resume_focus_from_break(&mut self) {
        let now = now_epoch();
        if self.start_epoch > 0 && self.break_duration_sec > 0 {
            let elapsed = if now >= self.start_epoch {
                now - self.start_epoch
            } else {
                0
            };
            self.remaining_break_secs = self.break_duration_sec.saturating_sub(elapsed);
        }

        // Ta vare på de gjenværende pauseminuttene så de legges til ny beregnet pause
        self.banked_break_secs = self.remaining_break_secs;
        self.remaining_break_secs = 0;
        self.break_duration_sec = 0;

        // Fortsett fokus der den slapp
        self.state = FlowState::Focus;
        self.start_epoch = now.saturating_sub(self.elapsed_focus_secs);
        self.focus_at_last_break = self.elapsed_focus_secs;
        self.accumulator = Duration::ZERO;
        self.seq += 1;
        self.last_synced_seq = self.seq;
        self.broadcast_state();

        let focus_fmt = format_time(self.elapsed_focus_secs);
        let banked_fmt = format_time(self.banked_break_secs);
        notify(
            "Fokus gjenopptatt",
            &format!(
                "Fortsetter fokusøkt fra {}.\nDu har {} pause spart til gode!",
                focus_fmt, banked_fmt
            ),
        );
    }

    pub fn toggle(&mut self) {
        match self.state {
            FlowState::Idle => self.start_focus(),
            FlowState::MicroStart => self.cancel_micro_start(),
            FlowState::Focus => self.start_break(),
            FlowState::Break => self.resume_focus_from_break(),
        }
    }

    pub fn reset(&mut self) {
        self.state = FlowState::Idle;
        self.elapsed_focus_secs = 0;
        self.remaining_break_secs = 0;
        self.banked_break_secs = 0;
        self.focus_at_last_break = 0;
        self.start_epoch = 0;
        self.break_duration_sec = 0;
        self.accumulator = Duration::ZERO;
        self.seq += 1;
        self.last_synced_seq = self.seq;
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
            if self.state == FlowState::Focus {
                self.total_focus_secs += 1;
            }
        }

        let now = now_epoch();
        match self.state {
            FlowState::MicroStart => {
                let elapsed = if self.start_epoch > 0 && now >= self.start_epoch {
                    now - self.start_epoch
                } else {
                    0
                };
                if elapsed >= 300 {
                    // 5 min fullført! Glir automatisk over i Fokus
                    self.state = FlowState::Focus;
                    self.elapsed_focus_secs = elapsed;
                    self.seq += 1;
                    self.last_synced_seq = self.seq;
                    self.broadcast_state();
                    notify(
                        "5 minutter fullført",
                        "Terskelen er brutt og du er i gang! Fortsett flyten så lenge det føles bra.",
                    );
                } else {
                    self.micro_remaining_secs = 300_u64.saturating_sub(elapsed);
                }
            }
            FlowState::Focus => {
                if self.start_epoch > 0 && now >= self.start_epoch {
                    self.elapsed_focus_secs = now - self.start_epoch;
                }
            }
            FlowState::Break => {
                if self.start_epoch > 0 && self.break_duration_sec > 0 {
                    let elapsed = if now >= self.start_epoch {
                        now - self.start_epoch
                    } else {
                        0
                    };
                    self.remaining_break_secs = self.break_duration_sec.saturating_sub(elapsed);
                }
                if self.remaining_break_secs == 0 {
                    self.state = FlowState::Idle;
                    self.start_epoch = 0;
                    self.break_duration_sec = 0;
                    self.banked_break_secs = 0;
                    self.focus_at_last_break = 0;
                    self.seq += 1;
                    self.last_synced_seq = self.seq;
                    self.broadcast_state();
                    notify(
                        "Pausen er over",
                        "Hjernen er uthvilt og klar for neste fokusøkt.",
                    );
                }
            }
            FlowState::Idle => {}
        }
    }

    pub fn current_time_display(&self) -> String {
        match self.state {
            FlowState::MicroStart => format_time(self.micro_remaining_secs),
            FlowState::Focus => format_time(self.elapsed_focus_secs),
            FlowState::Break => format_time(self.remaining_break_secs),
            FlowState::Idle => "00:00".to_string(),
        }
    }

    pub fn progress_pct(&self) -> f32 {
        match self.state {
            FlowState::Idle => 0.0,
            FlowState::MicroStart => {
                let elapsed = 300_u64.saturating_sub(self.micro_remaining_secs);
                (elapsed as f32 / 300.0).clamp(0.0, 1.0)
            }
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

pub(crate) fn notify(title: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .appname("Tether")
        .summary(title)
        .body(body)
        .timeout(notify_rust::Timeout::Milliseconds(6000))
        .show();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_break_resume_banks_unspent_pause_and_continues_focus() {
        let mut f = Flowmodoro::default();
        f.focus_ratio = 45;
        f.break_ratio = 15;

        // 1. Fokus i 45 minutter (2700s)
        f.start_focus();
        f.elapsed_focus_secs = 2700;

        // 2. Start pause: 15 minutter (900s) opptjent
        f.start_break();
        assert_eq!(f.state, FlowState::Break);
        assert_eq!(f.remaining_break_secs, 900);
        assert_eq!(f.banked_break_secs, 0);

        // 3. Bruker bruker 5 minutter, 10 minutter (600s) gjenstår
        f.remaining_break_secs = 600;
        f.break_duration_sec = 600;
        f.start_epoch = now_epoch();

        // 4. Bruker klikker "Avslutt pause" for å fortsette fokusøkt
        f.resume_focus_from_break();
        assert_eq!(f.state, FlowState::Focus);
        assert_eq!(f.elapsed_focus_secs, 2700); // Fokus fortsetter nøyaktig der den slapp!
        assert_eq!(f.banked_break_secs, 600);   // De 10 gjenstående minuttene er trygt spart!
        assert_eq!(f.remaining_break_secs, 0);

        // 5. Bruker jobber i 15 minutter til (900s) -> totalt 3600s (60 min)
        f.elapsed_focus_secs = 3600;

        // 6. Beregn tilgjengelig pause:
        // Ny fokus: 3600 - 2700 = 900s (15 min) -> ny opptjent = 300s (5 min).
        // Total pause = 600s (spart) + 300s (ny) = 900s (15 min)!
        let break_time = f.calculate_break_time();
        assert_eq!(break_time, 900);

        // 7. Start ny pause
        f.start_break();
        assert_eq!(f.state, FlowState::Break);
        assert_eq!(f.remaining_break_secs, 900);
        assert_eq!(f.banked_break_secs, 0);

        // 8. Kun eksplisitt nullstill sletter pausene og fokustiden
        f.reset();
        assert_eq!(f.state, FlowState::Idle);
        assert_eq!(f.elapsed_focus_secs, 0);
        assert_eq!(f.remaining_break_secs, 0);
        assert_eq!(f.banked_break_secs, 0);
    }
}
