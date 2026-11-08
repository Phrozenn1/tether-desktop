use crate::storage;

pub struct RamDumpState {
    pub now: String,
    pub status: String,
    pub next_step: String,
    pub feedback_message: Option<String>,
    pub last_dump: Option<(String, String, String, String)>,
}

impl Default for RamDumpState {
    fn default() -> Self {
        Self {
            now: String::new(),
            status: String::new(),
            next_step: String::new(),
            feedback_message: None,
            last_dump: storage::load_last_ramdump(),
        }
    }
}

impl RamDumpState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if self.now.trim().is_empty() && self.status.trim().is_empty() && self.next_step.trim().is_empty() {
            self.feedback_message = Some("Fyll ut minst ett felt før du lagrer!".to_string());
            return Ok(());
        }

        storage::append_ramdump(&self.now, &self.status, &self.next_step)?;

        self.last_dump = storage::load_last_ramdump();
        self.now.clear();
        self.status.clear();
        self.next_step.clear();
        self.feedback_message = Some("RAM-dump lagret og arbeidsminnet er frigjort! ⚓".to_string());
        Ok(())
    }
}
