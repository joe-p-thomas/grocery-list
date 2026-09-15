use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GroceryList {
    pub items: Vec<String>,
}

impl GroceryList {
    pub fn load(path: impl AsRef<Path>) -> Result<GroceryList, String> {
        let path = path.as_ref();
        match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents)
                .map_err(|e| format!("couldn't parse list file '{}': {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(GroceryList::default()),
            Err(e) => Err(format!("couldn't read list file '{}': {e}", path.display())),
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("couldn't serialize list: {e}"))?;
        fs::write(path, json).map_err(|e| format!("couldn't write list file '{}': {e}", path.display()))
    }

    pub fn add(&mut self, item: String) {
        self.items.push(item);
    }
}
