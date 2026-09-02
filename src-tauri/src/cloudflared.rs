// cloudflared.rs — Manages the cloudflared binary lifecycle
// Handles download, install, start, stop, and status monitoring

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sysinfo::{System, Pid};

pub struct CloudflaredManager {
    /// Active cloudflared child processes keyed by tunnel name
    processes: Mutex<HashMap<String, Child>>,
    /// Current status of each tunnel
    statuses: Mutex<HashMap<String, TunnelStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub name: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub public_url: Option<String>,
    pub services: Vec<ServiceEntry>,
    pub error: Option<String>,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub hostname: String,
    pub service: String,
    pub protocol: String,
}

impl Default for CloudflaredManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CloudflaredManager {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
        }
    }

    /// Get the expected path to the cloudflared binary for the current OS
    pub fn get_binary_path() -> PathBuf {
        let app_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        let dir = app_dir.join("tunnelforge");
        let _ = std::fs::create_dir_all(&dir);

        #[cfg(target_os = "windows")]
        {
            dir.join("cloudflared.exe")
        }
        #[cfg(target_os = "macos")]
        {
            dir.join("cloudflared")
        }
        #[cfg(target_os = "linux")]
        {
            dir.join("cloudflared")
        }
    }

    /// Check if cloudflared is installed (either our managed copy or system PATH)
    pub fn is_installed() -> bool {
        let managed_path = Self::get_binary_path();
        if managed_path.exists() {
            return true;
        }
        which::which("cloudflared").is_ok()
    }

    /// Get the binary path to use — prefer managed install, fall back to system PATH
    pub fn resolve_binary() -> Option<PathBuf> {
        let managed = Self::get_binary_path();
        if managed.exists() {
            return Some(managed);
        }
        which::which("cloudflared").ok()
    }

    /// Get the download URL for the current platform
    pub fn get_download_url() -> &'static str {
        #[cfg(target_os = "windows")]
        {
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe"
        }
        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "aarch64")]
            {
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-arm64.tgz"
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz"
            }
        }
        #[cfg(target_os = "linux")]
        {
            #[cfg(target_arch = "aarch64")]
            {
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64"
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64"
            }
        }
    }

    /// Download and install cloudflared binary
    pub async fn download_binary() -> Result<PathBuf, String> {
        let url = Self::get_download_url();
        let dest = Self::get_binary_path();

        log::info!("Downloading cloudflared from {}", url);

        let response = reqwest::get(url)
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed: HTTP {}", response.status()));
        }

        let bytes = response.bytes()
            .await
            .map_err(|e| format!("Failed to read download: {}", e))?;

        // Handle macOS .tgz archive
        if url.ends_with(".tgz") {
            let temp_tgz = dest.with_extension("tgz");
            std::fs::write(&temp_tgz, &bytes)
                .map_err(|e| format!("Failed to write archive: {}", e))?;

            // Extract using tar command
            let status = Command::new("tar")
                .args(&["-xzf", temp_tgz.to_str().unwrap(), "-C", dest.parent().unwrap().to_str().unwrap()])
                .status()
                .map_err(|e| format!("Failed to extract archive: {}", e))?;

            if !status.success() {
                return Err("Failed to extract cloudflared archive".to_string());
            }
            let _ = std::fs::remove_file(&temp_tgz);
        } else {
            std::fs::write(&dest, &bytes)
                .map_err(|e| format!("Failed to write binary: {}", e))?;
        }

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest)
                .map_err(|e| format!("Failed to read metadata: {}", e))?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest, perms)
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }

        log::info!("cloudflared installed to {:?}", dest);
        Ok(dest)
    }

    /// Start a tunnel with the given config file
    pub fn start_tunnel(&self, tunnel_name: &str, config_path: &PathBuf) -> Result<(), String> {
        let binary = Self::resolve_binary()
            .ok_or_else(|| "cloudflared binary not found. Please install it first.".to_string())?;

        log::info!("Starting tunnel '{}' with config {:?}", tunnel_name, config_path);

        let child = Command::new(&binary)
            .args(&["tunnel", "--config", config_path.to_str().unwrap(), "run"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start cloudflared: {}", e))?;

        let pid = child.id();

        // Store the process handle
        {
            let mut procs = self.processes.lock().unwrap();
            procs.insert(tunnel_name.to_string(), child);
        }

        // Update status
        {
            let mut statuses = self.statuses.lock().unwrap();
            statuses.insert(tunnel_name.to_string(), TunnelStatus {
                name: tunnel_name.to_string(),
                running: true,
                pid: Some(pid),
                started_at: Some(Utc::now()),
                public_url: None, // Will be updated when we parse logs
                services: vec![],
                error: None,
                bytes_in: 0,
                bytes_out: 0,
            });
        }

        log::info!("Tunnel '{}' started with PID {}", tunnel_name, pid);
        Ok(())
    }

    /// Start a tunnel using a Cloudflare token (Zero Trust dashboard managed)
    pub fn start_tunnel_with_token(&self, tunnel_name: &str, token: &str) -> Result<(), String> {
        let binary = Self::resolve_binary()
            .ok_or_else(|| "cloudflared binary not found. Please install it first.".to_string())?;

        log::info!("Starting tunnel '{}' with token", tunnel_name);

        let child = Command::new(&binary)
            .args(&["tunnel", "run", "--token", token])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start cloudflared: {}", e))?;

        let pid = child.id();

        {
            let mut procs = self.processes.lock().unwrap();
            procs.insert(tunnel_name.to_string(), child);
        }

        {
            let mut statuses = self.statuses.lock().unwrap();
            statuses.insert(tunnel_name.to_string(), TunnelStatus {
                name: tunnel_name.to_string(),
                running: true,
                pid: Some(pid),
                started_at: Some(Utc::now()),
                public_url: None,
                services: vec![],
                error: None,
                bytes_in: 0,
                bytes_out: 0,
            });
        }

        log::info!("Token tunnel '{}' started with PID {}", tunnel_name, pid);
        Ok(())
    }

    /// Stop a running tunnel
    pub fn stop_tunnel(&self, tunnel_name: &str) -> Result<(), String> {
        let mut procs = self.processes.lock().unwrap();
        if let Some(mut child) = procs.remove(tunnel_name) {
            // Try graceful kill first
            #[cfg(unix)]
            {
                if let Some(pid) = child.id() as i32 {
                    unsafe { libc::kill(pid, libc::SIGTERM) };
                }
            }
            #[cfg(windows)]
            {
                let _ = child.kill();
            }

            // Wait a moment then force kill if still alive
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = child.kill();
            let _ = child.wait();

            log::info!("Tunnel '{}' stopped", tunnel_name);
        }

        // Update status
        let mut statuses = self.statuses.lock().unwrap();
        if let Some(status) = statuses.get_mut(tunnel_name) {
            status.running = false;
            status.pid = None;
            status.started_at = None;
        }

        Ok(())
    }

    /// Stop all running tunnels
    pub fn stop_all(&self) {
        let names: Vec<String> = {
            let procs = self.processes.lock().unwrap();
            procs.keys().cloned().collect()
        };
        for name in &names {
            let _ = self.stop_tunnel(name);
        }
    }

    /// Get current status of a tunnel
    pub fn get_status(&self, tunnel_name: &str) -> Option<TunnelStatus> {
        let statuses = self.statuses.lock().unwrap();
        statuses.get(tunnel_name).cloned()
    }

    /// Get status of all tunnels
    pub fn get_all_statuses(&self) -> Vec<TunnelStatus> {
        let statuses = self.statuses.lock().unwrap();
        statuses.values().cloned().collect()
    }

    /// Check if a tunnel process is still alive and update status
    pub fn check_processes_alive(&self) {
        let mut procs = self.processes.lock().unwrap();
        let mut statuses = self.statuses.lock().unwrap();
        let mut sys = System::new_all();

        // Collect names of dead tunnels
        let mut dead: Vec<String> = vec![];

        for (name, child) in procs.iter_mut() {
            let pid = child.id();
            sys.refresh_processes();

            // Check if process is still running
            match child.try_wait() {
                Ok(Some(_status)) => {
                    dead.push(name.clone());
                }
                Ok(None) => {
                    // Still running
                    if let Some(status) = statuses.get_mut(name) {
                        status.running = true;
                    }
                }
                Err(_) => {
                    dead.push(name.clone());
                }
            }
        }

        for name in &dead {
            procs.remove(name);
            if let Some(status) = statuses.get_mut(name) {
                status.running = false;
                status.pid = None;
                status.started_at = None;
                status.error = Some("Process exited unexpectedly".to_string());
            }
            log::warn!("Tunnel '{}' died unexpectedly", name);
        }
    }
}
