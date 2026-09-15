use std::fmt;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub sections: Vec<Section>,
}

#[derive(Debug, Deserialize)]
pub struct Section {
    pub name: String,
    pub items: Vec<String>,
}

#[derive(Debug)]
pub enum LoadError {
    Io { path: String, source: std::io::Error },
    Parse(serde_yaml::Error),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { path, source } => {
                write!(f, "couldn't read catalog file '{path}': {source}")
            }
            LoadError::Parse(e) => write!(f, "couldn't parse catalog YAML: {e}"),
        }
    }
}

impl Catalog {
    pub fn load(path: impl AsRef<Path>) -> Result<Catalog, LoadError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.display().to_string(),
            source,
        })?;
        serde_yaml::from_str(&contents).map_err(LoadError::Parse)
    }

    /// Returns a list of human-readable problems with the catalog.
    /// An empty list means the catalog is valid.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let mut seen_sections = std::collections::HashSet::new();

        for section in &self.sections {
            if section.name.trim().is_empty() {
                issues.push("a section has an empty name".to_string());
            } else if !seen_sections.insert(section.name.as_str()) {
                issues.push(format!("duplicate section: {}", section.name));
            }

            if section.items.is_empty() {
                issues.push(format!("section '{}' has no items", section.name));
            }

            let mut seen_items = std::collections::HashSet::new();
            for item in &section.items {
                if item.trim().is_empty() {
                    issues.push(format!("section '{}' has an empty item name", section.name));
                } else if !seen_items.insert(item.as_str()) {
                    issues.push(format!("duplicate item '{item}' in section '{}'", section.name));
                }
            }
        }

        issues
    }
}

impl fmt::Display for Catalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for section in &self.sections {
            writeln!(f, "{}", section.name)?;
            for item in &section.items {
                writeln!(f, "  - {item}")?;
            }
        }
        Ok(())
    }
}
