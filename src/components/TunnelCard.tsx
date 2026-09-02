import { useState } from "react";
import { api } from "../utils/api";
import type { TunnelDefinition, TunnelStatus } from "../types";

interface Props {
  tunnel: TunnelDefinition;
  status: TunnelStatus | undefined;
  onRefresh: () => void;
}

const protocolIcons: Record<string, string> = {
  http: "🌐",
  https: "🔒",
  tcp: "🔌",
  udp: "📡",
  ssh: "💻",
  rdp: "🖥️",
};

export function TunnelCard({ tunnel, status, onRefresh }: Props) {
  const [expanded, setExpanded] = useState(false);
  const [showAddService, setShowAddService] = useState(false);
  const [newService, setNewService] = useState({
    hostname: "",
    protocol: "http",
    localHost: "localhost",
    localPort: 8080,
    description: "",
  });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isRunning = status?.running ?? false;

  const handleStart = async () => {
    setBusy(true);
    setError(null);
    try {
      await api.startTunnel(tunnel.id);
      onRefresh();
    } catch (e: any) {
      setError(e.toString());
    }
    setBusy(false);
  };

  const handleStop = async () => {
    setBusy(true);
    setError(null);
    try {
      await api.stopTunnel(tunnel.id);
      onRefresh();
    } catch (e: any) {
      setError(e.toString());
    }
    setBusy(false);
  };

  const handleDelete = async () => {
    if (!confirm(`Delete tunnel "${tunnel.name}"? This cannot be undone.`)) return;
    try {
      await api.deleteTunnel(tunnel.id);
      onRefresh();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleAddService = async () => {
    setBusy(true);
    setError(null);
    try {
      await api.addService(
        tunnel.id,
        newService.hostname,
        newService.protocol,
        newService.localHost,
        newService.localPort,
        newService.description
      );
      setShowAddService(false);
      setNewService({ hostname: "", protocol: "http", localHost: "localhost", localPort: 8080, description: "" });
      onRefresh();
    } catch (e: any) {
      setError(e.toString());
    }
    setBusy(false);
  };

  const handleRemoveService = async (serviceId: string) => {
    try {
      await api.removeService(tunnel.id, serviceId);
      onRefresh();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const uptime = status?.started_at
    ? Math.floor((Date.now() - new Date(status.started_at).getTime()) / 1000)
    : 0;

  return (
    <div className={`tunnel-card ${isRunning ? "running" : "stopped"}`}>
      <div className="tunnel-header" onClick={() => setExpanded(!expanded)}>
        <div className="tunnel-status-dot" />
        <div className="tunnel-info">
          <h3>{tunnel.name}</h3>
          <span className="tunnel-meta">
            {tunnel.services.length} service{tunnel.services.length !== 1 ? "s" : ""} •{" "}
            {isRunning ? `Running ${formatUptime(uptime)}` : "Stopped"}
            {tunnel.token && " • Token-managed"}
          </span>
        </div>
        <div className="tunnel-actions">
          {isRunning ? (
            <button className="btn btn-stop" onClick={(e) => { e.stopPropagation(); handleStop(); }} disabled={busy}>
              Stop
            </button>
          ) : (
            <button className="btn btn-start" onClick={(e) => { e.stopPropagation(); handleStart(); }} disabled={busy}>
              Start
            </button>
          )}
          <button className="btn btn-ghost" onClick={(e) => { e.stopPropagation(); setExpanded(!expanded); }}>
            {expanded ? "▼" : "▶"}
          </button>
        </div>
      </div>

      {error && <div className="tunnel-error">{error}</div>}

      {expanded && (
        <div className="tunnel-body">
          {tunnel.services.length === 0 && !tunnel.token && (
            <p className="empty-state">No services configured. Add one below to expose a local port to the internet.</p>
          )}

          {tunnel.services.map((service) => (
            <div key={service.id} className="service-row">
              <span className="service-icon">{protocolIcons[service.protocol] || "🌐"}</span>
              <div className="service-details">
                <span className="service-hostname">{service.hostname || "—"}</span>
                <span className="service-target">→ {service.protocol}://{service.local_host}:{service.local_port}</span>
                {service.description && <span className="service-desc">{service.description}</span>}
              </div>
              {!tunnel.token && (
                <button className="btn btn-small btn-danger" onClick={() => handleRemoveService(service.id)}>
                  Remove
                </button>
              )}
            </div>
          ))}

          {!tunnel.token && (
            <>
              {showAddService ? (
                <div className="add-service-form">
                  <div className="form-row">
                    <input
                      type="text"
                      placeholder="hostname (e.g. app.yourdomain.com)"
                      value={newService.hostname}
                      onChange={(e) => setNewService({ ...newService, hostname: e.target.value })}
                    />
                  </div>
                  <div className="form-row">
                    <select
                      value={newService.protocol}
                      onChange={(e) => setNewService({ ...newService, protocol: e.target.value })}
                    >
                      <option value="http">HTTP</option>
                      <option value="https">HTTPS</option>
                      <option value="tcp">TCP</option>
                      <option value="udp">UDP</option>
                      <option value="ssh">SSH</option>
                      <option value="rdp">RDP</option>
                    </select>
                    <input
                      type="text"
                      placeholder="host"
                      value={newService.localHost}
                      onChange={(e) => setNewService({ ...newService, localHost: e.target.value })}
                    />
                    <input
                      type="number"
                      placeholder="port"
                      value={newService.localPort}
                      onChange={(e) => setNewService({ ...newService, localPort: parseInt(e.target.value) || 0 })}
                    />
                  </div>
                  <input
                    type="from_text"
                    placeholder="description (optional)"
                    value={newService.description}
                    onChange={(e) => setNewService({ ...newService, description: e.target.value })}
                    style={{ width: "100%", marginBottom: "8px" }}
                  />
                  <div className="form-actions">
                    <button className="btn btn-primary" onClick={handleAddService} disabled={busy}>
                      {busy ? "Adding..." : "Add Service"}
                    </button>
                    <button className="btn btn-ghost" onClick={() => setShowAddService(false)}>
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                <button className="btn btn-secondary" onClick={() => setShowAddService(true)}>
                  + Add Service
                </button>
              )}
            </>
          )}

          <button className="btn btn-small btn-danger-ghost" onClick={handleDelete} style={{ marginTop: "12px" }}>
            Delete Tunnel
          </button>
        </div>
      )}
    </div>
  );
}

function formatUptime(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
}
