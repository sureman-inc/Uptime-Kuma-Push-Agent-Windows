#![windows_subsystem = "windows"]

mod config;
mod autostart;
mod monitor;

use std::{env, fs, sync::{Arc, Mutex}};
use log::{info, error};
use serde::Deserialize;

// Imports for tray and webview window
use tray_icon::{
    Icon,
    menu::{Menu, MenuItem, MenuEvent},
    TrayIconBuilder,
};
use tao::{
    dpi::LogicalSize,
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

use crate::config::{Config, CheckType, load_config, save_config, get_app_data_dir, get_config_path};
use crate::autostart::handle_autostart_logic;
use crate::monitor::{run_monitoring_loop, test_target};

#[derive(Debug)]
enum UserEvent {
    TrayMenuEvent(MenuEvent),
    SettingsSaved(Config),
    TestResult { id: usize, success: bool, latency: u128, message: String },
    CloseSettings,
    OpenSettings,
}

#[derive(Debug, Deserialize)]
struct DeviceTestPayload {
    id: usize,
    target: String,
    check_type: CheckType,
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct SaveConfigPayload {
    config: Config,
}

fn setup_logging(enable_file_logging: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    if enable_file_logging {
        let logs_dir = get_app_data_dir().join("logs");
        if !logs_dir.exists() {
            fs::create_dir_all(&logs_dir)?;
        }
        let log_file = logs_dir.join("agent.log");
        dispatch = dispatch.chain(fern::log_file(log_file)?);
    }

    dispatch.apply()?;
    log_panics::init();
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Application Fatal Error: {}", e);
        println!("\nPress Enter to exit...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        std::process::exit(1);
    }
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let config_path = get_config_path();
    let is_first_run = !config_path.exists();

    let initial_config = load_config()?;
    let _ = setup_logging(initial_config.enable_logging);

    info!("=======================================================");
    info!("Starting Uptime Kuma Push Agent (Multi-Device) v5.0.0");
    info!("=======================================================");
    if is_first_run {
        info!("First run detected: opening Settings wizard.");
    }

    let config_shared = Arc::new(Mutex::new(initial_config.clone()));

    // System Tray Menu
    let settings_item = MenuItem::new("Settings", true, None);
    let quit_item = MenuItem::new("Exit", true, None);
    let tray_menu = Menu::new();
    let _ = tray_menu.append(&settings_item);
    let _ = tray_menu.append(&quit_item);

    let mut icon_path = env::current_exe()?
        .parent()
        .unwrap()
        .join("icon.ico");

    if !icon_path.exists() {
        let alt_path = env::current_dir()?.join("icon.ico");
        if alt_path.exists() {
            icon_path = alt_path;
        }
    }

    let icon = match Icon::from_resource(1, None) {
        Ok(i) => i,
        Err(_) => {
            if icon_path.exists() {
                Icon::from_path(icon_path, None).unwrap_or_else(|_| Icon::from_rgba(vec![0; 32 * 32 * 4], 32, 32).unwrap())
            } else {
                Icon::from_rgba(vec![0; 32 * 32 * 4], 32, 32).unwrap()
            }
        }
    };

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_icon(icon)
        .with_tooltip("Uptime Kuma Agent - Branch Monitor")
        .build()?;

    // Spawn Background Multi-Device Monitoring Loop
    let config_for_loop = Arc::clone(&config_shared);
    tokio::spawn(async move {
        run_monitoring_loop(config_for_loop).await;
    });

    // Tao Event Loop
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Tray Menu events proxy
    let menu_channel = MenuEvent::receiver();
    let proxy_menu = proxy.clone();
    std::thread::spawn(move || {
        while let Ok(event) = menu_channel.recv() {
            let _ = proxy_menu.send_event(UserEvent::TrayMenuEvent(event));
        }
    });

    let mut settings_window = None;

    if is_first_run {
        let _ = proxy.send_event(UserEvent::OpenSettings);
    }

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            tao::event::Event::UserEvent(user_event) => {
                match user_event {
                    UserEvent::TrayMenuEvent(menu_event) => {
                        if menu_event.id == quit_item.id() {
                            *control_flow = ControlFlow::Exit;
                        } else if menu_event.id == settings_item.id() {
                            let _ = proxy.send_event(UserEvent::OpenSettings);
                        }
                    }
                    UserEvent::OpenSettings => {
                        if settings_window.is_none() {
                            let window = WindowBuilder::new()
                                .with_title("Uptime Kuma Agent - Settings")
                                .with_inner_size(LogicalSize::new(760.0, 640.0))
                                .with_min_inner_size(LogicalSize::new(580.0, 480.0))
                                .with_resizable(true)
                                .build(event_loop)
                                .unwrap();

                            let html_content = include_str!("settings.html");
                            let proxy_webview = proxy.clone();

                            let webview = WebViewBuilder::new()
                                .with_ipc_handler(move |request| {
                                    let msg = request.body();
                                    
                                    if msg == "request_config" {
                                        // Request handled via evaluate_script below
                                    } else if msg == "close_window" {
                                        let _ = proxy_webview.send_event(UserEvent::CloseSettings);
                                    } else if let Ok(save_payload) = serde_json::from_str::<SaveConfigPayload>(&msg) {
                                        let _ = proxy_webview.send_event(UserEvent::SettingsSaved(save_payload.config));
                                    } else if let Ok(test_payload) = serde_json::from_str::<DeviceTestPayload>(&msg) {
                                        let proxy_test = proxy_webview.clone();
                                        tokio::spawn(async move {
                                            let (success, latency, message) = test_target(
                                                &test_payload.target,
                                                &test_payload.check_type,
                                                test_payload.port,
                                                2000,
                                            ).await;
                                            let _ = proxy_test.send_event(UserEvent::TestResult {
                                                id: test_payload.id,
                                                success,
                                                latency,
                                                message,
                                            });
                                        });
                                    } else if let Ok(direct_config) = serde_json::from_str::<Config>(&msg) {
                                        let _ = proxy_webview.send_event(UserEvent::SettingsSaved(direct_config));
                                    }
                                })
                                .with_html(html_content)
                                .build(&window)
                                .unwrap();

                            // Send current config to the Webview UI
                            let current_cfg = config_shared.lock().unwrap().clone();
                            if let Ok(config_json) = serde_json::to_string(&serde_json::json!({
                                "type": "config",
                                "data": current_cfg
                            })) {
                                let _ = webview.evaluate_script(&format!("window.postMessage('{}', '*')", config_json.replace('\\', "\\\\").replace('\'', "\\'")));
                            }

                            settings_window = Some((window, webview));
                        } else if let Some((window, _)) = &settings_window {
                            window.set_focus();
                        }
                    }
                    UserEvent::TestResult { id, success, latency, message } => {
                        if let Some((_, webview)) = &settings_window {
                            if let Ok(result_json) = serde_json::to_string(&serde_json::json!({
                                "type": "test_result",
                                "id": id,
                                "success": success,
                                "latency": latency,
                                "message": message
                            })) {
                                let _ = webview.evaluate_script(&format!("window.postMessage('{}', '*')", result_json.replace('\\', "\\\\").replace('\'', "\\'")));
                            }
                        }
                    }
                    UserEvent::SettingsSaved(new_cfg) => {
                        info!("Saving new configuration for branch '{}'...", new_cfg.branch_name);
                        let _ = handle_autostart_logic(&new_cfg);
                        if let Err(e) = save_config(&new_cfg) {
                            error!("Failed to save configuration: {}", e);
                        } else {
                            {
                                let mut config = config_shared.lock().unwrap();
                                *config = new_cfg;
                            }
                            if let Some((_, webview)) = &settings_window {
                                let _ = webview.evaluate_script("window.postMessage(JSON.stringify({type: 'success'}), '*')");
                            }
                        }
                    }
                    UserEvent::CloseSettings => {
                        settings_window = None;
                    }
                }
            }
            tao::event::Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => {
                settings_window = None;
            }
            _ => (),
        }
    });
}