# Uptime Kuma Push Agent for Windows (v5.0)

A high-performance, ultra-lightweight multi-device monitoring station for Windows built for **[Uptime Kuma](https://github.com/louislam/uptime-kuma)**. 

Designed for restaurants, retail stores, warehouses, and branch offices to monitor local network infrastructure (thermal receipt printers, routers, Wi-Fi access points, POS terminals, and servers) concurrently and report real-time status and latency to Uptime Kuma.

---

## 📋 Table of Contents
1. [✨ Key Features](#-key-features)
2. [🚀 Quick Start](#-quick-start)
3. [📡 Setting Up Uptime Kuma Push Monitors](#-setting-up-uptime-kuma-push-monitors)
4. [🛠️ Supported Device Presets & Check Types](#-supported-device-presets--check-types)
5. [🖥️ Graphical Settings & Diagnostics](#-graphical-settings--diagnostics)
6. [📝 Configuration File Reference (`config.toml`)](#-configuration-file-reference-configtoml)
7. [🔨 How to Build from Source](#-how-to-build-from-source)
8. [📁 Data Storage & Log Locations](#-data-storage--log-locations)
9. [🔍 Troubleshooting & Diagnostics](#-troubleshooting--diagnostics)
10. [📄 License](#-license)

---

## ✨ Key Features

* **Multi-Device Concurrent Monitoring**: Monitor 10–50+ devices simultaneously in under 50ms using Tokio async tasks (`tokio::spawn`).
* **Native Windows ICMP Ping**: Fast IP ping via `iphlpapi.dll` (`IcmpSendEcho`) without spawning external `cmd.exe` processes and without requiring Administrator rights.
* **TCP Port Health Checks**: Instant connectivity testing for receipt printers (port `9100`), Windows RDP (port `3389`), and Web interfaces (`80`/`443`).
* **Interactive Diagnostics**: Built-in **⚡ Test** button in settings providing instant round-trip latency (ms) verification before saving.
* **Modern Glassmorphism UI**: Built with Wry/Tao webview, device presets, and responsive layout.
* **Zero Overhead**: Operates silently in the Windows System Tray consuming only **3–6 MB RAM** and **0% CPU**.
* **One-Click Windows Autostart**: Seamless startup integration via Windows Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).

---

## 🚀 Quick Start

1. Download the latest `kuma_agent.exe` (or build from source).
2. Launch `kuma_agent.exe`. On first run, the **Settings** window will open automatically.
3. In the **General Settings** tab:
   * Enter your **Branch Name** (e.g. `Downtown Restaurant`).
   * Enter your **Uptime Kuma Base URL** (e.g. `https://kuma.yourdomain.com`).
4. In the **Monitored Devices** tab:
   * Click **+ Add Device**.
   * Select a preset (e.g., *Receipt Printer*, *Router*, *Wi-Fi AP*).
   * Enter the device IP address and paste the **Push Token** from Uptime Kuma.
   * Click **⚡ Test** to verify connection live.
5. Click **Save Changes**. The agent will minimize to the System Tray and continuously monitor your branch devices in the background.

---

## 📡 Setting Up Uptime Kuma Push Monitors

To receive heartbeats and latency graphs in Uptime Kuma:

1. Open your Uptime Kuma dashboard.
2. Click **Add New Monitor**.
3. Select **Monitor Type** as **`Push`**.
4. Set a friendly name (e.g. `Main Branch - Cashier Printer` or `Main Branch - Router`).
5. Set the **Heartbeat Interval** to match your agent interval (default: `30` seconds).
6. Save the monitor.
7. Copy either:
   * Just the **Push Token** (e.g., `s9fX2j90K1`), or
   * The **Full Push URL** (e.g., `https://kuma.yourdomain.com/api/push/s9fX2j90K1?status=up&msg=OK&ping=`).

> [!TIP]
> If you specify the **Uptime Kuma Base URL** in General Settings (e.g. `https://kuma.yourdomain.com`), you only need to paste the short **Token** for each device!

---

## 🛠️ Supported Device Presets & Check Types

Use the **-- Apply Preset --** dropdown in settings to auto-configure standard network hardware:

| Preset | Check Type | Default Port | Ideal For |
| :--- | :--- | :--- | :--- |
| **🖨️ Receipt Printer** | `TCP` | `9100` | POS thermal & ticket printers (Epson, Star, Xprinter, Sam4s, Bixolon) |
| **🌐 Router / Gateway** | `ICMP` | — | Main internet gateways (MikroTik, Keenetic, Cisco, TP-Link) |
| **📶 Wi-Fi AP** | `ICMP` | — | Access points (UniFi, Aruba, Ruijie, MikroTik CAPsMAN) |
| **🖥️ POS / Terminal (RDP)**| `TCP` | `3389` | Waitress monoblocks, Windows POS stations, RDP servers |
| **🖥️ POS / Terminal (Ping)**| `ICMP` | — | Android POS terminals, touch monitors, smart scales |
| **⚙️ Custom TCP Port** | `TCP` | *Custom* | Database servers, web admin panels (80/443), cameras (554 RTSP) |
| **⚙️ Custom ICMP Ping** | `ICMP` | — | Any IP device supporting standard ICMP echo |

---

## 🖥️ Graphical Settings & Diagnostics

To open the settings panel at any time, right-click the **Uptime Kuma icon** in the Windows System Tray (near the clock) and select **Settings**.

### 1. General Settings Tab
* **Branch / Location Name**: Identifies this store/office in push messages and logs.
* **Check Interval (sec)**: Polling interval across all devices (default: `30` seconds, min: `5` sec).
* **Uptime Kuma Base URL**: Base URL of your Kuma server.
* **Host PC Heartbeat Token (Optional)**: Sends a heartbeat verifying that this Windows agent PC is online.
* **Launch automatically on Windows startup**: Enables automatic background launch.
* **Enable logging to file**: Writes detailed status logs to `%APPDATA%\KumaAgent\logs\agent.log`.

### 2. Monitored Devices Tab
* **+ Add Device**: Adds a new device card.
* **Device Name**: Descriptive name (e.g. `Bar Receipt Printer`).
* **Target IP / Hostname**: Local IP (e.g. `192.168.1.160`) or domain name.
* **Check Type & Port**: Switch between `ICMP` (Ping) and `TCP` (Port check).
* **Kuma Push Token**: Token or full push URL.
* **⚡ Test Button**: Instantly verifies target reachability and displays live latency (`🟢 2 ms` or `🔴 Connection timed out`).
* **Enable / Disable Toggle**: Temporarily disable monitoring for a specific device without deleting it.
* **Delete Button (✕)**: Removes the device from the list.

---

## 📝 Configuration File Reference (`config.toml`)

The configuration file is stored at `%APPDATA%\KumaAgent\config.toml` and can be edited manually:

```toml
branch_name = "Downtown Restaurant"
kuma_base_url = "https://kuma.example.com"
interval_sec = 30
autostart = true
enable_logging = true

# Optional Host PC heartbeat
agent_push_token = "HOST_PC_TOKEN"

# 1. Thermal Receipt Printer (TCP Port 9100)
[[devices]]
name = "Cashier Printer"
target = "192.168.1.160"
check_type = "tcp"
port = 9100
token = "PRINTER_TOKEN_1"
enabled = true

# 2. Kitchen Ticket Printer (TCP Port 9100)
[[devices]]
name = "Kitchen Printer"
target = "192.168.1.165"
check_type = "tcp"
port = 9100
token = "PRINTER_TOKEN_2"
enabled = true

# 3. Main Internet Router (Native ICMP Ping)
[[devices]]
name = "Main Router MikroTik"
target = "192.168.1.1"
check_type = "icmp"
token = "ROUTER_TOKEN_3"
enabled = true

# 4. Dining Room Wi-Fi AP (Native ICMP Ping)
[[devices]]
name = "Dining Hall AP"
target = "192.168.1.50"
check_type = "icmp"
token = "WIFI_TOKEN_4"
enabled = true
```

---

## 🔨 How to Build from Source

### Prerequisites
* Rust toolchain (cargo, rustc): [rustup.rs](https://rustup.rs/)

### Build Commands
```powershell
# 1. Clone repository
git clone https://github.com/sureman-inc/Uptime-Kuma-Push-Agent-Windows.git
cd Uptime-Kuma-Push-Agent-Windows

# 2. Verify compilation
cargo check

# 3. Build optimized release binary
cargo build --release
```

The compiled binary `kuma_agent.exe` will be located in `target/release/`.

---

## 📁 Data Storage & Log Locations

All application data is stored in the standard Windows Application Data directory:
`%APPDATA%\KumaAgent\`

* **Configuration File**: `%APPDATA%\KumaAgent\config.toml`
* **Log File**: `%APPDATA%\KumaAgent\logs\agent.log`

> **Quick Access**: Press `Win + R`, paste `%APPDATA%\KumaAgent`, and press **Enter**.

---

## 🔍 Troubleshooting & Diagnostics

### 1. Device is shown as "Offline" in Uptime Kuma:
* Open Settings and click **⚡ Test** next to the device:
  * If the test returns **"Port 9100 open"**, check if your Uptime Kuma Push Token or Base URL is typed correctly.
  * If the test returns **"Connection refused"** or **"Connection timed out"**, verify that the device is powered on, has the correct IP address, and is on the same local subnet.
  * For printers, ensure the printer's network interface is active (print a network configuration slip from the printer).

### 2. Windows Defender / Firewall warnings:
* The agent only performs outbound local network checks (ICMP/TCP) and HTTPS push requests. It does not open any inbound listening ports.

### 3. Reviewing detailed logs:
* Open `%APPDATA%\KumaAgent\logs\agent.log` to see timestamped diagnostic output for every monitoring cycle and HTTP push response code.

---

## 📄 License

MIT License. See `LICENSE` for details.
