//! Real-time file system watcher using notify crate

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use anyhow::Result;
use tracing::{info, warn, debug};

use crate::db::Database;
use crate::core::ConversionEngine;

pub struct FileMonitor {
    db: Arc<RwLock<Database>>,
    engine: Arc<RwLock<ConversionEngine>>,
    app_handle: tauri::AppHandle,
    watcher: Option<notify::RecommendedWatcher>,
    is_running: bool,
}

impl FileMonitor {
    pub fn new(
        db: Arc<RwLock<Database>>,
        engine: Arc<RwLock<ConversionEngine>>,
        app_handle: tauri::AppHandle,
    ) -> Self {
        Self {
            db,
            engine,
            app_handle,
            watcher: None,
            is_running: false,
        }
    }
    
    /// Start monitoring the configured input directory
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }
        
        // Get monitored path from settings
        let monitored_path = {
            let db = self.db.read().await;
            let settings = db.get_settings()?;
            PathBuf::from(
                settings["monitored_directory"]
                    .as_str()
                    .unwrap_or("~/Downloads")
            )
        };
        
        // Expand ~
        let monitored_path = if monitored_path.starts_with("~") {
            let home = std::env::var("HOME").unwrap_or_default();
            monitored_path.to_string_lossy().replacen("~", &home, 1).into()
        } else {
            monitored_path
        };
        
        if !monitored_path.exists() {
            anyhow::bail!("Monitored directory does not exist: {:?}", monitored_path);
        }
        
        info!("Starting file monitor on: {:?}", monitored_path);
        
        // Create channel for file events
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        
        // Setup notify watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    for path in event.paths {
                        if path.extension().map_or(false, |e| e.eq_ignore_ascii_case("nsz")) {
                            let _ = tx.blocking_send(path);
                        }
                    }
                }
            }
        })?;
        
        watcher.watch(&monitored_path, RecursiveMode::Recursive)?;
        
        self.watcher = Some(watcher);
        self.is_running = true;
        
        // Spawn background task to handle file events
        let db = self.db.clone();
        let engine = self.engine.clone();
        let app_handle = self.app_handle.clone();
        
        tokio::spawn(async move {
            while let Some(path) = rx.recv().await {
                // Debounce: wait a bit to ensure file is fully written
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                // Check if file still exists and is stable
                if !Self::is_file_stable(&path).await {
                    warn!("File not stable, skipping: {:?}", path);
                    continue;
                }
                
                info!("Detected new NSZ file: {:?}", path);
                
                // Send notification
                let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                    .title("NSZ Detected")
                    .body(format!("Converting: {}", path.file_name().unwrap_or_default().to_string_lossy()))
                    .show();
                
                // Queue for conversion
                let engine_guard = engine.read().await;
                if let Err(e) = engine_guard.convert_file(&path.to_string_lossy()).await {
                    error!("Auto-conversion failed for {:?}: {}", path, e);
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        self.is_running = false;
        self.watcher = None;
        info!("File monitor stopped");
        Ok(())
    }
    
    /// Check if file has finished being written (size stable for 1 second)
    async fn is_file_stable(path: &Path) -> bool {
        if !path.exists() {
            return false;
        }
        
        let size1 = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        tokio::time::sleep(Duration::from_secs(1)).await;
        let size2 = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        
        size1 == size2 && size1 > 0
    }
}
