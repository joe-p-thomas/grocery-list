use crate::catalog::{Catalog, Section};
use crate::list::GroceryList;

pub const CATALOG_PATH: &str = "data/catalog.yml";
pub const LIST_PATH: &str = "data/list.json";

pub const MENU_ITEMS: &[&str] = &["List", "Catalog", "Exit"];
const LIST_COMMANDS: &[&str] = &["/add", "/remove", "/clear"];
const CATALOG_COMMANDS: &[&str] = &["/format", "/open"];
const CATALOG_SECTION_COMMANDS: &[&str] = &["/add"];

pub enum Mode {
    Menu,
    List,
    AddItem,
    RemoveItem,
    Catalog,
    CatalogSection,
    CatalogAddItem,
}

pub struct App {
    pub mode: Mode,
    pub menu_selected: usize,
    pub working_list: GroceryList,
    pub catalog_sections: Vec<Section>,
    pub current_section: Option<String>,
    pub input: String,
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
    pub status: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            mode: Mode::Menu,
            menu_selected: 0,
            working_list: GroceryList::default(),
            catalog_sections: Vec::new(),
            current_section: None,
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
                Mode::Menu => Vec::new(),
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
                Mode::Catalog => {
                    if let Some(partial) = self.input.strip_prefix("/open ") {
                        let query = partial.to_lowercase();
                        self.catalog_sections
                            .iter()
                            .filter(|section| section.name.to_lowercase().starts_with(&query))
                            .map(|section| format!("/open {}", section.name))
                            .collect()
                    } else {
                        CATALOG_COMMANDS
                            .iter()
                            .filter(|command| command.starts_with(self.input.as_str()))
                            .map(|command| command.to_string())
                            .collect()
                    }
                }
                Mode::CatalogSection => CATALOG_SECTION_COMMANDS
                    .iter()
                    .filter(|command| command.starts_with(self.input.as_str()))
                    .map(|command| command.to_string())
                    .collect(),
                Mode::CatalogAddItem => Vec::new(),
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

    pub fn select_menu_prev(&mut self) {
        self.menu_selected = self
            .menu_selected
            .checked_sub(1)
            .unwrap_or(MENU_ITEMS.len() - 1);
    }

    pub fn select_menu_next(&mut self) {
        self.menu_selected = (self.menu_selected + 1) % MENU_ITEMS.len();
    }

    pub fn confirm_menu(&mut self) {
        match self.menu_selected {
            0 => self.open_list(),
            1 => self.open_catalog(),
            2 => self.should_quit = true,
            _ => unreachable!(),
        }
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
            Mode::Menu => {}
            Mode::List => self.run_list_command(&text),
            Mode::AddItem => self.add_item(&text),
            Mode::RemoveItem => self.remove_item(&text),
            Mode::Catalog => self.run_catalog_command(&text),
            Mode::CatalogSection => self.run_catalog_section_command(&text),
            Mode::CatalogAddItem => self.add_catalog_item(&text),
        }
    }

    /// Step back one level: AddItem/RemoveItem -> List, CatalogAddItem -> CatalogSection,
    /// CatalogSection -> Catalog, List/Catalog -> Command -> (quit, handled by caller).
    pub fn back(&mut self) {
        let next = match self.mode {
            Mode::AddItem | Mode::RemoveItem => Mode::List,
            Mode::List | Mode::Catalog => Mode::Menu,
            Mode::CatalogAddItem => Mode::CatalogSection,
            Mode::CatalogSection => Mode::Catalog,
            Mode::Menu => Mode::Menu,
        };
        if matches!(self.mode, Mode::CatalogSection) {
            self.current_section = None;
        }
        self.mode = next;
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

    fn run_list_command(&mut self, command: &str) {
        match command {
            "/add" => self.mode = Mode::AddItem,
            "/remove" => self.mode = Mode::RemoveItem,
            "/clear" => self.clear_list(),
            other => self.status = Some(format!("unknown command: {other}")),
        }
    }

    fn run_catalog_command(&mut self, command: &str) {
        if command == "/format" {
            self.format_catalog();
        } else if let Some(name) = command.strip_prefix("/open ") {
            self.open_catalog_section(name.trim());
        } else {
            self.status = Some(format!("unknown command: {command}"));
        }
    }

    fn run_catalog_section_command(&mut self, command: &str) {
        match command {
            "/add" => self.mode = Mode::CatalogAddItem,
            other => self.status = Some(format!("unknown command: {other}")),
        }
    }

    fn open_list(&mut self) {
        match Catalog::load(CATALOG_PATH) {
            Ok(catalog) => {
                self.catalog_sections = catalog.sections;
                self.status = None;
            }
            Err(e) => self.status = Some(format!("Error loading catalog: {e}")),
        }

        match GroceryList::load(LIST_PATH) {
            Ok(list) => self.working_list = list,
            Err(e) => self.status = Some(e),
        }

        self.mode = Mode::List;
    }

    fn open_catalog(&mut self) {
        match Catalog::load(CATALOG_PATH) {
            Ok(catalog) => {
                self.catalog_sections = catalog.sections;
                self.status = None;
            }
            Err(e) => self.status = Some(e.to_string()),
        }
        self.mode = Mode::Catalog;
    }

    fn open_catalog_section(&mut self, name: &str) {
        let matched = self
            .catalog_sections
            .iter()
            .find(|section| section.name.eq_ignore_ascii_case(name));

        let Some(section) = matched else {
            self.status = Some(format!("No such section '{name}'"));
            return;
        };

        self.current_section = Some(section.name.clone());
        self.status = None;
        self.mode = Mode::CatalogSection;
    }

    fn format_catalog(&mut self) {
        let mut catalog = match Catalog::load(CATALOG_PATH) {
            Ok(catalog) => catalog,
            Err(e) => {
                self.status = Some(e.to_string());
                return;
            }
        };

        catalog.sort_items();
        match catalog.save(CATALOG_PATH) {
            Ok(()) => {
                self.catalog_sections = catalog.sections;
                self.status = Some("Catalog formatted".to_string());
            }
            Err(e) => self.status = Some(e),
        }
    }

    fn add_catalog_item(&mut self, name: &str) {
        let Some(section_name) = self.current_section.clone() else {
            self.status = Some("No section open".to_string());
            return;
        };

        let mut catalog = match Catalog::load(CATALOG_PATH) {
            Ok(catalog) => catalog,
            Err(e) => {
                self.status = Some(e.to_string());
                return;
            }
        };

        let Some(section) = catalog
            .sections
            .iter_mut()
            .find(|section| section.name == section_name)
        else {
            self.status = Some(format!("No such section '{section_name}'"));
            return;
        };

        section.items.push(name.to_string());
        section.items.sort_by_key(|item| item.to_lowercase());

        match catalog.save(CATALOG_PATH) {
            Ok(()) => {
                self.catalog_sections = catalog.sections;
                self.status = Some(format!("Added {name} to {section_name}"));
            }
            Err(e) => self.status = Some(e),
        }
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
