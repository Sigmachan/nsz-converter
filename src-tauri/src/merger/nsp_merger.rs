//! NSP Multi-part file merger
//! 
//! Some games are distributed as multiple .nsp files (e.g., base game + updates + DLCs, or split large files).
//! This module detects and merges them into a single installable NSP.

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::{Result, bail};
use tracing::{info, debug};
use walkdir::WalkDir;

pub struct NspMerger;

impl NspMerger {
    /// Detect if NSP files in a directory should be merged
    /// Returns groups of files that belong together (same title ID)
    pub fn detect_merge_groups(dir: &Path) -> Result<Vec<MergeGroup>> {
        let mut title_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
        
        // Scan directory for NSP files
        for entry in WalkDir::new(dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if path.extension().map_or(false, |e| e.eq_ignore_ascii_case("nsp")) {
                if let Some(title_id) = Self::extract_title_id(path) {
                    title_groups.entry(title_id).or_default().push(path.to_path_buf());
                }
            }
        }
        
        // Convert to merge groups (only groups with 2+ files need merging)
        let groups: Vec<MergeGroup> = title_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .map(|(title_id, mut files)| {
                // Sort by file size (largest first = likely base game)
                files.sort_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0));
                files.reverse();
                
                MergeGroup {
                    title_id,
                    files,
                    output_name: None,
                }
            })
            .collect();
        
        Ok(groups)
    }
    
    /// Merge multiple NSP files into one
    /// Uses PFS0 container format knowledge to combine them
    pub async fn merge_group(group: &MergeGroup, output_path: &Path) -> Result<PathBuf> {
        info!("Merging {} NSP files for title {}", group.files.len(), group.title_id);
        
        // For now, use a simple approach: if files are split parts of the same game,
        // we need to use specialized tools. The most common case is:
        // - Multiple NSPs with same title ID that should be installed together
        
        // Check if this is a split-file scenario (rare, but supported by some dumpers)
        if Self::is_split_file_scenario(&group.files) {
            return Self::merge_split_files(group, output_path).await;
        }
        
        // Otherwise, these are likely separate titles (base + update + DLC)
        // In this case, we organize them but don't merge the binaries
        // The user will install them separately
        
        bail!("Multi-part NSP merging for separate titles (base/update/DLC) is handled by folder organization, not binary merging")
    }
    
    fn is_split_file_scenario(files: &[PathBuf]) -> bool {
        // Heuristic: if all files have similar sizes and names like "game.part1.nsp", "game.part2.nsp"
        // This is rare - most "multi-part" NSPs are actually different titles
        if files.len() < 2 {
            return false;
        }
        
        let names: Vec<String> = files.iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .map(|s| s.to_lowercase())
            .collect();
        
        // Check for "part", "disc", or numeric suffixes
        names.iter().any(|n| n.contains("part") || n.contains("disc")) ||
        names.iter().any(|n| n.contains(".00") || n.contains(".01"))
    }
    
    async fn merge_split_files(group: &MergeGroup, output_path: &Path) -> Result<PathBuf> {
        // This would require implementing PFS0 merging
        // For production, we'd delegate to a tool like nsc_builder or implement PFS0 spec
        // For now, return an error indicating this needs the full implementation
        
        bail!(
            "Split-file NSP merging requires PFS0 container manipulation. \
             This feature will be implemented using the PFS0 specification. \
             For now, install the NSP files separately - they are valid multi-part dumps."
        )
    }
    
    fn extract_title_id(path: &Path) -> Option<String> {
        // Try filename patterns first
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(caps) = regex::Regex::new(r"\[([0-9A-Fa-f]{16})\]")
                .ok()
                .and_then(|re| re.captures(name))
            {
                return Some(caps[1].to_uppercase());
            }
        }
        
        // Would need to parse NSP header for accurate detection
        // For now, rely on filename
        None
    }
}

#[derive(Debug, Clone)]
pub struct MergeGroup {
    pub title_id: String,
    pub files: Vec<PathBuf>,
    pub output_name: Option<String>,
}
