//! SQLite database for settings, game mappings, and conversion history

mod schema;

use std::path::Path;
use anyhow::Result;
use rusqlite::{Connection, params};
use serde_json::Value;
use chrono::Utc;
use uuid::Uuid;

use crate::core::conversion::{ConversionJob, ConversionStatus};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(db_path)?;
        
        // Run migrations
        schema::run_migrations(&conn)?;
        
        Ok(Self { conn })
    }
    
    // Settings
    
    pub fn get_settings(&self) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value FROM settings"
        )?;
        
        let settings: std::collections::HashMap<String, Value> = stmt
            .query_map([], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, serde_json::from_str(&value).unwrap_or(Value::String(value))))
            })?
            .collect::<std::result::Result<_, _>>()?;
        
        // Return defaults if empty
        if settings.is_empty() {
            return Ok(serde_json::json!({
                "monitored_directory": "~/Downloads",
                "output_directory": "~/NSZ_Converted",
                "auto_convert": true,
                "auto_merge": true,
                "verify_after_convert": true,
                "delete_source_after_convert": false,
            }));
        }
        
        Ok(serde_json::to_value(settings)?)
    }
    
    pub fn update_settings(&self, settings: &Value) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        
        if let Value::Object(map) = settings {
            for (key, value) in map {
                tx.execute(
                    "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                    params![key, value.to_string(), Utc::now().to_rfc3339()],
                )?;
            }
        }
        
        tx.commit()?;
        Ok(())
    }
    
    // Game mappings (Title ID -> Game Name)
    
    pub fn get_game_mappings(&self) -> Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT title_id, game_name, created_at FROM game_mappings ORDER BY game_name"
        )?;
        
        let mappings = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "title_id": row.get::<_, String>(0)?,
                    "game_name": row.get::<_, String>(1)?,
                    "created_at": row.get::<_, String>(2)?,
                }))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(mappings)
    }
    
    pub fn add_game_mapping(&self, title_id: &str, game_name: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO game_mappings (title_id, game_name, created_at) VALUES (?1, ?2, ?3)",
            params![title_id.to_uppercase(), game_name, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }
    
    pub fn remove_game_mapping(&self, title_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM game_mappings WHERE title_id = ?1",
            params![title_id.to_uppercase()],
        )?;
        Ok(())
    }
    
    pub fn get_game_name(&self, title_id: &str) -> Result<Option<String>> {
        let result: Option<String> = self.conn.query_row(
            "SELECT game_name FROM game_mappings WHERE title_id = ?1",
            params![title_id.to_uppercase()],
            |row| row.get(0),
        ).ok();
        
        Ok(result)
    }
    
    // Conversion jobs
    
    pub fn add_conversion_job(&self, job: &ConversionJob) -> Result<()> {
        self.conn.execute(
            "INSERT INTO conversion_jobs 
             (id, input_path, output_path, status, created_at, started_at, completed_at, title_id, game_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                job.id.to_string(),
                job.input_path.to_string_lossy().to_string(),
                job.output_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                serde_json::to_string(&job.status)?,
                job.created_at.to_rfc3339(),
                job.started_at.map(|t| t.to_rfc3339()),
                job.completed_at.map(|t| t.to_rfc3339()),
                job.title_id,
                job.game_name,
            ],
        )?;
        Ok(())
    }
    
    pub fn update_conversion_job(&self, job: &ConversionJob) -> Result<()> {
        self.conn.execute(
            "UPDATE conversion_jobs SET 
             output_path = ?2, status = ?3, started_at = ?4, completed_at = ?5, title_id = ?6, game_name = ?7
             WHERE id = ?1",
            params![
                job.id.to_string(),
                job.output_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                serde_json::to_string(&job.status)?,
                job.started_at.map(|t| t.to_rfc3339()),
                job.completed_at.map(|t| t.to_rfc3339()),
                job.title_id,
                job.game_name,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_pending_jobs(&self) -> Result<Vec<ConversionJob>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, input_path, output_path, status, created_at, started_at, completed_at, title_id, game_name
             FROM conversion_jobs 
             WHERE json_extract(status, '$.Pending') IS NOT NULL OR json_extract(status, '$.InProgress') IS NOT NULL
             ORDER BY created_at"
        )?;
        
        let jobs = stmt
            .query_map([], |row| {
                Ok(ConversionJob {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    input_path: PathBuf::from(row.get::<_, String>(1)?),
                    output_path: row.get::<_, Option<String>>(2)?.map(PathBuf::from),
                    status: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?).unwrap().with_timezone(&Utc),
                    started_at: row.get::<_, Option<String>>(5)?.map(|s| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                    completed_at: row.get::<_, Option<String>>(6)?.map(|s| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                    title_id: row.get(7)?,
                    game_name: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(jobs)
    }
    
    pub fn get_conversion_queue(&self) -> Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, input_path, output_path, status, created_at, title_id, game_name
             FROM conversion_jobs 
             WHERE json_extract(status, '$.Pending') IS NOT NULL OR json_extract(status, '$.InProgress') IS NOT NULL
             ORDER BY created_at DESC"
        )?;
        
        let queue = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "input_path": row.get::<_, String>(1)?,
                    "output_path": row.get::<_, Option<String>>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "created_at": row.get::<_, String>(4)?,
                    "title_id": row.get::<_, Option<String>>(5)?,
                    "game_name": row.get::<_, Option<String>>(6)?,
                }))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(queue)
    }
    
    pub fn get_conversion_history(&self, limit: i32) -> Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, input_path, output_path, status, created_at, completed_at, title_id, game_name
             FROM conversion_jobs 
             WHERE json_extract(status, '$.Completed') IS NOT NULL OR json_extract(status, '$.Failed') IS NOT NULL
             ORDER BY completed_at DESC
             LIMIT ?1"
        )?;
        
        let history = stmt
            .query_map(params![limit], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "input_path": row.get::<_, String>(1)?,
                    "output_path": row.get::<_, Option<String>>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "created_at": row.get::<_, String>(4)?,
                    "completed_at": row.get::<_, Option<String>>(5)?,
                    "title_id": row.get::<_, Option<String>>(6)?,
                    "game_name": row.get::<_, Option<String>>(7)?,
                }))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(history)
    }
}
