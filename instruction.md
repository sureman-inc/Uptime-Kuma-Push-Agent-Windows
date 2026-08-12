# Uptime Kuma Push Agent Guide

A lightweight, multi-device network monitoring agent for Windows designed for retail stores, restaurants, and branch offices.

---

## 🚀 Quick Start
1. Launch `kuma_agent.exe`.
2. On first run, the **Settings** window will open automatically.
3. Enter your **Kuma Base URL**, **Branch Name**, and add your local devices (printers, routers, Wi-Fi APs, POS terminals).
4. Use the **⚡ Test** button to instantly verify connection to any device.
5. Click **Save Changes**.

---

## 📁 Data Storage
All configuration files and logs are saved in:
`%APPDATA%\KumaAgent\`

* **Configuration**: `%APPDATA%\KumaAgent\config.toml`
* **Log files**: `%APPDATA%\KumaAgent\logs\agent.log`

You can open this folder anytime by pressing `Win + R`, entering `%APPDATA%\KumaAgent`, and hitting Enter.

---

## ⚙️ Configuration Methods

### Method 1: Graphical Settings (Recommended)
1. Right-click the Uptime Kuma icon in the Windows System Tray (near the clock) and select **Settings**.
2. **General Tab**: Configure Branch Name, Kuma Base URL, interval, autostart, and file logging.
3. **Devices Tab**: Add, configure, test, and remove network devices.
4. Click **Save Changes**.

### Method 2: Direct TOML Configuration
Edit `%APPDATA%\KumaAgent\config.toml`:
```toml
branch_name = "Downtown Restaurant"
kuma_base_url = "https://kuma.example.com"
interval_sec = 30
autostart = true
enable_logging = true

[[devices]]
name = "Kitchen Printer"
target = "192.168.1.160"
check_type = "tcp"
port = 9100
token = "TOKEN_PRINTER"
enabled = true

[[devices]]
name = "Branch Router"
target = "192.168.1.1"
check_type = "icmp"
token = "TOKEN_ROUTER"
enabled = true
```

---

## 🛠️ Supported Check Types
* **TCP Port Check (`tcp`)**: Ideal for receipt printers (port 9100), RDP (port 3389), HTTP/HTTPS web panels (ports 80/443), or databases.
* **Native ICMP Ping (`icmp`)**: Fast, lightweight IP ping without administrative privileges using Windows `iphlpapi.dll`. Perfect for routers, Wi-Fi access points, POS terminals, and cameras.

---

## 🔍 Troubleshooting
* **Tray Icon Missing**: Check Windows Task Manager to confirm `kuma_agent.exe` is running, or check `%APPDATA%\KumaAgent\logs\agent.log`.
* **Push Not Received**: Verify that your **Kuma Base URL** and device **Push Tokens** match your Uptime Kuma push monitors.
