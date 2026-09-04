//! Database schema and migrations

use rusqlite::Connection;
use anyhow::Result;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    
    // Game name mappings
    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_mappings (
            title_id TEXT PRIMARY KEY,
            game_name TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    
    // Conversion job history
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversion_jobs (
            id TEXT PRIMARY KEY,
            input_path TEXT NOT NULL,
            output_path TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT,
            title_id TEXT,
            game_name TEXT
        )",
        [],
    )?;
    
    // Create indexes for performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_jobs_status ON conversion_jobs (status)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_jobs_created ON conversion_jobs (created_at DESC)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_mappings_title ON game_mappings (title_id)",
        [],
    )?;
    
    Ok(())
}
