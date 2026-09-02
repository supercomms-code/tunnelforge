// API wrapper — all Tauri IPC calls go through here
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, TunnelDefinition, TunnelStatus } from "../types";

export const api = {
  // Cloudflared management
  checkInstalled: () => invoke<boolean>("check_cloudflared_installed"),
  getDownloadUrl: () => invoke<string>("get_cloudflared_download_url"),
  install: () => invoke<string>("install_cloudflared"),

  // Config
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config: AppConfig) => invoke<void>("save_app_config", { config }),
  updateSettings: (autoStart: boolean, minimizeToTray: boolean) =>
    invoke<void>("update_settings", { autoStart, minimizeToTray }),

  // Tunnels
  createTunnel: (name: string, token: string | null, autoStart: boolean) =>
    invoke<TunnelDefinition>("create_tunnel", { name, token, autoStart }),
  deleteTunnel: (tunnelId: string) =>
    invoke<void>("delete_tunnel", { tunnelId }),
  startTunnel: (tunnelId: string) =>
    invoke<void>("start_tunnel", { tunnelId }),
  stopTunnel: (tunnelId: string) =>
    invoke<void>("stop_tunnel", { tunnelId }),

  // Services
  addService: (
    tunnelId: string,
    hostname: string,
    protocol: string,
    localHost: string,
    localPort: number,
    description: string
  ) =>
    invoke<void>("add_service", {
      tunnelId,
      hostname,
      protocol,
      localHost,
      localPort,
      description,
    }),
  removeService: (tunnelId: string, serviceId: string) =>
    invoke<void>("remove_service", { tunnelId, serviceId }),

  // Status
  getTunnelStatus: (tunnelName: string) =>
    invoke<TunnelStatus | null>("get_tunnel_status", { tunnelName }),
  getAllStatuses: () => invoke<TunnelStatus[]>("get_all_tunnel_statuses"),

  // Quick tunnel (no account needed)
  startQuickTunnel: (localPort: number, protocol: string) =>
    invoke<void>("start_quick_tunnel", { localPort, protocol }),
  stopQuickTunnel: () => invoke<void>("stop_quick_tunnel"),

  // Misc
  getAppVersion: () => invoke<string>("get_app_version"),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
};
