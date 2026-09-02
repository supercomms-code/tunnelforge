# TunnelForge

**Expose local services to the internet without a VPS.**

TunnelForge is a cross-platform desktop application (Windows, macOS, Linux) that makes it dead simple to expose your local network services to the public internet using Cloudflare Tunnel — no VPS to rent, no port forwarding, no CLI required.

## Why?

If you're behind CGNAT (Starlink, 4G/5G, many ISPs), you can't host anything from your local network. Traditional solutions require either:
- Renting a VPS ($5+/month) and manually configuring WireGuard/frp/rathole
- Using cloudflared CLI (powerful but requires terminal expertise)
- Paying ngrok $10-99+/month for managed tunnels

TunnelForge wraps Cloudflare Tunnel in a clean desktop GUI. Create tunnels, add services, hit start — done.

## Features

- **Cross-platform**: Windows, macOS (Intel + Apple Silicon), Linux
- **Zero-config quick tunnels**: Instant random URL via trycloudflare.com — no account needed
- **Named tunnels**: Bring your own Cloudflare account for persistent custom domains
- **System tray integration**: Tunnels run in the background, check status at a glance
- **Auto-start**: Configured tunnels can start automatically when the app launches
- **Multi-service**: One tunnel can expose multiple local services on different subdomains
- **Protocol support**: HTTP, HTTPS, TCP, UDP, SSH, RDP
- **Auto-install**: Downloads and installs cloudflared binary automatically
- **100% free**: Uses Cloudflare's free tier — no subscription required

## Quick Start

1. Download TunnelForge for your platform from [Releases](../../releases)
2. Launch the app — it lives in your system tray
3. Go to the **Quick Tunnel** tab
4. Enter a local port (e.g., `8080`) and click **Start**
5. You'll get a public `trycloudflare.com` URL — share it with anyone

## Named Tunnels (Custom Domains)

For persistent URLs with your own domain:

1. Create a free Cloudflare account at [dash.cloudflare.com](https://dash.cloudflare.com)
2. Go to **Zero Trust → Networks → Tunnels → Create a tunnel**
3. Copy the tunnel token
4. In TunnelForge, click **Create Tunnel**, paste your token
5. Add services mapping your domain to local ports (e.g., `app.yourdomain.com` → `localhost:3000`)
6. Hit **Start** — your service is live

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (1.77.2+)
- [Node.js](https://nodejs.org/) (18+)
- Platform-specific dependencies:
  - **Linux**: `libwebkit2gtk-4.1-dev librsvg2-dev patchelf`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio C++ Build Tools

```bash
npm install
npm run tauri dev    # Development mode
npm run tauri build  # Production build
```

## Architecture

```
TunnelForge Desktop App (Tauri v2 + React)
├── Rust backend (src-tauri/)
│   ├── cloudflared.rs  — Binary lifecycle management
│   ├── config.rs       — YAML config generation + app settings
│   └── commands.rs     — Tauri IPC bridge
├── React frontend (src/)
│   ├── App.tsx          — Main UI with tabs and tunnel list
│   ├── components/      — TunnelCard, CreateTunnelModal, QuickTunnel, InstallBanner
│   └── hooks/           — useTunnelStore (state management)
└── Cloudflare Tunnel (cloudflared) — The actual tunnel daemon
```

TunnelForge uses the **BYO Cloudflare Account** model — you bring your own free Cloudflare account and token. The app is a management GUI, not a hosted service. This is 100% compliant with Cloudflare's Terms of Service.

## Tech Stack

- **Tauri v2** — Cross-platform desktop framework (Rust + web frontend)
- **React + TypeScript** — Frontend UI
- **cloudflared** — Cloudflare's open-source tunnel daemon (Apache 2.0)
- **serde_yaml** — Config file generation
- **GitHub Actions** — CI/CD for auto-building installers

## License

MIT

---

Made with ⛏ by Jackson Ward
