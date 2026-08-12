# Uptime Kuma Push Agent (v5.0) - User & Admin Guide

A lightweight, multi-device network monitoring agent for Windows. Designed for restaurants, retail stores, warehouses, and branch offices to monitor local infrastructure (receipt printers, routers, Wi-Fi access points, POS terminals, and servers) and report real-time status to **[Uptime Kuma](https://github.com/louislam/uptime-kuma)**.

---

## 📋 Table of Contents
1. [Quick Start](#-quick-start)
2. [Setting Up Uptime Kuma Push Monitors](#-setting-up-uptime-kuma-push-monitors)
3. [Configuration via Graphical Interface](#-configuration-via-graphical-interface)
4. [Supported Device Presets & Check Types](#-supported-device-presets--check-types)
5. [Manual Configuration (config.toml)](#-manual-configuration-configtoml)
6. [Data Storage & Log Locations](#-data-storage--log-locations)
7. [System Tray & Background Operation](#-system-tray--background-operation)
8. [Troubleshooting & Diagnostics](#-troubleshooting--diagnostics)

---

## 🚀 Quick Start

1. Download or compile `kuma_agent.exe`.
2. Run `kuma_agent.exe`. On first launch, the **Settings** window will open automatically.
3. In **General Settings**, enter your **Branch Name** and **Uptime Kuma Base URL** (e.g. `https://kuma.yourdomain.com`).
4. Switch to the **Monitored Devices** tab, click **+ Add Device**, and select a preset (e.g., Receipt Printer, Router, Wi-Fi AP).
5. Enter the device's local IP address and paste the **Push Token** from Uptime Kuma.
6. Click **⚡ Test** to verify connection in real time.
7. Click **Save Changes**. The agent will minimize to the Windows System Tray and monitor all devices in the background.

---

## 📡 Setting Up Uptime Kuma Push Monitors

To receive heartbeats and latency graphs in Uptime Kuma:

1. Open your Uptime Kuma web dashboard.
2. Click **Add New Monitor**.
3. Set **Monitor Type** to **`Push`**.
4. Set a friendly name (e.g., `Branch Main - Cashier Printer` or `Branch Main - Router`).
5. Set the **Heartbeat Interval** to match your agent interval (default: `30` seconds).
6. Save the monitor.
7. Copy either:
   * Just the **Push Token** (e.g., `s9fX2j90K1`), or
   * The **Full Push URL** (e.g., `https://kuma.yourdomain.com/api/push/s9fX2j90K1?status=up&msg=OK&ping=`).

> [!TIP]
> If you specify the **Uptime Kuma Base URL** in General Settings (e.g. `https://kuma.yourdomain.com`), you only need to paste the short **Token** for each device!

---

## 🖥️ Configuration via Graphical Interface

To open settings at any time:
* Right-click the **Uptime Kuma icon** in the Windows System Tray (near the clock) and select **Settings**.

### 1. General Settings Tab
* **Branch / Location Name**: Friendly identifier for this store/office (e.g. `Downtown Restaurant`).
* **Check Interval (sec)**: How often all devices are polled (default: `30` seconds, minimum `5` sec).
* **Uptime Kuma Base URL**: Base address of your server (e.g. `https://kuma.example.com`).
* **Host PC Heartbeat Token (Optional)**: If set, sends an extra heartbeat confirming this agent computer is online and connected to the internet.
* **Launch automatically on Windows startup**: Enables automatic background launch via Windows Registry.
* **Enable logging to file**: Writes detailed status logs to `%APPDATA%\KumaAgent\logs\agent.log`.

### 2. Monitored Devices Tab
* **+ Add Device**: Adds a new device card.
* **Device Name**: Descriptive name (e.g. `Kitchen Ticket Printer`, `Hall AP`).
* **Target IP / Hostname**: Local IP (e.g. `192.168.1.160`) or domain name.
* **Check Type & Port**: `ICMP` (Ping) or `TCP` (Port check).
* **Kuma Push Token**: Token or full push URL.
* **⚡ Test Button**: Instantly tests connectivity to the target device and displays real-time latency (`🟢 3 ms` or `🔴 Offline`).
* **Enable / Disable Toggle**: Temporarily disable monitoring for a specific device without deleting it.
* **Delete Button (✕)**: Removes the device from the list.

---

## 🛠️ Supported Device Presets & Check Types

When adding devices, use the **-- Apply Preset --** dropdown to auto-configure standard hardware:

| Preset | Check Type | Port | Ideal For |
| :--- | :--- | :--- | :--- |
| **🖨️ Receipt Printer** | `TCP` | `9100` | POS thermal printers (Epson, Star, Xprinter, Sam4s, Bixolon) |
| **🌐 Router / Gateway** | `ICMP` | — | Main internet gateways (MikroTik, Keenetic, Cisco, TP-Link) |
| **📶 Wi-Fi AP** | `ICMP` | — | Access points (UniFi, Aruba, Ruijie, MikroTik CAPsMAN) |
| **🖥️ POS / Terminal (RDP)**| `TCP` | `3389` | Waitress monoblocks, Windows POS stations, RDP servers |
| **🖥️ POS / Terminal (Ping)**| `ICMP` | — | Android POS terminals, touch monitors, smart scales |
| **⚙️ Custom TCP Port** | `TCP` | *Custom* | Database servers, web interfaces (80/443), cameras (554 RTSP) |
| **⚙️ Custom ICMP Ping** | `ICMP` | — | Any IP device supporting standard ICMP echo |

---

## 📝 Manual Configuration (`config.toml`)

Advanced users can edit the configuration file directly at `%APPDATA%\KumaAgent\config.toml`:

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

## 📁 Data Storage & Log Locations

All application data is stored in the standard Windows Application Data folder:
`%APPDATA%\KumaAgent\`

* **Configuration File**: `%APPDATA%\KumaAgent\config.toml`
* **Log File**: `%APPDATA%\KumaAgent\logs\agent.log`

> **Quick Access**: Press `Win + R`, paste `%APPDATA%\KumaAgent`, and press **Enter**.

---

## 🔄 System Tray & Background Operation

* **Silent Execution**: The agent runs silently in the background with zero terminal popups (`windows_subsystem`).
* **Memory & CPU Footprint**: Uses only **3–6 MB RAM** and **0% CPU** thanks to Tokio async I/O.
* **Tray Controls**:
  * **Settings**: Opens the graphical configuration panel.
  * **Exit**: Completely terminates the background agent.

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
* Open `%APPDATA%\KumaAgent\logs\agent.log` in Notepad to see timestamped diagnostic output for every monitoring cycle and HTTP push response code.
