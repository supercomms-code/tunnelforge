// commands.rs — Tauri IPC commands (the bridge between Rust backend and React frontend)

use crate::cloudflared::{CloudflaredManager, TunnelStatus, cloudflared_command};
use crate::config::{
    AppConfig, TunnelDefinition, ServiceMapping, load_config, save_config,
    save_tunnel_config, generate_quick_tunnel_args,
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::ShellExt;
use uuid::Uuid;

/// Check if cloudflared binary is installed
#[tauri::command]
pub fn check_cloudflared_installed() -> bool {
    CloudflaredManager::is_installed()
}

/// Get the download URL for cloudflared for the current platform
#[tauri::command]
pub fn get_cloudflared_download_url() -> String {
    CloudflaredManager::get_download_url().to_string()
}

/// Download and install the cloudflared binary
#[tauri::command]
pub async fn install_cloudflared() -> Result<String, String> {
    CloudflaredManager::download_binary().await
        .map(|p| p.to_string_lossy().to_string())
}

/// Load the full app config (all tunnels + settings)
#[tauri::command]
pub fn get_config() -> AppConfig {
    load_config()
}

/// Save the full app config
#[tauri::command]
pub fn save_app_config(config: AppConfig) -> Result<(), String> {
    save_config(&config)
}

/// Create a new tunnel
#[tauri::command]
pub fn create_tunnel(
    name: String,
    token: Option<String>,
    auto_start: bool,
) -> Result<TunnelDefinition, String> {
    let mut config = load_config();
    
    // Check for duplicate name
    if config.tunnels.iter().any(|t| t.name == name) {
        return Err(format!("A tunnel named '{}' already exists", name));
    }

    let tunnel = TunnelDefinition {
        id: Uuid::new_v4().to_string(),
        name,
        token,
        credentials_file: None,
        services: vec![],
        auto_start,
    };

    config.tunnels.push(tunnel.clone());
    save_config(&config)?;
    Ok(tunnel)
}

/// Delete a tunnel by ID
#[tauri::command]
pub fn delete_tunnel(
    manager: State<'_, CloudflaredManager>,
    tunnel_id: String,
) -> Result<(), String> {
    let mut config = load_config();
    
    if let Some(tunnel) = config.tunnels.iter().find(|t| t.id == tunnel_id) {
        // Stop the tunnel if running
        let _ = manager.stop_tunnel(&tunnel.name);
    }

    config.tunnels.retain(|t| t.id != tunnel_id);
    save_config(&config)
}

/// Add a service mapping to an existing tunnel
#[tauri::command]
pub fn add_service(
    tunnel_id: String,
    hostname: String,
    protocol: String,
    local_host: String,
    local_port: u16,
    description: String,
) -> Result<(), String> {
    let mut config = load_config();
    
    let tunnel = config.tunnels.iter_mut()
        .find(|t| t.id == tunnel_id)
        .ok_or("Tunnel not found")?;

    let service = ServiceMapping {
        id: Uuid::new_v4().to_string(),
        hostname,
        protocol,
        local_host,
        local_port,
        description,
    };

    tunnel.services.push(service);
    save_config(&config)?;

    // Regenerate the cloudflared config file
    let updated_config = load_config();
    let tunnel = updated_config.tunnels.iter()
        .find(|t| t.id == tunnel_id)
        .unwrap();
    let _ = save_tunnel_config(tunnel);

    Ok(())
}

/// Remove a service from a tunnel
#[tauri::command]
pub fn remove_service(
    tunnel_id: String,
    service_id: String,
) -> Result<(), String> {
    let mut config = load_config();
    
    let tunnel = config.tunnels.iter_mut()
        .find(|t| t.id == tunnel_id)
        .ok_or("Tunnel not found")?;

    tunnel.services.retain(|s| s.id != service_id);
    save_config(&config)?;

    // Regenerate cloudflared config
    let updated_config = load_config();
    let tunnel = updated_config.tunnels.iter()
        .find(|t| t.id == tunnel_id)
        .unwrap();
    let _ = save_tunnel_config(tunnel);

    Ok(())
}

/// Start a tunnel
#[tauri::command]
pub fn start_tunnel(
    manager: State<'_, CloudflaredManager>,
    tunnel_id: String,
) -> Result<(), String> {
    let config = load_config();
    let tunnel = config.tunnels.iter()
        .find(|t| t.id == tunnel_id)
        .ok_or("Tunnel not found")?;

    // If tunnel has a token, use token-based start
    if let Some(token) = &tunnel.token {
        return manager.start_tunnel_with_token(&tunnel.name, token);
    }

    // Otherwise, use config file
    if tunnel.services.is_empty() {
        return Err("No services configured for this tunnel. Add at least one service before starting.".to_string());
    }

    let config_path = save_tunnel_config(tunnel)?;
    manager.start_tunnel(&tunnel.name, &config_path)
}

/// Stop a tunnel
#[tauri::command]
pub fn stop_tunnel(
    manager: State<'_, CloudflaredManager>,
    tunnel_id: String,
) -> Result<(), String> {
    let config = load_config();
    let tunnel = config.tunnels.iter()
        .find(|t| t.id == tunnel_id)
        .ok_or("Tunnel not found")?;
    manager.stop_tunnel(&tunnel.name)
}

/// Get the status of a specific tunnel
#[tauri::command]
pub fn get_tunnel_status(
    manager: State<'_, CloudflaredManager>,
    tunnel_name: String,
) -> Option<TunnelStatus> {
    manager.get_status(&tunnel_name)
}

/// Get status of all tunnels
#[tauri::command]
pub fn get_all_tunnel_statuses(
    manager: State<'_, CloudflaredManager>,
) -> Vec<TunnelStatus> {
    manager.check_processes_alive();
    manager.get_all_statuses()
}

/// Start a quick tunnel (no Cloudflare account needed — uses trycloudflare.com)
#[tauri::command]
pub fn start_quick_tunnel(
    app: AppHandle,
    manager: State<'_, CloudflaredManager>,
    local_port: u16,
    protocol: String,
) -> Result<(), String> {
    let binary = CloudflaredManager::resolve_binary()
        .ok_or("cloudflared not installed")?;

    let args = generate_quick_tunnel_args(local_port, &protocol);

    let mut child = cloudflared_command(&binary)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start quick tunnel: {}", e))?;

    let pid = child.id();
    // Take stderr now, before the child is moved into storage — cloudflared prints
    // the public trycloudflare.com URL to stderr shortly after starting.
    let stderr = child.stderr.take();

    {
        let mut procs = manager.processes.lock().unwrap();
        procs.insert("quick-tunnel".to_string(), child);
    }

    {
        let mut statuses = manager.statuses.lock().unwrap();
        statuses.insert("quick-tunnel".to_string(), TunnelStatus {
            name: "quick-tunnel".to_string(),
            running: true,
            pid: Some(pid),
            started_at: Some(chrono::Utc::now()),
            public_url: None,
            services: vec![],
            error: None,
            bytes_in: 0,
            bytes_out: 0,
        });
    }

    // Watch cloudflared's output on a background thread. As soon as it prints the
    // public trycloudflare.com URL, store it so the UI's polling picks it up.
    if let Some(stderr) = stderr {
        let app_handle = app.clone();
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                if let Some(start) = line.find("https://") {
                    if line[start..].contains(".trycloudflare.com") {
                        let end = line[start..]
                            .find(|c: char| c.is_whitespace() || c == '|')
                            .map(|i| start + i)
                            .unwrap_or(line.len());
                        let url = line[start..end].trim().to_string();

                        let manager = app_handle.state::<CloudflaredManager>();
                        let mut statuses = manager.statuses.lock().unwrap();
                        if let Some(status) = statuses.get_mut("quick-tunnel") {
                            status.public_url = Some(url);
                        }
                        break;
                    }
                }
            }
        });
    }

    Ok(())
}

/// Stop the quick tunnel
#[tauri::command]
pub fn stop_quick_tunnel(
    manager: State<'_, CloudflaredManager>,
) -> Result<(), String> {
    manager.stop_tunnel("quick-tunnel")
}

/// Get app version
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Open a URL in the default browser
#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    app.shell()
        .open(url, None)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

/// Update app settings (auto-start, minimize to tray, etc.)
#[tauri::command]
pub fn update_settings(
    auto_start: bool,
    minimize_to_tray: bool,
) -> Result<(), String> {
    let mut config = load_config();
    config.auto_start = auto_start;
    config.minimize_to_tray = minimize_to_tray;
    save_config(&config)
}
