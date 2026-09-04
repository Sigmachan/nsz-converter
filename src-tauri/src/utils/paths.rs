//! Path utilities

use std::path::PathBuf;
use anyhow::Result;

pub fn ensure_directory(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(path.replacen("~", &home, 1))
    } else {
        PathBuf::from(path)
    }
}
