import { useState } from "react";
import { useTunnelStore } from "./hooks/useTunnelStore";
import { TunnelCard } from "./components/TunnelCard";
import { CreateTunnelModal } from "./components/CreateTunnelModal";
import { InstallBanner } from "./components/InstallBanner";
import { QuickTunnel } from "./components/QuickTunnel";
import { api } from "./utils/api";

export default function App() {
  const {
    config,
    statuses,
    cloudflaredInstalled,
    loading,
    error,
    refreshConfig,
    refreshStatuses,
    checkInstalled,
    setError,
  } = useTunnelStore();

  const [showSettings, setShowSettings] = useState(false);
  const [activeTab, setActiveTab] = useState<"tunnels" | "quick">("tunnels");

  const handleRefresh = () => {
    refreshConfig();
    refreshStatuses();
  };

  const getStatusFor = (tunnelName: string) =>
    statuses.find((s) => s.name === tunnelName);

  if (loading) {
    return <div className="loading-screen"><div className="spinner" /><p>Loading TunnelForge...</p></div>;
  }

  return (
    <div className="app">
      {/* Header */}
      <header className="app-header">
        <div className="logo">
          <span className="logo-icon">⛏</span>
          <span className="logo-text">TunnelForge</span>
        </div>
        <div className="header-actions">
          <span className={`install-status ${cloudflaredInstalled ? "ok" : "missing"}`}>
            {cloudflaredInstalled ? "✓ cloudflared ready" : "⚠ cloudflared missing"}
          </span>
          <button className="btn btn-ghost btn-small" onClick={() => setShowSettings(!showSettings)}>
            ⚙ Settings
          </button>
        </div>
      </header>

      {/* Error banner */}
      {error && (
        <div className="error-banner" onClick={() => setError(null)}>
          {error} ✕
        </div>
      )}

      {/* Install banner */}
      {!cloudflaredInstalled && <InstallBanner onInstalled={checkInstalled} />}

      {/* Settings panel */}
      {showSettings && config && (
        <SettingsPanel
          config={config}
          onSave={(autoStart, minimizeToTray) => {
            api.updateSettings(autoStart, minimizeToTray).then(refreshConfig);
            setShowSettings(false);
          }}
        />
      )}

      {/* Tabs */}
      <div className="tabs">
        <button
          className={`tab ${activeTab === "tunnels" ? "active" : ""}`}
          onClick={() => setActiveTab("tunnels")}
        >
          My Tunnels {config && config.tunnels.length > 0 && `(${config.tunnels.length})`}
        </button>
        <button
          className={`tab ${activeTab === "quick" ? "active" : ""}`}
          onClick={() => setActiveTab("quick")}
        >
          ⚡ Quick Tunnel
        </button>
      </div>

      {/* Content */}
      <main className="main-content">
        {activeTab === "tunnels" && (
          <>
            {config && config.tunnels.length === 0 ? (
              <div className="empty-state-screen">
                <h2>Welcome to TunnelForge</h2>
                <p>Expose your local services to the internet — no VPS, no port forwarding, no CLI required.</p>
                <CreateTunnelModal onCreated={handleRefresh} />
                <div className="getting-started">
                  <h3>Quick Start Guide</h3>
                  <ol>
                    <li>Create a free Cloudflare account at <a href="#" onClick={(e) => { e.preventDefault(); api.openUrl("https://dash.cloudflare.com/sign-up"); }}>dash.cloudflare.com</a></li>
                    <li>Go to Zero Trust → Networks → Tunnels → Create a tunnel</li>
                    <li>Copy the tunnel token</li>
                    <li>Click "Create Tunnel" above and paste your token</li>
                    <li>Add services (your local ports) to expose them publicly</li>
                    <li>Hit Start — your service is now live on the internet!</li>
                  </ol>
                </div>
              </div>
            ) : (
              <div className="tunnel-list">
                {config?.tunnels.map((tunnel) => (
                  <TunnelCard
                    key={tunnel.id}
                    tunnel={tunnel}
                    status={getStatusFor(tunnel.name)}
                    onRefresh={handleRefresh}
                  />
                ))}
                <CreateTunnelModal onCreated={handleRefresh} />
              </div>
            )}
          </>
        )}

        {activeTab === "quick" && (
          <QuickTunnel onStarted={refreshStatuses} />
        )}
      </main>

      {/* Footer */}
      <footer className="app-footer">
        <span>TunnelForge v0.1.0 • Powered by Cloudflare Tunnel</span>
        <a href="#" onClick={(e) => { e.preventDefault(); api.openUrl("https://github.com/tunnelforge/tunnelforge"); }}>Docs</a>
      </footer>
    </div>
  );
}

function SettingsPanel({ config, onSave }: { config: any; onSave: (a: boolean, b: boolean) => void }) {
  const [autoStart, setAutoStart] = useState(config.auto_start);
  const [minimizeToTray, setMinimizeToTray] = useState(config.minimize_to_tray);

  return (
    <div className="settings-panel">
      <h3>Settings</h3>
      <label className="checkbox-label">
        <input type="checkbox" checked={autoStart} onChange={(e) => setAutoStart(e.target.checked)} />
        Auto-start tunnels when TunnelForge launches
      </label>
      <label className="checkbox-label">
        <input type="checkbox" checked={minimizeToTray} onChange={(e) => setMinimizeToTray(e.target.checked)} />
        Minimize to system tray on close
      </label>
      <button className="btn btn-primary" onClick={() => onSave(autoStart, minimizeToTray)}>Save Settings</button>
    </div>
  );
}
