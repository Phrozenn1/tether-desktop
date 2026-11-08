use crate::storage;

pub struct ImpulseState {
    pub input_text: String,
    pub items: Vec<String>,
    pub feedback: Option<String>,
}

impl Default for ImpulseState {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            items: storage::load_impulses(),
            feedback: None,
        }
    }
}

impl ImpulseState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture(&mut self) -> Result<(), std::io::Error> {
        let trimmed = self.input_text.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        storage::append_impulse(trimmed)?;
        self.items = storage::load_impulses();
        self.input_text.clear();
        self.feedback = Some("Strøtanke forankret! ⚓ Tilbake til flyten.".to_string());
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<(), std::io::Error> {
        if index < self.items.len() {
            self.items.remove(index);
            storage::save_all_impulses(&self.items)?;
            self.feedback = Some("Oppgave fullført og fjernet! ✓".to_string());
        }
        Ok(())
    }
}
