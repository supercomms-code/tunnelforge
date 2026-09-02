// Type definitions shared between frontend and backend

export interface AppConfig {
  tunnels: TunnelDefinition[];
  cloudflare_token: string | null;
  auto_start: boolean;
  minimize_to_tray: boolean;
  version: string;
}

export interface TunnelDefinition {
  id: string;
  name: string;
  token: string | null;
  credentials_file: string | null;
  services: ServiceMapping[];
  auto_start: boolean;
}

export interface ServiceMapping {
  id: string;
  hostname: string;
  protocol: string;
  local_host: string;
  local_port: number;
  description: string;
}

export interface TunnelStatus {
  name: string;
  running: boolean;
  pid: number | null;
  started_at: string | null;
  public_url: string | null;
  services: ServiceEntry[];
  error: string | null;
  bytes_in: number;
  bytes_out: number;
}

export interface ServiceEntry {
  hostname: string;
  service: string;
  protocol: string;
}
