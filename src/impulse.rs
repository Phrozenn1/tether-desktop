use crate::storage::{self, TetherSettings};

pub struct ImpulseState {
    pub input_text: String,
    pub items: Vec<String>,
    pub feedback: Option<String>,
}

impl Default for ImpulseState {
    fn default() -> Self {
        let settings = storage::load_settings();
        Self {
            input_text: String::new(),
            items: storage::load_impulses_with_settings(&settings),
            feedback: None,
        }
    }
}

impl ImpulseState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reload(&mut self, settings: &TetherSettings) {
        self.items = storage::load_impulses_with_settings(settings);
    }

    pub fn capture_with_settings(&mut self, settings: &TetherSettings) -> Result<(), std::io::Error> {
        let trimmed = self.input_text.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        storage::append_impulse_with_settings(settings, trimmed)?;
        self.items = storage::load_impulses_with_settings(settings);
        self.input_text.clear();
        let msg = match settings.language.resolve() {
            storage::Language::Norwegian => "Strøtanke forankret. Tilbake til flyten.",
            storage::Language::English => "Side-quest captured. Return to flow.",
        };
        self.feedback = Some(msg.to_string());
        Ok(())
    }

    pub fn remove_with_settings(&mut self, settings: &TetherSettings, index: usize) -> Result<(), std::io::Error> {
        if index < self.items.len() {
            self.items.remove(index);
            storage::save_all_impulses_with_settings(settings, &self.items)?;
            let msg = match settings.language.resolve() {
                storage::Language::Norwegian => "Oppgave fullført og fjernet.",
                storage::Language::English => "Task completed and removed.",
            };
            self.feedback = Some(msg.to_string());
        }
        Ok(())
    }

    pub fn capture(&mut self) -> Result<(), std::io::Error> {
        let settings = storage::load_settings();
        self.capture_with_settings(&settings)
    }

    pub fn remove(&mut self, index: usize) -> Result<(), std::io::Error> {
        let settings = storage::load_settings();
        self.remove_with_settings(&settings, index)
    }
}
