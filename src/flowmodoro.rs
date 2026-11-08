use std::time::Duration;

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
            accumulator: Duration::ZERO,
        }
    }
}

impl Flowmodoro {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn is_focus(&self) -> bool {
        self.state == FlowState::Focus
    }

    #[allow(dead_code)]
    pub fn is_break(&self) -> bool {
        self.state == FlowState::Break
    }

    pub fn is_active(&self) -> bool {
        self.state != FlowState::Idle
    }

    pub fn start_focus(&mut self) {
        self.state = FlowState::Focus;
        self.elapsed_focus_secs = 0;
        self.accumulator = Duration::ZERO;
        notify("Fokusøkt startet 🚀", "Jobb uforstyrret så lenge du har god flyt!");
    }

    pub fn calculate_break_time(&self) -> u64 {
        let f_base = self.focus_ratio.max(1) as f64;
        let b_base = self.break_ratio.max(1) as f64;
        let earned = (self.elapsed_focus_secs as f64) * (b_base / f_base);
        (earned as u64).max(30) // Minimum 30 sekunders pause
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
        self.accumulator = Duration::ZERO;

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
        self.accumulator = Duration::ZERO;
    }

    pub fn tick(&mut self, dt: Duration) {
        if self.state == FlowState::Idle {
            return;
        }

        self.accumulator += dt;
        while self.accumulator >= Duration::from_secs(1) {
            self.accumulator -= Duration::from_secs(1);

            match self.state {
                FlowState::Focus => {
                    self.elapsed_focus_secs += 1;
                }
                FlowState::Break => {
                    if self.remaining_break_secs > 0 {
                        self.remaining_break_secs -= 1;
                    }
                    if self.remaining_break_secs == 0 {
                        self.state = FlowState::Idle;
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
