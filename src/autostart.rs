use std::{env, fs};
use log::info;
use winreg::enums::*;
use winreg::RegKey;
use crate::config::{get_app_data_dir, get_config_path, Config};

pub fn handle_autostart_logic(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let (key, _disp) = hkcu.create_subkey(path)?;

    if config.autostart {
        let current_exe = env::current_exe()?;
        let target_dir = get_app_data_dir();
        let target_exe = target_dir.join("kuma_agent.exe");
        let target_config = target_dir.join("config.toml");

        // If we are not already in the permanent target folder, copy executable there
        if current_exe != target_exe {
            info!("Installing to permanent location: {:?}", target_dir);
            fs::create_dir_all(&target_dir)?;
            
            // Copy EXE
            fs::copy(&current_exe, &target_exe)?;
            
            // Copy config if present
            let current_config = get_config_path();
            if current_config.exists() {
                let _ = fs::copy(&current_config, &target_config);
            }
        }

        info!("Enabling autostart in Windows Registry pointing to {:?}", target_exe);
        key.set_value("KumaAgent", &target_exe.to_str().unwrap_or_default())?;
    } else {
        info!("Disabling autostart in Windows Registry");
        let _ = key.delete_value("KumaAgent");
    }

    Ok(())
}
