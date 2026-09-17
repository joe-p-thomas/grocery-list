use crate::catalog::{Catalog, Section};
use crate::list::GroceryList;

pub const CATALOG_PATH: &str = "data/catalog.yml";
pub const LIST_PATH: &str = "data/list.json";

const TOP_COMMANDS: &[&str] = &["/open-list", "/validate-catalog", "/quit", "/exit"];
const LIST_COMMANDS: &[&str] = &["/add", "/remove", "/clear"];

pub enum Mode {
    Command,
    List,
    AddItem,
    RemoveItem,
}

pub struct App {
    pub mode: Mode,
    pub log: Vec<String>,
    pub working_list: GroceryList,
    pub catalog_sections: Vec<Section>,
    pub input: String,
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
    pub status: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            mode: Mode::Command,
            log: Vec::new(),
            working_list: GroceryList::default(),
            catalog_sections: Vec::new(),
            input: String::new(),
            suggestions: Vec::new(),
            selected_suggestion: 0,
            status: None,
            should_quit: false,
        }
    }

    pub fn input_changed(&mut self) {
        self.suggestions = if self.input.is_empty() {
            Vec::new()
        } else {
            match self.mode {
                Mode::Command => TOP_COMMANDS
                    .iter()
                    .filter(|command| command.starts_with(self.input.as_str()))
                    .map(|command| command.to_string())
                    .collect(),
                Mode::List => LIST_COMMANDS
                    .iter()
                    .filter(|command| command.starts_with(self.input.as_str()))
                    .map(|command| command.to_string())
                    .collect(),
                Mode::AddItem => {
                    let query = self.input.to_lowercase();
                    self.catalog_items()
                        .filter(|item| item.to_lowercase().starts_with(&query))
                        .cloned()
                        .collect()
                }
                Mode::RemoveItem => {
                    let query = self.input.to_lowercase();
                    let mut seen = std::collections::HashSet::new();
                    self.working_list
                        .items
                        .iter()
                        .filter(|item| item.to_lowercase().starts_with(&query))
                        .filter(|item| seen.insert(item.to_lowercase()))
                        .cloned()
                        .collect()
                }
            }
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
        if let Some(suggestion) = self.suggestions.get(self.selected_suggestion) {
            self.input = suggestion.clone();
            self.suggestions.clear();
        }
    }

    pub fn submit_input(&mut self) {
        let text = self.input.trim().to_string();
        self.input.clear();
        self.suggestions.clear();
        if text.is_empty() {
            return;
        }

        match self.mode {
            Mode::Command => self.run_command(&text),
            Mode::List => self.run_list_command(&text),
            Mode::AddItem => self.add_item(&text),
            Mode::RemoveItem => self.remove_item(&text),
        }
    }

    /// Step back one level: AddItem/RemoveItem -> List -> Command -> (quit, handled by caller).
    pub fn back(&mut self) {
        self.mode = match self.mode {
            Mode::AddItem | Mode::RemoveItem => Mode::List,
            Mode::List => Mode::Command,
            Mode::Command => Mode::Command,
        };
        self.input.clear();
        self.suggestions.clear();
        self.status = None;
    }

    /// Working list items grouped by catalog section, in catalog order.
    /// Sections with no items on the list are omitted.
    pub fn grouped_list(&self) -> Vec<(&str, Vec<&str>)> {
        self.catalog_sections
            .iter()
            .filter_map(|section| {
                let mut items: Vec<&str> = self
                    .working_list
                    .items
                    .iter()
                    .filter(|item| {
                        section
                            .items
                            .iter()
                            .any(|catalog_item| catalog_item.eq_ignore_ascii_case(item))
                    })
                    .map(|item| item.as_str())
                    .collect();
                items.sort_by_key(|item| item.to_lowercase());
                if items.is_empty() {
                    None
                } else {
                    Some((section.name.as_str(), items))
                }
            })
            .collect()
    }

    fn catalog_items(&self) -> impl Iterator<Item = &String> {
        self.catalog_sections.iter().flat_map(|s| s.items.iter())
    }

    fn run_command(&mut self, command: &str) {
        match command {
            "/quit" | "/exit" => self.should_quit = true,
            "/validate-catalog" => self.validate_catalog(),
            "/open-list" => self.open_list(),
            other => self.log.push(format!("unknown command: {other}")),
        }
    }

    fn run_list_command(&mut self, command: &str) {
        match command {
            "/add" => self.mode = Mode::AddItem,
            "/remove" => self.mode = Mode::RemoveItem,
            "/clear" => self.clear_list(),
            other => self.status = Some(format!("unknown command: {other}")),
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

    fn open_list(&mut self) {
        match Catalog::load(CATALOG_PATH) {
            Ok(catalog) => self.catalog_sections = catalog.sections,
            Err(e) => self.log.push(format!("Error loading catalog: {e}")),
        }

        match GroceryList::load(LIST_PATH) {
            Ok(list) => self.working_list = list,
            Err(e) => self.log.push(e),
        }

        self.status = None;
        self.mode = Mode::List;
    }

    fn add_item(&mut self, name: &str) {
        let canonical = self
            .catalog_items()
            .find(|item| item.eq_ignore_ascii_case(name))
            .cloned();

        let Some(canonical) = canonical else {
            self.status = Some(format!("Unknown item '{name}' — not in the catalog"));
            return;
        };

        self.working_list.add(canonical.clone());
        match self.working_list.save(LIST_PATH) {
            Ok(()) => self.status = Some(format!("Added {canonical}")),
            Err(e) => self.status = Some(e),
        }
    }

    fn remove_item(&mut self, name: &str) {
        let position = self
            .working_list
            .items
            .iter()
            .position(|item| item.eq_ignore_ascii_case(name));

        let Some(index) = position else {
            self.status = Some(format!("No such item '{name}' on the list"));
            return;
        };

        let removed = self.working_list.items.remove(index);
        match self.working_list.save(LIST_PATH) {
            Ok(()) => self.status = Some(format!("Removed {removed}")),
            Err(e) => self.status = Some(e),
        }
    }

    fn clear_list(&mut self) {
        self.working_list.items.clear();
        match self.working_list.save(LIST_PATH) {
            Ok(()) => self.status = Some("Cleared list".to_string()),
            Err(e) => self.status = Some(e),
        }
    }
}
