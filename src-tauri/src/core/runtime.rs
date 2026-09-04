//! Embedded nsz runtime manager
//! Downloads and manages prebuilt nsz binaries for users without Python

use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail};
use tracing::{info, warn, debug};
use reqwest::Client;
use sha2::{Sha256, Digest};

const NSZ_VERSION: &str = "5.0.0";
const NSZ_RELEASE_URL: &str = "https://github.com/nicoboss/nsz/releases/download";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    MacOS { arch: &'static str },
    Windows { arch: &'static str },
    Linux { arch: &'static str },
}

impl Platform {
    pub fn detect() -> Self {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Platform::MacOS { arch: "arm64" };
        
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Platform::MacOS { arch: "x64" };
        
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Platform::Windows { arch: "x64" };
        
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Platform::Linux { arch: "x64" };
        
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Platform::Linux { arch: "arm64" };
        
        #[allow(unreachable_code)]
        Platform::Linux { arch: "x64" } // fallback
    }
    
    pub fn binary_name(&self) -> &'static str {
        match self {
            Platform::MacOS { .. } => "nsz-cli-macos",
            Platform::Windows { .. } => "nsz-cli.exe",
            Platform::Linux { .. } => "nsz-cli-linux",
        }
    }
    
    pub fn download_url(&self) -> String {
        let (os, arch) = match self {
            Platform::MacOS { arch } => ("macos", arch),
            Platform::Windows { arch } => ("win64", arch),
            Platform::Linux { arch } => ("linux", arch),
        };
        
        format!(
            "{}/{}/nsz_v{}_{}_{}_portable.zip",
            NSZ_RELEASE_URL, NSZ_VERSION, NSZ_VERSION, os, arch
        )
    }
}

pub struct RuntimeManager {
    client: Client,
    platform: Platform,
    install_dir: PathBuf,
}

impl RuntimeManager {
    pub fn new(install_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&install_dir)?;
        
        Ok(Self {
            client: Client::builder()
                .user_agent("NSZ-Converter/1.0")
                .timeout(std::time::Duration::from_secs(300))
                .build()?,
            platform: Platform::detect(),
            install_dir,
        })
    }
    
    /// Check if nsz binary is available (either bundled or system)
    pub async fn ensure_nsz(&self) -> Result<PathBuf> {
        // First check if already bundled
        if let Some(path) = self.find_bundled() {
            info!("Using bundled nsz at: {:?}", path);
            return Ok(path);
        }
        
        // Check system installation
        if let Ok(path) = super::nsz_wrapper::NszWrapper::find_nsz_binary() {
            info!("Using system nsz at: {:?}", path);
            return Ok(path);
        }
        
        // Need to download
        warn!("nsz not found, downloading prebuilt binary for {:?}", self.platform);
        self.download_and_install().await
    }
    
    fn find_bundled(&self) -> Option<PathBuf> {
        let binary_name = self.platform.binary_name();
        let path = self.install_dir.join(binary_name);
        
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
    
    async fn download_and_install(&self) -> Result<PathBuf> {
        let url = self.platform.download_url();
        info!("Downloading nsz from: {}", url);
        
        let response = self.client.get(&url)
            .send()
            .await
            .context("Failed to download nsz release")?;
        
        if !response.status().is_success() {
            bail!("Download failed with status: {}", response.status());
        }
        
        let bytes = response.bytes().await?;
        
        // Extract the portable archive
        let binary_path = self.extract_portable(&bytes).await?;
        
        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&binary_path, perms)?;
        }
        
        info!("nsz installed to: {:?}", binary_path);
        Ok(binary_path)
    }
    
    async fn extract_portable(&self, archive_bytes: &[u8]) -> Result<PathBuf> {
        use std::io::Cursor;
        use zip::ZipArchive;
        
        let reader = Cursor::new(archive_bytes);
        let mut archive = ZipArchive::new(reader)?;
        
        // Find the CLI binary in the archive
        let binary_name = self.platform.binary_name();
        let mut target_path = self.install_dir.join(binary_name);
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name();
            
            // Look for the CLI binary (not the GUI)
            if file_name.contains("cli") || file_name.ends_with(".exe") && file_name.contains("nsz") {
                if file_name.to_lowercase().contains("cli") {
                    std::io::copy(&mut file, &mut std::fs::File::create(&target_path)?)?;
                    return Ok(target_path);
                }
            }
        }
        
        // Fallback: extract first nsz* file
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name();
            
            if file_name.to_lowercase().contains("nsz") && !file_name.ends_with("/") {
                let file_path = Path::new(file_name).file_name()
                    .map(|n| self.install_dir.join(n))
                    .unwrap_or_else(|| self.install_dir.join("nsz"));
                
                std::io::copy(&mut file, &mut std::fs::File::create(&file_path)?)?;
                return Ok(file_path);
            }
        }
        
        bail!("Could not find nsz binary in downloaded archive")
    }
    
    pub fn get_platform_name(&self) -> String {
        match self.platform {
            Platform::MacOS { arch } => format!("macOS ({})", arch),
            Platform::Windows { arch } => format!("Windows ({})", arch),
            Platform::Linux { arch } => format!("Linux ({})", arch),
        }
    }
}
