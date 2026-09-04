//! NSZ Converter - Production-ready NSZ/NSP management tool
//! Cross-platform (macOS, Windows, Linux) with automatic monitoring and embedded nsz runtime

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod monitor;
mod merger;
mod organizer;
mod db;
mod utils;

use tauri::{Manager, State};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::Database;
use crate::monitor::FileMonitor;
use crate::core::{ConversionEngine, RuntimeManager};

pub struct AppState {
    db: Arc<RwLock<Database>>,
    monitor: Arc<RwLock<FileMonitor>>,
    engine: Arc<RwLock<ConversionEngine>>,
    runtime: Arc<RwLock<RuntimeManager>>,
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db = state.db.read().await;
    db.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_settings(state: State<'_, AppState>, settings: serde_json::Value) -> Result<(), String> {
    let db = state.db.read().await;
    db.update_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_game_mappings(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.read().await;
    db.get_game_mappings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_game_mapping(
    state: State<'_, AppState>,
    title_id: String,
    game_name: String,
) -> Result<(), String> {
    let db = state.db.read().await;
    db.add_game_mapping(&title_id, &game_name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_game_mapping(state: State<'_, AppState>, title_id: String) -> Result<(), String> {
    let db = state.db.read().await;
    db.remove_game_mapping(&title_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    let mut monitor = state.monitor.write().await;
    monitor.start().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    let mut monitor = state.monitor.write().await;
    monitor.stop().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_conversion_queue(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.read().await;
    db.get_conversion_queue().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_conversion_history(state: State<'_, AppState>, limit: Option<i32>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.read().await;
    db.get_conversion_history(limit.unwrap_or(100)).map_err(|e| e.to_string())
}

#[tauri::command]
async fn manual_convert(state: State<'_, AppState>, file_path: String) -> Result<String, String> {
    let engine = state.engine.read().await;
    engine.convert_file(&file_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ensure_nsz_runtime(state: State<'_, AppState>) -> Result<String, String> {
    let runtime = state.runtime.read().await;
    let path = runtime.ensure_nsz().await.map_err(|e| e.to_string())?;
    Ok(format!("nsz ready at: {:?}", path))
}

#[tauri::command]
async fn get_platform_info(state: State<'_, AppState>) -> Result<String, String> {
    let runtime = state.runtime.read().await;
    Ok(runtime.get_platform_name())
}

#[tauri::command]
async fn open_output_folder(state: State<'_, AppState>, game_name: String) -> Result<(), String> {
    let db = state.db.read().await;
    let settings = db.get_settings().map_err(|e| e.to_string())?;
    let output_dir = settings["output_directory"].as_str().unwrap_or("~/NSZ_Converted");
    
    let path = std::path::Path::new(&output_dir).join(&game_name);
    opener::open(path).map_err(|e| e.to_string())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nsz_converter=debug".parse().unwrap())
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Initialize database
            let db_path = app.path().app_data_dir().unwrap().join("nsz_converter.db");
            let db = Database::new(&db_path).expect("Failed to initialize database");
            let db = Arc::new(RwLock::new(db));
            
            // Initialize runtime manager (embedded nsz binaries)
            let runtime_dir = app.path().app_data_dir().unwrap().join("nsz_runtime");
            let runtime = RuntimeManager::new(runtime_dir).expect("Failed to initialize runtime manager");
            let runtime = Arc::new(RwLock::new(runtime));
            
            // Initialize conversion engine
            let engine = ConversionEngine::new(db.clone());
            let engine = Arc::new(RwLock::new(engine));
            
            // Initialize file monitor
            let monitor = FileMonitor::new(db.clone(), engine.clone(), app_handle);
            let monitor = Arc::new(RwLock::new(monitor));
            
            app.manage(AppState {
                db,
                monitor,
                engine,
                runtime,
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            get_game_mappings,
            add_game_mapping,
            remove_game_mapping,
            start_monitoring,
            stop_monitoring,
            get_conversion_queue,
            get_conversion_history,
            manual_convert,
            ensure_nsz_runtime,
            get_platform_info,
            open_output_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
