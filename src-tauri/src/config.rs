// config.rs — Generates and manages cloudflared config.yml files
// Also manages the app's own configuration (tunnel list, settings)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

/// A single ingress rule mapping a hostname to a local service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRule {
    pub hostname: String,
    pub service: String,
    pub originRequest: Option<OriginRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub httpHostHeader: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub originServerName: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectTimeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noTLSVerify: Option<bool>,
}

/// A tunnel configuration with all its services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub name: String,
    pub credentials_file: Option<String>,
    pub token: Option<String>,
    pub ingress: Vec<IngressRule>,
    pub warp_routing: Option<bool>,
    pub metrics: Option<String>,
}

/// App-wide settings and tunnel store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub tunnels: Vec<TunnelDefinition>,
    pub cloudflare_token: Option<String>,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelDefinition {
    pub id: String,
    pub name: String,
    pub token: Option<String>,
    pub credentials_file: Option<String>,
    pub services: Vec<ServiceMapping>,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMapping {
    pub id: String,
    pub hostname: String,
    pub protocol: String,    // http, https, tcp, udp, ssh, rdp
    pub local_host: String,  // localhost, 127.0.0.1, 192.168.1.x
    pub local_port: u16,
    pub description: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tunnels: vec![],
            cloudflare_token: None,
            auto_start: false,
            minimize_to_tray: true,
            version: "0.1.0".to_string(),
        }
    }
}

/// Get the app config directory
pub fn get_config_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    let path = dir.join("tunnelforge");
    let _ = std::fs::create_dir_all(&path);
    path
}

/// Get the path to the app config file
pub fn get_config_file_path() -> PathBuf {
    get_config_dir().join("config.json")
}

/// Get the path for a specific tunnel's cloudflared config.yml
pub fn get_tunnel_config_path(tunnel_name: &str) -> PathBuf {
    let dir = get_config_dir().join("tunnels");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(format!("{}.yml", sanitize_name(tunnel_name)))
}

/// Load the app config from disk
pub fn load_config() -> AppConfig {
    let path = get_config_file_path();
    if !path.exists() {
        let config = AppConfig::default();
        let _ = save_config(&config);
        return config;
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_default()
        }
        Err(_) => AppConfig::default(),
    }
}

/// Save the app config to disk
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_file_path();
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    log::info!("Config saved to {:?}", path);
    Ok(())
}

/// Generate a cloudflared config.yml for a tunnel definition
pub fn generate_cloudflared_config(tunnel: &TunnelDefinition) -> String {
    let mut ingress_rules: Vec<HashMap<String, serde_yaml::Value>> = vec![];
    
    for service in &tunnel.services {
        let mut rule = HashMap::new();
        rule.insert("hostname".to_string(), serde_yaml::Value::String(service.hostname.clone()));
        
        let service_url = match service.protocol.as_str() {
            "tcp" => format!("tcp://{}:{}", service.local_host, service.local_port),
            "udp" => format!("udp://{}:{}", service.local_host, service.local_port),
            "ssh" => format!("ssh://localhost:{}", service.local_port),
            "rdp" => format!("tcp://{}:{}", service.local_host, service.local_port),
            "https" => format!("https://{}:{}", service.local_host, service.local_port),
            _ => format!("http://{}:{}", service.local_host, service.local_port),
        };
        rule.insert("service".to_string(), serde_yaml::Value::String(service_url));
        ingress_rules.push(rule);
    }

    // Catch-all rule (required by cloudflared)
    let mut catch_all = HashMap::new();
    catch_all.insert("service".to_string(), serde_yaml::Value::String("http_status:404".to_string()));
    ingress_rules.push(catch_all);

    let mut config = serde_yaml::Mapping::new();
    
    if let Some(token) = &tunnel.token {
        // Token-based tunnels are managed remotely — no local config needed
        // But we still generate it for reference
    }
    
    if let Some(creds) = &tunnel.credentials_file {
        config.insert(
            serde_yaml::Value::String("tunnel".to_string()),
            serde_yaml::Value::String(tunnel.name.clone()),
        );
        config.insert(
            serde_yaml::Value::String("credentials-file".to_string()),
            serde_yaml::Value::String(creds.clone()),
        );
    }

    let ingress_yaml: Vec<serde_yaml::Value> = ingress_rules
        .into_iter()
        .map(|m| serde_yaml::to_value(m).unwrap())
        .collect();

    config.insert(
        serde_yaml::Value::String("ingress".to_string()),
        serde_yaml::Value::Sequence(ingress_yaml),
    );

    // Add metrics for local monitoring
    config.insert(
        serde_yaml::Value::String("metrics".to_string()),
        serde_yaml::Value::String("127.0.0.1:35117".to_string()),
    );

    let doc = serde_yaml::to_string(&config)
        .map_err(|e| format!("YAML serialization failed: {}", e))
        .unwrap_or_else(|e| {
            log::error!("Failed to generate config YAML: {}", e);
            "# Error generating config".to_string()
        });

    doc
}

/// Save a tunnel's cloudflared config.yml to disk
pub fn save_tunnel_config(tunnel: &TunnelDefinition) -> Result<PathBuf, String> {
    let yaml = generate_cloudflared_config(tunnel);
    let path = get_tunnel_config_path(&tunnel.name);
    std::fs::write(&path, yaml)
        .map_err(|e| format!("Failed to write tunnel config: {}", e))?;
    log::info!("Tunnel config saved to {:?}", path);
    Ok(path)
}

/// Generate a quick tunnel URL (trycloudflare.com — no account needed)
pub fn generate_quick_tunnel_args(local_port: u16, protocol: &str) -> Vec<String> {
    let service = match protocol {
        "tcp" => format!("tcp://localhost:{}", local_port),
        "https" => format!("https://localhost:{}", local_port),
        _ => format!("http://localhost:{}", local_port),
    };
    vec![
        "tunnel".to_string(),
        "--url".to_string(),
        service,
    ]
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
