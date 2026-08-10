#![windows_subsystem = "windows"]
use std::{env, fs, time::Duration, sync::{Arc, Mutex}, path::PathBuf, os::windows::process::CommandExt};
use reqwest::Client;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use toml;
use chrono::Local;
use log::{info, error, warn};
use winreg::enums::*;
use winreg::RegKey;
const CREATE_NO_WINDOW: u32 = 0x08000000;

// Импорты для трея и окна
use tray_icon::{
    Icon,
    menu::{Menu, MenuItem, MenuEvent},
    TrayIconBuilder,
};
use tao::{
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{WebViewBuilder};

// --- Структура конфигурации ---
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Config {
    kuma_push_url: String,
    #[serde(default = "default_interval")]
    interval_sec: u64,
    #[serde(default)]
    autostart: bool,
    #[serde(default)]
    ping_target: String,
    #[serde(default = "default_enable_logging")]
    pub enable_logging: bool,
}

fn default_enable_logging() -> bool {
    true
}

fn default_interval() -> u64 {
    30
}

#[derive(Debug)]
enum UserEvent {
    TrayMenuEvent(MenuEvent),
    SettingsSaved(Config),
    CloseSettings,
    OpenSettings,
}

fn get_app_data_dir() -> PathBuf {
    if let Ok(app_data) = env::var("APPDATA") {
        let dir = std::path::Path::new(&app_data).join("KumaAgent");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir
    } else {
        env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or_else(|| &std::path::Path::new("."))
            .to_path_buf()
    }
}

fn get_config_path() -> PathBuf {
    get_app_data_dir().join("config.toml")
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

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    info!("Attempting to load configuration from: {:?}", config_path);

    if !config_path.exists() {
        info!("Config file not found, using defaults.");
        return Ok(Config {
            kuma_push_url: String::new(),
            interval_sec: 30,
            autostart: false,
            ping_target: String::new(),
            enable_logging: true,
        });
    }

    let config_content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_content)?;
    info!("Configuration loaded successfully.");
    Ok(config)
}

fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    let toml_string = toml::to_string(config)?;
    fs::write(config_path, toml_string)?;
    info!("Configuration saved to {:?}", get_config_path());
    Ok(())
}

fn handle_autostart_logic(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let (key, _disp) = hkcu.create_subkey(path)?;

    if config.autostart {
        let current_exe = env::current_exe()?;
        let target_dir = get_app_data_dir();
        let target_exe = target_dir.join("kuma_agent.exe");
        let target_config = target_dir.join("config.toml");

        // Если мы еще не в целевой папке, копируем себя туда
        if current_exe != target_exe {
            info!("Installing to permanent location: {:?}", target_dir);
            fs::create_dir_all(&target_dir)?;
            
            // Копируем EXE
            fs::copy(&current_exe, &target_exe)?;
            
            // Копируем конфиг, если он есть
            let current_config = get_config_path();
            if current_config.exists() {
                fs::copy(&current_config, &target_config)?;
            }
        }

        info!("Enabling autostart in Registry pointing to {:?}", target_exe);
        key.set_value("KumaAgent", &target_exe.to_str().unwrap_or_default())?;
    } else {
        info!("Disabling autostart in Registry");
        let _ = key.delete_value("KumaAgent");
    }

    Ok(())
}

// --- Основная логика отправки Heartbeat ---
async fn run_heartbeat_loop(config_arc: Arc<Mutex<Config>>) {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|e| {
            error!("Failed to create HTTP client: {}", e);
            Client::new()
        });

    loop {
        let (url, interval, ping_target) = {
            let config = config_arc.lock().unwrap();
            (config.kuma_push_url.clone(), config.interval_sec, config.ping_target.clone())
        };

        if url.is_empty() {
            warn!("Kuma Push URL is not configured. Waiting 10 seconds...");
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        }

        let start_time = tokio::time::Instant::now();
        info!("[{}] Sending heartbeat...", Local::now().format("%Y-%m-%d %H:%M:%S"));

        let internal_ping_time_ms = start_time.elapsed().as_millis().to_string();
        let mut final_url = url.replace("{PING}", &internal_ping_time_ms);

        // Внешний пинг, если настроен
        if !ping_target.is_empty() {
            let ping_command = format!("chcp 65001 > nul && ping -n 1 {}", ping_target);
            let ping_future = tokio::process::Command::new("cmd")
                .args(["/c", &ping_command])
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            let external_ping = match tokio::time::timeout(Duration::from_secs(2), ping_future).await {
                Ok(Ok(output)) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    info!("[DEBUG] Raw ping output: {}", stdout);
                    
                    // Ищем число, за которым следуют любые символы (кроме цифр) и слово TTL
                    // Это позволяет пропустить "байты=32" и взять именно время.
                    let re = regex::Regex::new(r"[=<]\s*(\d+)\s*[^0-9\r\n]+TTL").unwrap();
                    if let Some(caps) = re.captures(&stdout) {
                        let val = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_else(|| "1".to_string());
                        if val == "0" { "1".to_string() } else { val }
                    } else {
                        "1".to_string()
                    }
                }
                _ => "1".to_string(), // Timeout or error
            };
            final_url = final_url.replace("{EXPING}", &external_ping);
        } else {
            final_url = final_url.replace("{EXPING}", "0");
        }

        info!("[{}] Final Push URL: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), final_url);

        match client.get(&final_url)
            .header("User-Agent", "UptimeKumaPushAgent/4.6.0")
            .send()
            .await 
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("[{}] Heartbeat sent successfully! Response status: {}",
                             Local::now().format("%Y-%m-%d %H:%M:%S"),
                             response.status());
                } else {
                    warn!("[{}] Server responded with status: {}",
                          Local::now().format("%Y-%m-%d %H:%M:%S"),
                          response.status());
                }
            },
            Err(e) => {
                error!("[{}] Error sending heartbeat: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), e);
            },
        }

        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}

// --- Главная функция ---
#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Application Error: {}", e);
        println!("\nPress Enter to exit...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        std::process::exit(1);
    }
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let config_path = get_config_path();
    let is_first_run = !config_path.exists();
    
    let initial_config = load_config()?;
    setup_logging(initial_config.enable_logging)?;

    info!("Starting Uptime Kuma Push Agent v4.6.0...");
    if is_first_run {
        info!("First run detected: config file missing.");
    }

    let config_shared = Arc::new(Mutex::new(initial_config.clone()));

    // --- Настройка меню трея ---
    let settings_item = MenuItem::new("Настройки", true, None);
    let quit_item = MenuItem::new("Выход", true, None);
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
        .with_tooltip("Uptime Kuma Agent")
        .build()?;

    // Запускаем цикл сердцебиения в фоне
    let config_for_loop = Arc::clone(&config_shared);
    tokio::spawn(async move {
        run_heartbeat_loop(config_for_loop).await;
    });

    // --- Tao Event Loop ---
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Канал для событий меню (пробрасываем в основной цикл через прокси)
    let menu_channel = MenuEvent::receiver();
    let proxy_menu = proxy.clone();
    std::thread::spawn(move || {
        while let Ok(event) = menu_channel.recv() {
            let _ = proxy_menu.send_event(UserEvent::TrayMenuEvent(event));
        }
    });

    let mut settings_window = None;

    // Trigger first run wizard if needed
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
                                .with_title("Uptime Kuma Agent Settings")
                                .with_inner_size(tao::dpi::LogicalSize::new(450.0, 550.0))
                                .with_resizable(false)
                                .build(event_loop)
                                .unwrap();

                            let html_content = include_str!("settings.html");
                            
                            let proxy_webview = proxy.clone();
                            let config_current = config_shared.lock().unwrap().clone();

                            let webview = WebViewBuilder::new()
                                .with_ipc_handler(move |request| {
                                    let msg = request.body();
                                    if msg == "request_config" {
                                        // Handled by evaluate_script below
                                    } else if msg == "close_window" {
                                        let _ = proxy_webview.send_event(UserEvent::CloseSettings);
                                    } else {
                                        // Try to parse as new config
                                        if let Ok(new_cfg) = serde_json::from_str::<Config>(&msg) {
                                            let _ = proxy_webview.send_event(UserEvent::SettingsSaved(new_cfg));
                                        }
                                    }
                                })
                                .with_html(html_content)
                                .build(&window)
                                .unwrap();

                            // Send current config immediately after creation
                            let config_json = serde_json::to_string(&serde_json::json!({
                                "type": "config",
                                "kuma_push_url": config_current.kuma_push_url,
                                "interval_sec": config_current.interval_sec,
                                "autostart": config_current.autostart,
                                "enable_logging": config_current.enable_logging,
                                "ping_target": config_current.ping_target
                            })).unwrap();
                            let _ = webview.evaluate_script(&format!("window.postMessage('{}', '*')", config_json));

                            settings_window = Some((window, webview));
                        } else {
                            if let Some((window, _)) = &settings_window {
                                window.set_focus();
                            }
                        }
                    }
                    UserEvent::SettingsSaved(new_cfg) => {
                        info!("Saving new configuration...");
                        let _ = handle_autostart_logic(&new_cfg);
                        if let Err(e) = save_config(&new_cfg) {
                            error!("Failed to save config: {}", e);
                        } else {
                            let mut config = config_shared.lock().unwrap();
                            *config = new_cfg;
                            
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