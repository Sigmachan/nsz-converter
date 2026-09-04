//! Conversion job management and engine

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{info, warn, error};

use crate::db::Database;
use super::nsz_wrapper::NszWrapper;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversionStatus {
    Pending,
    InProgress { progress: f32 },
    Completed { output_path: PathBuf },
    Failed { error: String },
    Skipped { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionJob {
    pub id: Uuid,
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub status: ConversionStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub title_id: Option<String>,
    pub game_name: Option<String>,
}

pub struct ConversionEngine {
    db: Arc<RwLock<Database>>,
    nsz: NszWrapper,
}

impl ConversionEngine {
    pub fn new(db: Arc<RwLock<Database>>) -> Self {
        let nsz = NszWrapper::new().expect("Failed to initialize nsz wrapper");
        Self { db, nsz }
    }
    
    /// Main entry point: convert a single NSZ file
    pub async fn convert_file(&self, input_path: &str) -> Result<String> {
        let input = PathBuf::from(input_path);
        
        // Validate it's an NSZ file
        if !input.extension().map_or(false, |e| e.eq_ignore_ascii_case("nsz")) {
            anyhow::bail!("Not an NSZ file: {:?}", input);
        }
        
        // Create job
        let job = ConversionJob {
            id: Uuid::new_v4(),
            input_path: input.clone(),
            output_path: None,
            status: ConversionStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            title_id: None,
            game_name: None,
        };
        
        // Save to queue
        {
            let db = self.db.write().await;
            db.add_conversion_job(&job)?;
        }
        
        // Execute conversion
        let result = self.execute_conversion(job).await;
        
        match result {
            Ok(job) => Ok(format!("Conversion completed: {:?}", job.output_path)),
            Err(e) => {
                error!("Conversion failed: {}", e);
                Err(e)
            }
        }
    }
    
    async fn execute_conversion(&self, mut job: ConversionJob) -> Result<ConversionJob> {
        job.started_at = Some(Utc::now());
        job.status = ConversionStatus::InProgress { progress: 0.0 };
        
        // Update status
        {
            let db = self.db.write().await;
            db.update_conversion_job(&job)?;
        }
        
        // Get settings for output directory
        let settings = {
            let db = self.db.read().await;
            db.get_settings()?
        };
        
        let output_base = PathBuf::from(
            settings["output_directory"]
                .as_str()
                .unwrap_or("~/NSZ_Converted")
        );
        
        // Expand ~ to home directory
        let output_base = if output_base.starts_with("~") {
            let home = std::env::var("HOME").unwrap_or_default();
            output_base.to_string_lossy().replacen("~", &home, 1).into()
        } else {
            output_base
        };
        
        // Get title ID for game name lookup
        let title_id = self.detect_title_id(&job.input_path).await?;
        job.title_id = title_id.clone();
        
        // Get game name from mappings
        let game_name = if let Some(ref tid) = title_id {
            let db = self.db.read().await;
            db.get_game_name(tid)?
        } else {
            None
        };
        
        job.game_name = game_name.clone();
        
        // Create game-specific output directory
        let game_dir = if let Some(ref name) = game_name {
            output_base.join(name)
        } else {
            // Fallback to title ID or "Unknown"
            let fallback = title_id.as_deref().unwrap_or("Unknown");
            output_base.join(fallback)
        };
        
        std::fs::create_dir_all(&game_dir)?;
        
        // Perform decompression via nsz
        info!("Converting {:?} -> {:?}", job.input_path, game_dir);
        
        match self.nsz.decompress_nsz(&job.input_path, &game_dir, true).await {
            Ok(nsp_path) => {
                job.output_path = Some(nsp_path.clone());
                job.status = ConversionStatus::Completed { output_path: nsp_path };
                job.completed_at = Some(Utc::now());
                
                info!("Conversion successful: {:?}", job.output_path);
            }
            Err(e) => {
                job.status = ConversionStatus::Failed { error: e.to_string() };
                job.completed_at = Some(Utc::now());
                error!("Conversion failed: {}", e);
            }
        }
        
        // Update database
        {
            let db = self.db.write().await;
            db.update_conversion_job(&job)?;
        }
        
        Ok(job)
    }
    
    async fn detect_title_id(&self, path: &Path) -> Result<Option<String>> {
        // Try to extract from filename first (fast path)
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Common patterns: [0100ABCDEF123456][v0].nsz or TitleName [0100ABCDEF123456].nsz
            if let Some(caps) = regex::Regex::new(r"\[([0-9A-Fa-f]{16})\]")
                .ok()
                .and_then(|re| re.captures(file_name))
            {
                return Ok(Some(caps[1].to_uppercase()));
            }
        }
        
        // Fall back to nsz --info for accurate detection
        match self.nsz.get_file_info(path).await {
            Ok(info) => Ok(info.title_id),
            Err(_) => Ok(None),
        }
    }
    
    /// Batch convert all pending NSZ files
    pub async fn process_pending_queue(&self) -> Result<()> {
        let pending = {
            let db = self.db.read().await;
            db.get_pending_jobs()?
        };
        
        for job in pending {
            if let Err(e) = self.execute_conversion(job).await {
                warn!("Batch job failed: {}", e);
            }
        }
        
        Ok(())
    }
}
