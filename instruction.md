# Uptime Kuma Agent Guide

## Quick Start
On the first launch, the application will automatically open the **Settings** window. You only need to enter your details and click **Save**.

## Data Storage
All settings and logs are located at:
`%APPDATA%\KumaAgent\`

You can quickly navigate there by pressing `Win + R` and entering `%APPDATA%\KumaAgent`.

## Configuration
### Method 1: Via Graphical Interface
1. If the program is already running, right-click the tray icon and select **Settings**.
2. Enter the **URL**, **Interval**, and **External Ping Target**.
3. Configure the startup and logging options.
4. Save the changes.

### Method 2: Via Configuration File
Edit `%APPDATA%\KumaAgent\config.toml`:
```toml
kuma_push_url="...&ping={EXPING}"
interval_sec = 30
ping_target = "8.8.8.8"
```

## How to Set Up the Push URL
- **`ping={EXPING}`** — sends the external ping latency to the graph.
- **`msg=...`** — text description.

**Example:**
`https://.../api/push/TOKEN?status=up&ping={EXPING}`

## Logging
The application log is located at `%APPDATA%\KumaAgent\logs\agent.log`.

## Troubleshooting
- **The window does not open**: Check if another instance of the application is already running in the system tray.
- **Data is not sent**: Check `agent.log` to see the final request URL being sent by the agent.

