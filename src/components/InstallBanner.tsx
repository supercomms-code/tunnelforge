import { useState } from "react";
import { api } from "../utils/api";

interface Props {
  onInstalled: () => void;
}

export function InstallBanner({ onInstalled }: Props) {
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleInstall = async () => {
    setInstalling(true);
    setError(null);
    try {
      await api.install();
      onInstalled();
    } catch (e: any) {
      setError(e.toString());
    }
    setInstalling(false);
  };

  return (
    <div className="install-banner">
      <div className="install-banner-content">
        <div>
          <h3>cloudflared not detected</h3>
          <p>
            TunnelForge uses Cloudflare's free tunnel service to expose your local services.
            You need the <code>cloudflared</code> binary installed. Click below to download and install it automatically.
          </p>
          {error && <div className="install-error">{error}</div>}
        </div>
        <button
          className="btn btn-primary btn-large"
          onClick={handleInstall}
          disabled={installing}
        >
          {installing ? "Downloading..." : "Install cloudflared"}
        </button>
      </div>
    </div>
  );
}
