use crate::catalog::Catalog;

pub const CATALOG_PATH: &str = "data/catalog.yml";

const COMMANDS: &[&str] = &["/validate-catalog", "/quit", "/exit"];

pub struct App {
    pub log: Vec<String>,
    pub input: String,
    pub suggestions: Vec<&'static str>,
    pub selected_suggestion: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            log: vec!["Type /validate-catalog or /quit".to_string()],
            input: String::new(),
            suggestions: Vec::new(),
            selected_suggestion: 0,
            should_quit: false,
        }
    }

    pub fn input_changed(&mut self) {
        self.suggestions = if self.input.is_empty() {
            Vec::new()
        } else {
            COMMANDS
                .iter()
                .filter(|command| command.starts_with(self.input.as_str()))
                .copied()
                .collect()
        };
        self.selected_suggestion = 0;
    }

    pub fn select_prev_suggestion(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        self.selected_suggestion = self
            .selected_suggestion
            .checked_sub(1)
            .unwrap_or(self.suggestions.len() - 1);
    }

    pub fn select_next_suggestion(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        self.selected_suggestion = (self.selected_suggestion + 1) % self.suggestions.len();
    }

    pub fn accept_suggestion(&mut self) {
        if let Some(command) = self.suggestions.get(self.selected_suggestion) {
            self.input = command.to_string();
            self.suggestions.clear();
        }
    }

    pub fn submit_command(&mut self) {
        let command = self.input.trim().to_string();
        self.input.clear();
        self.suggestions.clear();
        if command.is_empty() {
            return;
        }

        match command.as_str() {
            "/quit" | "/exit" => self.should_quit = true,
            "/validate-catalog" => self.validate_catalog(),
            other => self.log.push(format!("unknown command: {other}")),
        }
    }

    fn validate_catalog(&mut self) {
        match Catalog::load(CATALOG_PATH) {
            Ok(catalog) => {
                let issues = catalog.validate();
                if issues.is_empty() {
                    self.log.push("Catalog is valid.".to_string());
                } else {
                    self.log
                        .push(format!("Catalog has {} problem(s):", issues.len()));
                    for issue in issues {
                        self.log.push(format!("  - {issue}"));
                    }
                }
            }
            Err(e) => self.log.push(format!("Error loading catalog: {e}")),
        }
    }
}
