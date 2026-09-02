// main.rs — Entry point for TunnelForge Tauri application

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cloudflared;
mod config;
mod commands;

use cloudflared::CloudflaredManager;
use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

fn main() {
    env_logger::init();

    log::info!("TunnelForge starting up...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(CloudflaredManager::new())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent closing — hide to tray instead
                let config = config::load_config();
                if config.minimize_to_tray {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            // Build system tray menu
            let show = MenuItem::with_id(app, "show", "Show TunnelForge", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // Set up tray
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap())
                .menu(&menu)
                .tooltip("TunnelForge — Tunnel Manager")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            // Stop all tunnels before quitting
                            let manager = app.state::<CloudflaredManager>();
                            manager.stop_all();
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Auto-start tunnels if configured
            let app_config = config::load_config();
            if app_config.auto_start {
                log::info!("Auto-starting configured tunnels...");
                let manager = app.state::<CloudflaredManager>();
                for tunnel in &app_config.tunnels {
                    if tunnel.auto_start {
                        if let Some(token) = &tunnel.token {
                            let _ = manager.start_tunnel_with_token(&tunnel.name, token);
                        } else if !tunnel.services.is_empty() {
                            if let Ok(config_path) = config::save_tunnel_config(tunnel) {
                                let _ = manager.start_tunnel(&tunnel.name, &config_path);
                            }
                        }
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_cloudflared_installed,
            commands::get_cloudflared_download_url,
            commands::install_cloudflared,
            commands::get_config,
            commands::save_app_config,
            commands::create_tunnel,
            commands::delete_tunnel,
            commands::add_service,
            commands::remove_service,
            commands::start_tunnel,
            commands::stop_tunnel,
            commands::get_tunnel_status,
            commands::get_all_tunnel_statuses,
            commands::start_quick_tunnel,
            commands::stop_quick_tunnel,
            commands::get_app_version,
            commands::open_url,
            commands::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running TunnelForge");
}
