//! NSZ CLI subprocess wrapper
//! Delegates actual NSZ/NSP conversion to the battle-tested nsz Python tool

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use anyhow::{Result, Context, bail};
use tracing::{info, error, debug};

/// Wrapper around the nsz CLI tool for conversion operations
pub struct NszWrapper {
    nsz_path: PathBuf,
    keys_path: Option<PathBuf>,
}

impl NszWrapper {
    /// Create new wrapper, auto-detecting nsz installation
    pub fn new() -> Result<Self> {
        let nsz_path = Self::find_nsz_binary()?;
        info!("Found nsz binary at: {:?}", nsz_path);
        
        Ok(Self {
            nsz_path,
            keys_path: None,
        })
    }
    
    /// Set custom keys file path
    pub fn with_keys(mut self, keys_path: PathBuf) -> Self {
        self.keys_path = Some(keys_path);
        self
    }
    
    /// Find nsz binary in PATH or common locations
    fn find_nsz_binary() -> Result<PathBuf> {
        // Try common binary names
        let candidates = vec![
            "nsz",
            "nsz.exe",
            "nsz-cli",
            "nsz-cli.exe",
        ];
        
        for candidate in candidates {
            if let Ok(path) = which::which(candidate) {
                return Ok(path);
            }
        }
        
        // Check common installation paths
        #[cfg(target_os = "macos")]
        let common_paths = vec![
            "/usr/local/bin/nsz",
            "/opt/homebrew/bin/nsz",
            format!("{}/.local/bin/nsz", std::env::var("HOME").unwrap_or_default()),
        ];
        
        #[cfg(target_os = "windows")]
        let common_paths = vec![
            r"C:\Program Files\nsz\nsz.exe",
            format!(r"{}\AppData\Local\Programs\nsz\nsz.exe", std::env::var("USERPROFILE").unwrap_or_default()),
        ];
        
        #[cfg(target_os = "linux")]
        let common_paths = vec![
            "/usr/bin/nsz",
            "/usr/local/bin/nsz",
            format!("{}/.local/bin/nsz", std::env::var("HOME").unwrap_or_default()),
        ];
        
        for path in common_paths {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }
        
        bail!(
            "nsz binary not found. Please install nsz first:\n\
             pip3 install --upgrade nsz\n\
             Or download prebuilt binaries from: https://github.com/nicoboss/nsz/releases"
        )
    }
    
    /// Decompress NSZ to NSP (the core operation we need)
    /// 
    /// This is equivalent to: nsz -D --output <out_dir> <nsz_file>
    pub async fn decompress_nsz(
        &self,
        nsz_path: &Path,
        output_dir: &Path,
        overwrite: bool,
    ) -> Result<PathBuf> {
        let mut cmd = Command::new(&self.nsz_path);
        
        cmd.arg("-D")  // Decompress
           .arg("--output")
           .arg(output_dir)
           .arg(nsz_path);
        
        if overwrite {
            cmd.arg("--overwrite");
        }
        
        if let Some(ref keys) = self.keys_path {
            cmd.arg("--keys").arg(keys);
        }
        
        debug!("Running nsz decompress: {:?}", cmd);
        
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute nsz command")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("nsz decompression failed: {}", stderr);
            bail!("nsz decompression failed: {}", stderr);
        }
        
        // Determine output NSP path
        let file_name = nsz_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let nsp_name = file_name.replace(".nsz", ".nsp").replace(".NSZ", ".NSP");
        let nsp_path = output_dir.join(nsp_name);
        
        if !nsp_path.exists() {
            bail!("Expected output file not found: {:?}", nsp_path);
        }
        
        info!("Successfully decompressed {:?} -> {:?}", nsz_path, nsp_path);
        Ok(nsp_path)
    }
    
    /// Verify file integrity (NSP or NSZ)
    pub async fn verify_file(&self, file_path: &Path, quick: bool) -> Result<bool> {
        let mut cmd = Command::new(&self.nsz_path);
        
        if quick {
            cmd.arg("--quick-verify");
        } else {
            cmd.arg("--verify");
        }
        
        cmd.arg(file_path);
        
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;
        
        Ok(output.status.success())
    }
    
    /// Get file info (title ID, version, etc.)
    pub async fn get_file_info(&self, file_path: &Path) -> Result<FileInfo> {
        let mut cmd = Command::new(&self.nsz_path);
        cmd.arg("--info")
           .arg("--machine-readable")
           .arg(file_path);
        
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;
        
        if !output.status.success() {
            bail!("Failed to get file info");
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_file_info(&stdout)
    }
    
    fn parse_file_info(output: &str) -> Result<FileInfo> {
        // Parse machine-readable output from nsz --info
        // Format: key=value pairs
        let mut info = FileInfo::default();
        
        for line in output.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "title_id" => info.title_id = Some(value.trim().to_string()),
                    "version" => info.version = value.trim().parse().ok(),
                    "file_type" => info.file_type = Some(value.trim().to_string()),
                    "size" => info.size = value.trim().parse().ok(),
                    _ => {}
                }
            }
        }
        
        Ok(info)
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    pub title_id: Option<String>,
    pub version: Option<u32>,
    pub file_type: Option<String>,
    pub size: Option<u64>,
}
