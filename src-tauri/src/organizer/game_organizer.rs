//! Organizes converted NSP files into game-named folders
//! Uses user-defined mappings (Title ID -> Game Name) from the database

use std::path::{Path, PathBuf};
use anyhow::Result;
use tracing::info;

pub struct GameOrganizer {
    mappings: std::collections::HashMap<String, String>,
}

impl GameOrganizer {
    pub fn new() -> Self {
        Self {
            mappings: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_mappings(mut self, mappings: std::collections::HashMap<String, String>) -> Self {
        self.mappings = mappings;
        self
    }
    
    /// Get the organized output path for a file
    /// Returns: <output_base>/<GameName>/<filename.nsp>
    pub fn get_organized_path(
        &self,
        title_id: Option<&str>,
        output_base: &Path,
        filename: &str,
    ) -> PathBuf {
        let game_name = title_id
            .and_then(|tid| self.mappings.get(&tid.to_uppercase()))
            .cloned()
            .unwrap_or_else(|| {
                // Fallback naming
                title_id
                    .map(|tid| format!("Unknown_{}", tid))
                    .unwrap_or_else(|| "Unknown_Games".to_string())
            });
        
        output_base.join(game_name).join(filename)
    }
    
    /// Scan a directory and suggest folder organization
    pub fn suggest_organization(dir: &Path) -> Result<Vec<OrganizationSuggestion>> {
        let mut suggestions = Vec::new();
        
        for entry in walkdir::WalkDir::new(dir).max_depth(2).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if path.extension().map_or(false, |e| {
                let ext = e.to_string_lossy().to_lowercase();
                ext == "nsp" || ext == "nsz" || ext == "xci" || ext == "xcz"
            }) {
                if let Some(title_id) = Self::extract_title_id(path) {
                    suggestions.push(OrganizationSuggestion {
                        path: path.to_path_buf(),
                        title_id,
                        suggested_name: None,
                    });
                }
            }
        }
        
        Ok(suggestions)
    }
    
    fn extract_title_id(path: &Path) -> Option<String> {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(caps) = regex::Regex::new(r"\[([0-9A-Fa-f]{16})\]")
                .ok()
                .and_then(|re| re.captures(name))
            {
                return Some(caps[1].to_uppercase());
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct OrganizationSuggestion {
    pub path: PathBuf,
    pub title_id: String,
    pub suggested_name: Option<String>,
}
