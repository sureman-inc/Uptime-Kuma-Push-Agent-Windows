# Uptime Kuma Push Agent for Windows (Multi-Device Monitor)

A high-performance, lightweight Windows push agent for **[Uptime Kuma](https://github.com/louislam/uptime-kuma)**. 

Designed for branch offices, restaurants, and retail stores to monitor local network infrastructure (receipt printers, routers, Wi-Fi access points, POS terminals, and servers) using native ICMP and async TCP port checks.

---

## ✨ Features
* **Multi-Device Concurrent Monitoring**: Monitor 10–50+ devices simultaneously in under 50ms using Tokio async tasks.
* **Native Windows ICMP Ping**: Fast IP ping via `iphlpapi.dll` without spawning slow `cmd.exe` processes and without requiring Administrator rights.
* **TCP Port Check**: Instant status checks for receipt printers (port 9100), RDP (port 3389), Web admin panels (80/443), etc.
* **Modern Glassmorphism UI**: Built with Wry/Tao webview, tabs, presets, and live connection test buttons.
* **System Tray Integration**: Silent background execution with tray menu and status tooltip.
* **Windows Autostart**: Optional one-click startup integration via Windows Registry.
* **Extremely Lightweight**: Runs in background consuming only ~3–6 MB RAM and virtually 0% CPU.

---

## 🚀 Quick Start
On first launch, the **Settings** window will open automatically. Configure your branch, add devices, test connectivity, and click **Save Changes**.

---

## 🛠️ How to Build from Source

### Prerequisites
* Rust Toolchain (cargo, rustc): [rustup.rs](https://rustup.rs/)

### Build Commands
```powershell
# Check compilation
cargo check

# Debug build
cargo build

# Optimized Release build (Recommended for production)
cargo build --release
```

The compiled binary `kuma_agent.exe` will be located in `target/release/`.

---

## 📁 Data & Logs Storage
All settings and logs are located at:
`%APPDATA%\KumaAgent\`

* Configuration: `%APPDATA%\KumaAgent\config.toml`
* Log file: `%APPDATA%\KumaAgent\logs\agent.log`

---

## 📝 Configuration Example (`config.toml`)
```toml
branch_name = "Main Branch"
kuma_base_url = "https://kuma.example.com"
interval_sec = 30
autostart = true
enable_logging = true

# Optional Host PC heartbeat
agent_push_token = "HOST_PC_TOKEN"

# 1. Receipt Printer (Port 9100)
[[devices]]
name = "Cashier Printer"
target = "192.168.1.160"
check_type = "tcp"
port = 9100
token = "PRINTER_TOKEN"
enabled = true

# 2. Main Router (ICMP Ping)
[[devices]]
name = "Main Router"
target = "192.168.1.1"
check_type = "icmp"
token = "ROUTER_TOKEN"
enabled = true

# 3. Wi-Fi Access Point (ICMP Ping)
[[devices]]
name = "Hall Wi-Fi AP"
target = "192.168.1.50"
check_type = "icmp"
token = "WIFI_TOKEN"
enabled = true
```

---

## 📄 License
MIT License.
