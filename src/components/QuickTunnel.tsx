import { useState } from "react";
import { api } from "../utils/api";

interface Props {
  onStarted: () => void;
}

export function QuickTunnel({ onStarted }: Props) {
  const [port, setPort] = useState(8080);
  const [protocol, setProtocol] = useState("http");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  const handleStart = async () => {
    setBusy(true);
    setError(null);
    try {
      await api.startQuickTunnel(port, protocol);
      setRunning(true);
      onStarted();
    } catch (e: any) {
      setError(e.toString());
    }
    setBusy(false);
  };

  const handleStop = async () => {
    setBusy(true);
    try {
      await api.stopQuickTunnel();
      setRunning(false);
      onStarted();
    } catch (e: any) {
      setError(e.toString());
    }
    setBusy(false);
  };

  return (
    <div className="quick-tunnel">
      <div className="quick-tunnel-header">
        <h3>⚡ Quick Tunnel</h3>
        <span className="badge badge-free">No account needed</span>
      </div>
      <p>Instantly expose a local port to the internet. Cloudflare generates a random trycloudflare.com URL.</p>

      <div className="quick-tunnel-form">
        <select value={protocol} onChange={(e) => setProtocol(e.target.value)} disabled={running}>
          <option value="http">HTTP</option>
          <option value="https">HTTPS</option>
          <option value="tcp">TCP</option>
        </select>
        <input
          type="number"
          value={port}
          onChange={(e) => setPort(parseInt(e.target.value) || 0)}
          placeholder="Port"
          disabled={running}
        />
        {running ? (
          <button className="btn btn-stop" onClick={handleStop} disabled={busy}>
            {busy ? "Stopping..." : "Stop"}
          </button>
        ) : (
          <button className="btn btn-primary" onClick={handleStart} disabled={busy}>
            {busy ? "Starting..." : "Start Quick Tunnel"}
          </button>
        )}
      </div>

      {error && <div className="form-error">{error}</div>}
      {running && (
        <p className="quick-tunnel-hint">
          A random public URL has been generated. Check the system tray or cloudflared output for your trycloudflare.com link.
        </p>
      )}
    </div>
  );
}
