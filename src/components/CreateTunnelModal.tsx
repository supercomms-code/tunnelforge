import { useState } from "react";
import { api } from "../utils/api";
import type { TunnelDefinition } from "../types";

interface Props {
  onCreated: () => void;
}

export function CreateTunnelModal({ onCreated }: Props) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [token, setToken] = useState("");
  const [autoStart, setAutoStart] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async () => {
    if (!name.trim()) {
      setError("Tunnel name is required");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await api.createTunnel(name.trim(), token.trim() || null, autoStart);
      setOpen(false);
      setName("");
      setToken("");
      setAutoStart(false);
      onCreated();
    } catch (e: any) {
      setError(e.toString());
    }
    setBusy(false);
  };

  if (!open) {
    return (
      <button className="btn btn-primary btn-large" onClick={() => setOpen(true)}>
        + Create Tunnel
      </button>
    );
  }

  return (
    <div className="modal-overlay" onClick={() => setOpen(false)}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>Create New Tunnel</h2>

        <div className="form-group">
          <label>Tunnel Name</label>
          <input
            type="text"
            placeholder="e.g. my-home-server"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="form-group">
          <label>Cloudflare Tunnel Token (Optional)</label>
          <input
            type="text"
            placeholder="Paste token from Cloudflare Zero Trust dashboard"
            value={token}
            onChange={(e) => setToken(e.target.value)}
          />
          <p className="form-hint">
            Leave empty if you want to configure services manually with a config file.
            Get a token from{" "}
            <a href="#" onClick={(e) => { e.preventDefault(); api.openUrl("https://one.dash.cloudflare.com/"); }}>
              Cloudflare Zero Trust
            </a>{" "}
            → Networks → Tunnels → Create Tunnel.
          </p>
        </div>

        <div className="form-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={autoStart}
              onChange={(e) => setAutoStart(e.target.checked)}
            />
            Auto-start when TunnelForge launches
          </label>
        </div>

        {error && <div className="form-error">{error}</div>}

        <div className="modal-actions">
          <button className="btn btn-primary" onClick={handleCreate} disabled={busy}>
            {busy ? "Creating..." : "Create Tunnel"}
          </button>
          <button className="btn btn-ghost" onClick={() => setOpen(false)}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
