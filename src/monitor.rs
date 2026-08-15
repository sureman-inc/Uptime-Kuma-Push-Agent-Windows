use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::{error, info, warn};
use reqwest::Client;
use crate::config::{CheckType, Config};

// --- Windows Native ICMP FFI Definitions ---
type HANDLE = *mut std::ffi::c_void;
type DWORD = u32;
type WORD = u16;
type IPAddr = u32;

const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

#[repr(C)]
struct IpOptionInformation {
    ttl: u8,
    tos: u8,
    flags: u8,
    options_size: u8,
    options_data: *mut u8,
}

#[repr(C)]
struct IcmpEchoReply {
    address: IPAddr,
    status: DWORD,
    round_trip_time: DWORD,
    data_size: WORD,
    reserved: WORD,
    data: *mut u8,
    options: IpOptionInformation,
}

#[link(name = "iphlpapi")]
extern "system" {
    fn IcmpCreateFile() -> HANDLE;
    fn IcmpCloseHandle(icmp_handle: HANDLE) -> i32;
    fn IcmpSendEcho(
        icmp_handle: HANDLE,
        destination_address: IPAddr,
        request_data: *const std::ffi::c_void,
        request_size: WORD,
        request_options: *const IpOptionInformation,
        reply_buffer: *mut std::ffi::c_void,
        reply_size: DWORD,
        timeout: DWORD,
    ) -> DWORD;
}

fn resolve_to_ipv4(target: &str) -> Option<Ipv4Addr> {
    if let Ok(ip) = target.trim().parse::<Ipv4Addr>() {
        return Some(ip);
    }
    // Attempt DNS hostname resolution
    let host_with_port = format!("{}:80", target.trim());
    if let Ok(addrs) = host_with_port.to_socket_addrs() {
        for addr in addrs {
            if let IpAddr::V4(ipv4) = addr.ip() {
                return Some(ipv4);
            }
        }
    }
    None
}

/// Perform native Windows ICMP Ping
pub async fn ping_icmp(target: &str, timeout_ms: u32) -> (bool, u128, String) {
    let target_owned = target.to_string();
    tokio::task::spawn_blocking(move || {
        let ipv4 = match resolve_to_ipv4(&target_owned) {
            Some(ip) => ip,
            None => return (false, 0, format!("Could not resolve host '{}'", target_owned)),
        };

        unsafe {
            let handle = IcmpCreateFile();
            if handle == INVALID_HANDLE_VALUE || handle.is_null() {
                return (false, 0, "Failed to initialize Windows ICMP handle".to_string());
            }

            let dest_ip: IPAddr = u32::from_ne_bytes(ipv4.octets());
            let send_data: [u8; 32] = [0x55; 32];
            let reply_buffer_size = std::mem::size_of::<IcmpEchoReply>() + send_data.len() + 8;
            let mut reply_buffer: Vec<u8> = vec![0; reply_buffer_size];

            let replies_count = IcmpSendEcho(
                handle,
                dest_ip,
                send_data.as_ptr() as *const std::ffi::c_void,
                send_data.len() as WORD,
                std::ptr::null(),
                reply_buffer.as_mut_ptr() as *mut std::ffi::c_void,
                reply_buffer_size as DWORD,
                timeout_ms,
            );

            let result = if replies_count > 0 {
                let reply = &*(reply_buffer.as_ptr() as *const IcmpEchoReply);
                if reply.status == 0 {
                    let rtt = reply.round_trip_time as u128;
                    (true, rtt.max(1), format!("Echo reply received ({} ms)", rtt.max(1)))
                } else {
                    (false, 0, format!("ICMP status error: {}", reply.status))
                }
            } else {
                (false, 0, "Request timed out".to_string())
            };

            IcmpCloseHandle(handle);
            result
        }
    })
    .await
    .unwrap_or_else(|e| (false, 0, format!("Task error: {}", e)))
}

/// Perform TCP Port Ping
pub async fn ping_tcp(target: &str, port: u16, timeout_ms: u64) -> (bool, u128, String) {
    let addr = format!("{}:{}", target.trim(), port);
    let start = std::time::Instant::now();
    let res = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        tokio::net::TcpStream::connect(&addr),
    )
    .await;
    let latency = start.elapsed().as_millis().max(1);

    match res {
        Ok(Ok(_stream)) => (true, latency, format!("Port {} open ({} ms)", port, latency)),
        Ok(Err(e)) => (false, 0, format!("Connection refused: {}", e)),
        Err(_) => (false, 0, format!("Connection to port {} timed out", port)),
    }
}

/// Universal check for a target device
pub async fn test_target(
    target: &str,
    check_type: &CheckType,
    port: Option<u16>,
    timeout_ms: u64,
) -> (bool, u128, String) {
    if target.trim().is_empty() {
        return (false, 0, "Target address is empty".to_string());
    }

    match check_type {
        CheckType::Icmp => ping_icmp(target, timeout_ms as u32).await,
        CheckType::Tcp => {
            let target_port = port.unwrap_or(9100);
            ping_tcp(target, target_port, timeout_ms).await
        }
    }
}

/// Push status and latency to Uptime Kuma
pub async fn send_kuma_push(
    client: &Client,
    kuma_base_url: &str,
    token: &str,
    is_up: bool,
    latency_ms: u128,
    msg: &str,
/// Push status and latency to Uptime Kuma
pub async fn send_kuma_push(
    client: &Client,
    kuma_base_url: &str,
    token_or_url: &str,
    is_up: bool,
    latency_ms: u128,
    msg: &str,
) -> Result<u16, String> {
    let raw = token_or_url.trim();
    if raw.is_empty() {
        return Err("Empty push token or URL".to_string());
    }

    let status_str = if is_up { "up" } else { "down" };
    let latency_str = latency_ms.max(1).to_string();

    let final_url = if raw.starts_with("http://") || raw.starts_with("https://") {
        let mut base_and_query = raw.to_string();

        // 1. Handle template placeholders if present
        if base_and_query.contains("{STATUS}") {
            base_and_query = base_and_query.replace("{STATUS}", status_str);
        }
        if base_and_query.contains("{PING}") || base_and_query.contains("{EXPING}") {
            base_and_query = base_and_query
                .replace("{PING}", &latency_str)
                .replace("{EXPING}", &latency_str);
        }
        if base_and_query.contains("{MSG}") {
            base_and_query = base_and_query.replace("{MSG}", msg);
        }

        // 2. Parse and replace existing query parameters (e.g. status=up&msg=OK&ping=)
        if let Some((url_path, query)) = base_and_query.split_once('?') {
            let mut params: Vec<(String, String)> = Vec::new();
            let mut has_status = false;
            let mut has_ping = false;
            let mut has_msg = false;

            for pair in query.split('&') {
                if pair.is_empty() { continue; }
                if let Some((k, v)) = pair.split_once('=') {
                    if k == "status" {
                        params.push((k.to_string(), status_str.to_string()));
                        has_status = true;
                    } else if k == "ping" {
                        params.push((k.to_string(), latency_str.clone()));
                        has_ping = true;
                    } else if k == "msg" {
                        let final_msg = if !is_up { msg.to_string() } else if v.is_empty() { "OK".to_string() } else { v.to_string() };
                        params.push((k.to_string(), final_msg));
                        has_msg = true;
                    } else {
                        params.push((k.to_string(), v.to_string()));
                    }
                } else {
                    params.push((pair.to_string(), String::new()));
                }
            }

            if !has_status {
                params.push(("status".to_string(), status_str.to_string()));
            }
            if !has_ping {
                params.push(("ping".to_string(), latency_str));
            }
            if !has_msg {
                params.push(("msg".to_string(), msg.to_string()));
            }

            let query_str = params
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");

            format!("{}?{}", url_path, query_str)
        } else {
            format!("{}?status={}&msg={}&ping={}", base_and_query, status_str, msg, latency_str)
        }
    } else {
        let base = kuma_base_url.trim().trim_end_matches('/');
        if base.is_empty() {
            return Err("Kuma Base URL is not configured and token is not a full URL".to_string());
        }
        format!(
            "{}/api/push/{}?status={}&msg={}&ping={}",
            base,
            raw,
            status_str,
            msg,
            latency_str
        )
    };

    match client
        .get(&final_url)
        .header("User-Agent", "UptimeKumaPushAgent/5.0.0")
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(resp.status().as_u16())
            } else {
                Err(format!("Server returned HTTP {}", resp.status()))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

/// Run monitoring loop for all devices and agent host with exact interval compensation
pub async fn run_monitoring_loop(config_arc: Arc<Mutex<Config>>) {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|e| {
            error!("Failed to create reqwest HTTP client: {}", e);
            Client::new()
        });

    loop {
        let cycle_start = tokio::time::Instant::now();

        let (branch_name, kuma_base_url, interval_sec, agent_push_token, devices) = {
            let cfg = config_arc.lock().unwrap();
            (
                cfg.branch_name.clone(),
                cfg.kuma_base_url.clone(),
                cfg.interval_sec,
                cfg.agent_push_token.clone(),
                cfg.devices.clone(),
            )
        };

        info!("--- Starting monitoring cycle for branch '{}' ({} devices configured) ---", branch_name, devices.len());

        let mut check_tasks = Vec::new();

        // 1. Monitor Host PC / Agent Heartbeat if configured
        if !agent_push_token.trim().is_empty() {
            let client_ref = client.clone();
            let base_url = kuma_base_url.clone();
            let token = agent_push_token.clone();
            let b_name = branch_name.clone();

            check_tasks.push(tokio::spawn(async move {
                let (up, latency, _msg) = ping_icmp("1.1.1.1", 2000).await;
                let push_msg = if up { format!("{} (Agent Online)", b_name) } else { "Agent (Degraded)".to_string() };
                match send_kuma_push(&client_ref, &base_url, &token, up, latency, &push_msg).await {
                    Ok(code) => info!("[Agent Heartbeat] Push sent successfully (HTTP {})", code),
                    Err(e) => warn!("[Agent Heartbeat] Failed to push heartbeat: {}", e),
                }
            }));
        }

        // 2. Concurrently monitor all enabled devices
        for dev in devices.into_iter().filter(|d| d.enabled && !d.target.trim().is_empty()) {
            let client_ref = client.clone();
            let base_url = kuma_base_url.clone();

            check_tasks.push(tokio::spawn(async move {
                let timeout_ms = 2000;
                let (is_online, latency, status_details) = test_target(&dev.target, &dev.check_type, dev.port, timeout_ms).await;

                if is_online {
                    info!("[Device: {} ({} | {:?})] ONLINE - {} ms", dev.name, dev.target, dev.check_type, latency);
                } else {
                    warn!("[Device: {} ({} | {:?})] OFFLINE - {}", dev.name, dev.target, dev.check_type, status_details);
                }

                if !dev.token.trim().is_empty() {
                    let msg = if is_online { "OK" } else { &status_details };
                    match send_kuma_push(&client_ref, &base_url, &dev.token, is_online, latency, msg).await {
                        Ok(code) => info!("[Device: {}] Push reported to Kuma (HTTP {})", dev.name, code),
                        Err(e) => warn!("[Device: {}] Failed to push to Kuma: {}", dev.name, e),
                    }
                }
            }));
        }

        // Wait for all checks to complete
        for task in check_tasks {
            let _ = task.await;
        }

        // Compensate for execution time so the interval is exact
        let elapsed = cycle_start.elapsed();
        let target_interval = Duration::from_secs(interval_sec.max(5));
        if elapsed < target_interval {
            tokio::time::sleep(target_interval - elapsed).await;
        } else {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
