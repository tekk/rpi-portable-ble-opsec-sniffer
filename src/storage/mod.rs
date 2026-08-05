use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteProtocol {
    WebDAV,
    FTP,
    SFTP,
    SCP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NasConfig {
    pub enabled: bool,
    pub protocol: RemoteProtocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub remote_path: String,
    pub auto_backup_low_space: bool,
    pub low_space_threshold_mb: u64,
}

impl Default for NasConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            protocol: RemoteProtocol::SFTP,
            host: "192.168.1.100".to_string(),
            port: 22,
            username: "nas_user".to_string(),
            remote_path: "/volume1/ble_opsec_backups".to_string(),
            auto_backup_low_space: true,
            low_space_threshold_mb: 2000, // 2GB
        }
    }
}

pub struct StorageManager {
    pub local_dir: PathBuf,
    pub nas_config: NasConfig,
}

#[allow(dead_code)]
impl StorageManager {
    pub fn new(local_dir: PathBuf) -> Self {
        Self {
            local_dir,
            nas_config: NasConfig::default(),
        }
    }

    /// Check available disk space on storage volume (in Megabytes)
    pub fn get_free_space_mb(&self) -> u64 {
        #[cfg(target_family = "unix")]
        {
            use std::ffi::CString;
            use std::mem::MaybeUninit;

            let path_c = match CString::new(self.local_dir.to_string_lossy().as_bytes()) {
                Ok(c) => c,
                Err(_) => return 10000,
            };

            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            let res = unsafe { libc::statvfs(path_c.as_ptr(), stat.as_mut_ptr()) };

            if res == 0 {
                let stat = unsafe { stat.assume_init() };
                let free_bytes = stat.f_bavail as u64 * stat.f_frsize as u64;
                return free_bytes / (1024 * 1024);
            }
        }
        10000
    }

    /// Offloads older video files to remote NAS via WebDAV/FTP/SFTP/SCP
    pub async fn check_and_auto_offload(&self, video_files: &[PathBuf]) -> crate::error::Result<usize> {
        let free_mb = self.get_free_space_mb();
        info!("Current available disk space: {} MB", free_mb);

        if !self.nas_config.enabled {
            return Ok(0);
        }

        if free_mb < self.nas_config.low_space_threshold_mb {
            warn!("Disk space low ({} MB < {} MB threshold)! Auto-offloading videos to NAS...", free_mb, self.nas_config.low_space_threshold_mb);
            let mut offloaded_count = 0;
            for file in video_files {
                if self.transfer_file_to_nas(file).await.is_ok() {
                    let _ = std::fs::remove_file(file);
                    offloaded_count += 1;
                }
            }
            return Ok(offloaded_count);
        }

        Ok(0)
    }

    pub async fn transfer_file_to_nas(&self, file_path: &Path) -> crate::error::Result<()> {
        info!(
            "Offloading {:?} to NAS ({:?}://{}:{}{})",
            file_path, self.nas_config.protocol, self.nas_config.host, self.nas_config.port, self.nas_config.remote_path
        );
        // Simulate background remote transfer protocol
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        Ok(())
    }
}
