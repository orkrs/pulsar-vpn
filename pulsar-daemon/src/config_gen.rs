use serde_json::{json, Value};
use url::Url;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{RwLock, OnceLock};
use std::time::{Duration, Instant};
use pulsar_shared::vpn::RoutingConfig;
use pulsar_shared::vpn::routing_config::Mode as PbMode;
use tracing::{debug, trace, warn};

// --- ГЛОБАЛЬНЫЙ DNS КЭШ ---
static DNS_CACHE: OnceLock<RwLock<HashMap<String, (String, Instant)>>> = OnceLock::new();

fn get_dns_cache() -> &'static RwLock<HashMap<String, (String, Instant)>> {
    DNS_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

// --- ПЛАТФОРМЕННЫЕ КОНСТАНТЫ ---

/// Имена собственных процессов, которые НЕ ДОЛЖНЫ идти через TUN/VPN,
/// чтобы не зациклить трафик (control-канал Tauri ↔ daemon ↔ sing-box).
/// На Windows: .exe-суффикс; на Linux/macOS/Android: без суффикса.
#[cfg(target_os = "windows")]
const SELF_PROCESS_NAMES: &[&str] = &["pulsar-ui.exe", "pulsar-daemon.exe", "sing-box.exe"];
#[cfg(not(target_os = "windows"))]
const SELF_PROCESS_NAMES: &[&str] = &["pulsar-ui", "pulsar-daemon", "sing-box"];

/// Per-app routing через `process_name` работает только на платформах,
/// где TUN-стек видит PID отправителя. На Android VPN через
/// `android.net.VpnService` PID недоступен — apps-list потребляется
/// `VpnService.Builder.addAllowedApplication`/`addDisallowedApplication`
/// на стороне Kotlin, до `establish()`. Поэтому на Android маршрут по
/// `process_name` эмитировать НЕ нужно: sing-box его тихо проигнорирует.
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
const PROCESS_NAME_ROUTING_SUPPORTED: bool = true;
#[cfg(target_os = "android")]
const PROCESS_NAME_ROUTING_SUPPORTED: bool = false;

// --- ФУНКЦИИ ГЕНЕРАЦИИ OUTBOUND (VLESS, WIREGUARD, TROJAN, HYSTERIA2) ---

fn generate_vless_outbound(
    url: &Url,
    connection_address: &str,
    port: u16,
    server_host: &str,
    allow_insecure: bool,
) -> Result<Value> {
    let mut outbound = json!({
        "type": "vless", "tag": "proxy", "server": connection_address, "server_port": port,
        "uuid": url.username(), "packet_encoding": "xudp"
    });
    let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let proxy_obj = outbound.as_object_mut().unwrap();

    if let Some(flow) = params.get("flow") {
        if !flow.is_empty() {
            proxy_obj.insert("flow".to_string(), json!(flow));
        }
    }

    let sni = params.get("sni").unwrap_or(&server_host.to_string()).clone();
    let fp = params.get("fp").unwrap_or(&"chrome".to_string()).clone();

    if let Some(sec) = params.get("security") {
        let mut tls = json!({
            "enabled": true,
            "server_name": sni,
            "utls": { "enabled": true, "fingerprint": fp },
            "alpn": ["h2", "http/1.1"],
            "insecure": allow_insecure, // добавляем параметр
        });
        if sec == "reality" {
            let pbk = params.get("pbk").map(|s| s.as_str()).unwrap_or("");
            let sid = params.get("sid").map(|s| s.as_str()).unwrap_or("");
            tls.as_object_mut().unwrap().insert("reality".to_string(), json!({
                "enabled": true,
                "public_key": pbk,
                "short_id": sid
            }));
            // для reality insecure не используется, можно оставить или удалить
            tls.as_object_mut().unwrap().remove("insecure");
        }
        proxy_obj.insert("tls".to_string(), tls);
    }
    Ok(outbound)
}

fn generate_wireguard_outbound(url: &Url, connection_address: &str, port: u16) -> Result<Value> {
    let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let private_key = params.get("private_key").or_else(|| params.get("pk")).unwrap_or(&String::new()).clone();
    let peer_public_key = params.get("public_key").or_else(|| params.get("peer_key")).unwrap_or(&String::new()).clone();
    let local_address = params.get("address").unwrap_or(&"172.16.0.2/32".to_string()).clone();
    let local_addr_vec: Vec<String> = local_address.split(',').map(|s| s.trim().to_string()).collect();

    Ok(json!({
        "type": "wireguard", "tag": "proxy", "server": connection_address, "server_port": port,
        "local_address": local_addr_vec, "private_key": private_key, "peer_public_key": peer_public_key, "mtu": 1280
    }))
}

fn generate_trojan_outbound(
    url: &Url,
    connection_address: &str,
    port: u16,
    server_host: &str,
    allow_insecure: bool,
) -> Result<Value> {
    let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let password = url.username();
    let sni = params.get("sni").or_else(|| params.get("peer")).unwrap_or(&server_host.to_string()).clone();
    let mut alpn = vec!["h2", "http/1.1"];
    if let Some(alpn_str) = params.get("alpn") {
        alpn = alpn_str.split(',').map(|s| s.trim()).collect();
    }

    Ok(json!({
        "type": "trojan", "tag": "proxy", "server": connection_address, "server_port": port, "password": password,
        "tls": {
            "enabled": true,
            "server_name": sni,
            "insecure": allow_insecure, // используем параметр
            "alpn": alpn
        }
    }))
}

fn generate_hysteria2_outbound(
    url: &Url,
    connection_address: &str,
    port: u16,
    server_host: &str,
    allow_insecure: bool,
) -> Result<Value> {
    let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let password = if !url.password().unwrap_or("").is_empty() {
        url.password().unwrap()
    } else {
        url.username()
    };
    let sni = params.get("sni").or_else(|| params.get("peer")).unwrap_or(&server_host.to_string()).clone();

    let mut outbound = json!({
        "type": "hysteria2", "tag": "proxy", "server": connection_address, "server_port": port, "password": password,
        "tls": {
            "enabled": true,
            "server_name": sni,
            "insecure": allow_insecure, // используем параметр
            "alpn": ["h3"]
        }
    });

    if let Some(v) = params.get("up").or_else(|| params.get("up_mbps")) {
        outbound.as_object_mut().unwrap().insert("up_mbps".to_string(), json!(v.parse::<u32>().unwrap_or(0)));
    }
    if let Some(v) = params.get("down").or_else(|| params.get("down_mbps")) {
        outbound.as_object_mut().unwrap().insert("down_mbps".to_string(), json!(v.parse::<u32>().unwrap_or(0)));
    }

    if let Some(obfs) = params.get("obfs") {
        if obfs == &"salamander" {
            let obfs_password = params.get("obfs-password").unwrap_or(&"pulsar".to_string()).clone();
            outbound.as_object_mut().unwrap().insert("obfs".to_string(), json!({
                "type": "salamander",
                "password": obfs_password
            }));
        }
    }
    Ok(outbound)
}

/// Предпочитать IPv4 при исходящих соединениях прокси, если пользователь отключил IPv6.
fn apply_outbound_domain_strategy(proxy: &mut Value, use_ipv6: bool) {
    if use_ipv6 {
        return;
    }
    if let Some(obj) = proxy.as_object_mut() {
        obj.insert("domain_strategy".to_string(), json!("ipv4_only"));
    }
}

// --- ОСНОВНАЯ ФУНКЦИЯ ГЕНЕРАЦИИ ---

pub fn generate_singbox_config(link: &str, routing: &RoutingConfig) -> Result<String> {
    let use_ipv6 = routing.use_ipv6;
    debug!(mode = ?routing.mode(), enable_geoip = routing.enable_geoip, use_ipv6, "Generating config");

    // 1. Парсинг URL
    let url = Url::parse(link).map_err(|_| anyhow!("Invalid URL format"))?;
    let scheme = url.scheme();
    let server_host = url.host_str().ok_or_else(|| anyhow!("Missing host"))?;

    let port = url.port().unwrap_or_else(|| match scheme {
        "wireguard" | "warp" => 51820,
        _ => 443
    });

    // 2. РЕЗОЛВИМ IP-АДРЕС СЕРВЕРА С КЭШИРОВАНИЕМ
    let cache_key = server_host.to_string();
    let cache = get_dns_cache();
    let mut server_ip: Option<String> = None;

    if let Ok(read_guard) = cache.read() {
        if let Some((cached_ip, timestamp)) = read_guard.get(&cache_key) {
            if timestamp.elapsed() < Duration::from_secs(300) {
                debug!("Using cached IP for {}: {}", server_host, cached_ip);
                server_ip = Some(cached_ip.clone());
            }
        }
    }

    if server_ip.is_none() {
        match format!("{}:{}", server_host, port).to_socket_addrs() {
            Ok(addrs) => {
                let addrs: Vec<SocketAddr> = addrs.collect();
                let picked = if use_ipv6 {
                    addrs.first().copied()
                } else {
                    addrs.iter().find(|a| a.ip().is_ipv4()).copied()
                };
                if let Some(addr) = picked {
                    let ip_str = addr.ip().to_string();
                    debug!("Resolved server {} to IP {}", server_host, ip_str);
                    if let Ok(mut write_guard) = cache.write() {
                        write_guard.insert(cache_key, (ip_str.clone(), Instant::now()));
                    }
                    server_ip = Some(ip_str);
                } else if !addrs.is_empty() && !use_ipv6 {
                    warn!("No IPv4 for {}, using hostname; outbound dial uses IPv4-only", server_host);
                } else {
                    warn!("DNS resolution returned no addresses for {}", server_host);
                }
            }
            Err(e) => {
                warn!("Failed to resolve server IP manually: {}. Fallback to domain.", e);
            }
        }
    }

    let connection_address = server_ip.clone().unwrap_or_else(|| server_host.to_string());

    // 3. МАРШРУТИЗАТОР ПРОТОКОЛОВ с учётом allow_insecure
    let mut outbound_proxy = match scheme {
        "vless" => generate_vless_outbound(&url, &connection_address, port, server_host, routing.allow_insecure)?,
        "wireguard" | "warp" => generate_wireguard_outbound(&url, &connection_address, port)?,
        "trojan" => generate_trojan_outbound(&url, &connection_address, port, server_host, routing.allow_insecure)?,
        "hysteria2" | "hy2" => generate_hysteria2_outbound(&url, &connection_address, port, server_host, routing.allow_insecure)?,
        _ => return Err(anyhow!("Unsupported protocol scheme: {}", scheme)),
    };

    apply_outbound_domain_strategy(&mut outbound_proxy, use_ipv6);

    // --- ДИНАМИЧЕСКАЯ ГЕНЕРАЦИЯ DNS СЕРВЕРА (HTTP удалён) ---
    let dns_server = match routing.dns_type() {
        pulsar_shared::vpn::DnsType::DnsUdp => json!({
            "tag": "dns-remote",
            "type": "udp",
            "server": "1.1.1.1",
            "server_port": 53,
            "detour": "proxy"
        }),
        _ => json!({ // DnsHttps (и DnsHttp как fallback)
            "tag": "dns-remote",
            "type": "https",
            "server": "1.1.1.1",
            "server_port": 443,
            "detour": "proxy"
        }),
    };

    let dns = json!({
        "servers": [
            dns_server,
            { "tag": "dns-local", "type": "local", "detour": "direct" }
        ],
        "rules": [
            { "domain": [ server_host ], "server": "dns-local" },
            { "domain_suffix": [ ".local" ], "server": "dns-local" },
        ],
        "final": "dns-remote",
        "strategy": if use_ipv6 { "prefer_ipv4" } else { "ipv4_only" }
    });

    // --- Routing Rules ---
    let mut direct_ip_cidrs = vec![
        "127.0.0.0/8".to_string(), "::1/128".to_string(), "10.0.0.0/8".to_string(),
        "192.168.0.0/16".to_string(), "172.16.0.0/12".to_string(),
        "fc00::/7".to_string(), "fe80::/10".to_string()
    ];

    if let Some(ip) = server_ip {
        direct_ip_cidrs.push(format!("{}/{}", ip, if ip.contains(':') { 128 } else { 32 }));
    }

    let mut rules = vec![
        json!({ "protocol": "dns", "action": "hijack-dns" }),
        // Собственные процессы — напрямую, чтобы gRPC control-канал
        // Tauri ↔ daemon не зациклился через TUN.
        json!({ "process_name": SELF_PROCESS_NAMES, "action": "route", "outbound": "direct" }),
        json!({ "ip_cidr": direct_ip_cidrs, "action": "route", "outbound": "direct" }),
        json!({ "domain": [ "localhost", server_host ], "action": "route", "outbound": "direct" })
    ];

    let mode = routing.mode();
    let target = if mode == PbMode::Proxy { "proxy" } else { "direct" };

    // ── Домены пользователя (Bypass / Proxy) ────────────────────────
    // Используем `domain_suffix`, чтобы запись `google.com` покрывала
    // и `mail.google.com`, и `accounts.google.com`. Раньше был
    // `domain` (exact-match) — это вводило пользователей в заблуждение.
    if !routing.domains.is_empty() && mode != PbMode::Global {
        rules.push(json!({
            "domain_suffix": &routing.domains,
            "action": "route",
            "outbound": target
        }));
    }

    // ── Приложения (только там, где process_name работает) ─────────
    // На Android VpnService скрывает PID → sing-box игнорирует правило.
    // apps-list на Android должна потребляться Kotlin'ом через
    // VpnService.Builder.addAllowedApplication/Disallowed.
    if !routing.apps.is_empty() && PROCESS_NAME_ROUTING_SUPPORTED {
        rules.push(json!({
            "process_name": &routing.apps,
            "action": "route",
            "outbound": target
        }));
        if !routing.apps.is_empty() {
            debug!("Per-app routing: emitted process_name rule for {} apps (mode={:?})",
                routing.apps.len(), mode);
        }
    } else if !routing.apps.is_empty() {
        debug!("apps list provided but process_name is not supported on this OS \
                (Android: apps are routed via VpnService.Builder at native side)");
    }

    // ── GeoIP через rule_set (sing-box 1.12+) ──────────────────────
    let mut rule_sets = Vec::new();
    let mut geoip_dns_rules: Vec<Value> = Vec::new();

    if routing.enable_geoip && !routing.geoip_rules.is_empty() {
        for (country_code, action) in &routing.geoip_rules {
            // Нормализуем код в нижний регистр (SagerNet repo хранит
            // файлы в lowercase; proto-комментарий говорит "ru", но UI
            // исторически отправлял "RU").
            let cc = country_code.to_lowercase();
            let outbound = match action.as_str() {
                "proxy" => "proxy",
                "block" => "block",
                _ => "direct",
            };
            let tag = format!("geoip-{}", cc);

            if !rule_sets.iter().any(|rs: &Value| rs["tag"] == tag) {
                let url = format!(
                    "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/{}.srs",
                    tag
                );
                rule_sets.push(json!({
                    "tag": tag,
                    "type": "remote",
                    "format": "binary",
                    "url": url,
                    "download_detour": "direct",
                    "update_interval": "24h",
                }));
            }

            // ── DNS routing mirror ──
            // Если страна помечена `direct` — резолвим домены для её IP-диапазонов
            // через dns-local (иначе 1.1.1.1 через proxy видит все DNS-запросы).
            // Если `block` — отбрасываем DNS-запросы (action: "reject" поддерживается
            // sing-box dns rules).
            // `proxy` — оставляем по умолчанию (через proxy), DNS-зеркало не нужно.
            if outbound == "direct" {
                geoip_dns_rules.push(json!({
                    "rule_set": tag,
                    "server": "dns-local"
                }));
            } else if outbound == "block" {
                geoip_dns_rules.push(json!({
                    "rule_set": tag,
                    "action": "reject"
                }));
            }

            rules.push(json!({
                "rule_set": tag,
                "action": "route",
                "outbound": outbound
            }));
        }
    }

    // ── DNS-зеркало: в режиме Bypass домены пользователя резолвятся напрямую ──
    // Раньше DNS-запросы для bypass-доменов шли через 1.1.1.1-over-proxy
    // (прокси-оператор видел, какие сайты пользователь открывает, хотя
    // сам трафик шёл direct). Теперь mirror'им список в dns.rules.
    let mut dns_rules: Vec<Value> = dns
        .get("rules")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if mode == PbMode::Bypass && !routing.domains.is_empty() {
        for d in &routing.domains {
            dns_rules.push(json!({ "domain_suffix": [d], "server": "dns-local" }));
        }
    }
    // DNS-зеркала для GeoIP-правил (`direct` → dns-local, `block` → reject)
    dns_rules.extend(geoip_dns_rules);

    // Соберём финальный dns с расширенными правилами
    let mut dns_obj = dns.as_object().cloned().unwrap_or_default();
    dns_obj.insert("rules".to_string(), json!(dns_rules));
    let dns_final = Value::Object(dns_obj);

    let final_route = if mode == PbMode::Proxy { "direct" } else { "proxy" };

    let tun_addresses = if use_ipv6 {
        json!(["172.19.0.1/30", "fdfe:dcba:9876::1/126"])
    } else {
        json!(["172.19.0.1/30"])
    };

    let mut route_val = json!({
        "rules": rules,
        "final": final_route,
        "auto_detect_interface": true
    });
    if !use_ipv6 {
        if let Some(obj) = route_val.as_object_mut() {
            obj.insert(
                "default_domain_resolver".to_string(),
                json!({ "server": "dns-remote", "strategy": "ipv4_only" }),
            );
        }
    }
    if !rule_sets.is_empty() {
        if let Some(obj) = route_val.as_object_mut() {
            obj.insert("rule_set".to_string(), json!(rule_sets));
        }
    }

    // --- ОСНОВНОЙ ОБЪЕКТ КОНФИГА ---
    let config = json!({
        "log": { "level": "trace", "timestamp": true },
        "dns": dns_final,
        "inbounds": [{
            "type": "tun", "tag": "tun-in", "interface_name": "pulsar-tun",
            "address": tun_addresses,
            "mtu": 1280, "auto_route": true, "strict_route": true,
            "stack": "mixed", "sniff": true, "endpoint_independent_nat": true,
            "route_exclude_address": ["127.0.0.0/8", "::1/128", "10.0.0.0/8", "192.168.0.0/16", "172.16.0.0/12", "fc00::/7", "fe80::/10"]
        }],
        "outbounds": [ outbound_proxy, { "type": "direct", "tag": "direct" }, { "type": "block", "tag": "block" } ],
        "route": route_val,
        "experimental": {
            "cache_file": { "enabled": true }
        }
    });

    let config_str = serde_json::to_string_pretty(&config)?;
    trace!("--- SING-BOX CONFIG DUMP ---\n{}\n----------------------------", config_str);

    Ok(config_str)
}