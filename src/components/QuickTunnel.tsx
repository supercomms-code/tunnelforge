import { useEffect, useRef, useState } from "react";
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
  const [publicUrl, setPublicUrl] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const pollRef = useRef<number | null>(null);

  // While the tunnel is running but we don't have a URL yet, keep checking —
  // cloudflared usually prints it within a second or two of starting.
  useEffect(() => {
    if (running && !publicUrl) {
      pollRef.current = window.setInterval(async () => {
        try {
          const status = await api.getTunnelStatus("quick-tunnel");
          if (status?.public_url) {
            setPublicUrl(status.public_url);
            if (pollRef.current) window.clearInterval(pollRef.current);
          }
        } catch {
          // ignore transient polling errors
        }
      }, 1000);
    }
    return () => {
      if (pollRef.current) window.clearInterval(pollRef.current);
    };
  }, [running, publicUrl]);

  const handleStart = async () => {
    setBusy(true);
    setError(null);
    setPublicUrl(null);
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
      setPublicUrl(null);
      onStarted();
    } catch (e: any) {
      setError(e.toString());
    }
    setBusy(false);
  };

  const handleCopy = async () => {
    if (!publicUrl) return;
    await navigator.clipboard.writeText(publicUrl);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
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

      {running && !publicUrl && (
        <p className="quick-tunnel-hint">Waiting for Cloudflare to assign a public address…</p>
      )}

      {running && publicUrl && (
        <div className="quick-tunnel-result">
          <label>Your public address</label>
          <div className="quick-tunnel-url-row">
            <code>{publicUrl}</code>
            <button className="btn btn-secondary" onClick={handleCopy}>
              {copied ? "Copied" : "Copy"}
            </button>
            <button className="btn btn-secondary" onClick={() => api.openUrl(publicUrl)}>
              Open
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
