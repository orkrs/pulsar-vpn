use anyhow::{Context, Result, anyhow};
use pulsar_shared::vpn::RoutingConfig;
use serde_json::{Value, json};
use std::net::{IpAddr, ToSocketAddrs};
use url::Url;

pub fn generate_xray_config(link: &str, routing: &RoutingConfig) -> Result<String> {
    let url = Url::parse(link).context("invalid profile link")?;
    let proxy = match url.scheme() {
        "hy2" | "hysteria2" => hysteria_outbound(&url, routing.allow_insecure)?,
        "vless" => vless_outbound(&url)?,
        scheme => return Err(anyhow!("unsupported protocol: {scheme}")),
    };

    let selected_tag = if routing.mode == 2 { "proxy" } else { "direct" };
    let default_tag = if routing.mode == 2 { "direct" } else { "proxy" };
    let mut rules = vec![json!({
        "type": "field", "ip": ["geoip:private"], "outboundTag": "direct"
    })];

    if !routing.domains.is_empty() {
        let domains: Vec<_> = routing
            .domains
            .iter()
            .map(|domain| format!("domain:{domain}"))
            .collect();
        rules.push(json!({
            "type": "field", "domain": domains, "outboundTag": selected_tag
        }));
    }
    if !routing.apps.is_empty() {
        rules.push(json!({
            "type": "field", "process": routing.apps, "outboundTag": selected_tag
        }));
    }
    if routing.enable_geoip {
        for (country, action) in &routing.geoip_rules {
            if matches!(action.as_str(), "proxy" | "direct" | "block") {
                rules.push(json!({
                    "type": "field",
                    "ip": [format!("geoip:{}", country.to_lowercase())],
                    "outboundTag": action
                }));
            }
        }
    }
    rules.push(json!({
        "type": "field", "network": "tcp,udp", "outboundTag": default_tag
    }));

    let mut tun = json!({
        "name": "pulsar-tun",
        "mtu": 1280,
        "gateway": ["172.19.0.1/30"],
        "dns": ["1.1.1.1"]
    });
    #[cfg(target_os = "windows")]
    {
        tun["autoSystemRoutingTable"] = json!(["0.0.0.0/0"]);
        tun["autoOutboundsInterface"] = json!(windows_default_interface()?);
    }

    let config = json!({
        "log": { "loglevel": "warning" },
        "dns": {
            "servers": ["https+local://1.1.1.1/dns-query"],
            "queryStrategy": if routing.use_ipv6 { "UseIP" } else { "UseIPv4" }
        },
        "inbounds": [{
            "tag": "tun-in",
            "protocol": "tun",
            "settings": tun,
            "sniffing": {
                "enabled": true,
                "destOverride": ["http", "tls"],
                "routeOnly": true
            }
        }],
        "outbounds": [
            proxy,
            { "tag": "direct", "protocol": "freedom", "settings": {} },
            { "tag": "block", "protocol": "blackhole", "settings": {} }
        ],
        "routing": {
            "domainStrategy": if routing.enable_geoip { "IPIfNonMatch" } else { "AsIs" },
            "rules": rules
        }
    });
    Ok(serde_json::to_string_pretty(&config)?)
}

#[cfg(target_os = "windows")]
fn windows_default_interface() -> Result<String> {
    use std::os::windows::process::CommandExt;

    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-NetRoute -AddressFamily IPv4 -DestinationPrefix '0.0.0.0/0' | Where-Object NextHop -ne '0.0.0.0' | Sort-Object { $_.RouteMetric + $_.InterfaceMetric } | Select-Object -First 1 -ExpandProperty InterfaceAlias",
        ])
        .creation_flags(0x0800_0000)
        .output()
        .context("failed to detect the default network interface")?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || name.is_empty() {
        return Err(anyhow!("could not detect the default network interface"));
    }
    Ok(name)
}

fn hysteria_outbound(url: &Url, allow_insecure: bool) -> Result<Value> {
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("missing server host"))?;
    let auth = url.username();
    if auth.is_empty() {
        return Err(anyhow!("missing Hysteria2 auth"));
    }

    let mut sni = host.to_owned();
    let mut obfs = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "sni" => sni = value.into_owned(),
            "obfs-password" => obfs = Some(value.into_owned()),
            _ => {}
        }
    }

    let mut stream = json!({
        "network": "hysteria",
        "security": "tls",
        "tlsSettings": {
            "serverName": sni,
            "allowInsecure": allow_insecure,
            "alpn": ["h3"]
        },
        "hysteriaSettings": { "version": 2, "auth": auth }
    });
    if let Some(password) = obfs {
        stream["finalmask"] = json!({
            "udp": [{ "type": "salamander", "settings": { "password": password } }]
        });
    }

    Ok(json!({
        "tag": "proxy",
        "protocol": "hysteria",
        "settings": {
            "version": 2,
            "address": resolve_host(host)?,
            "port": url.port().unwrap_or(443),
            "targetStrategy": "UseIPv4"
        },
        "streamSettings": stream
    }))
}

fn resolve_host(host: &str) -> Result<String> {
    if host.parse::<IpAddr>().is_ok() {
        return Ok(host.to_owned());
    }
    if let Ok(mut addresses) = (host, 443).to_socket_addrs() {
        if let Some(ip) = addresses.find(|address| address.is_ipv4()) {
            return Ok(ip.ip().to_string());
        }
    }

    let answer: Value = reqwest::blocking::Client::new()
        .get("https://1.1.1.1/dns-query")
        .header("accept", "application/dns-json")
        .query(&[("name", host), ("type", "A")])
        .send()
        .context("DNS-over-HTTPS request failed")?
        .error_for_status()?
        .json()?;
    answer["Answer"]
        .as_array()
        .and_then(|answers| answers.iter().find_map(|item| item["data"].as_str()))
        .filter(|address| address.parse::<std::net::Ipv4Addr>().is_ok())
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("could not resolve {host}"))
}

fn vless_outbound(url: &Url) -> Result<Value> {
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("missing server host"))?;
    let id = url.username();
    if id.is_empty() {
        return Err(anyhow!("missing VLESS UUID"));
    }

    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let network = params.get("type").map(String::as_str).unwrap_or("tcp");
    let security = params.get("security").map(String::as_str).unwrap_or("none");
    let mut stream = json!({
        "network": network,
        "security": security
    });
    if security == "reality" {
        stream["realitySettings"] = json!({
            "serverName": params.get("sni").map(String::as_str).unwrap_or(host),
            "fingerprint": params.get("fp").map(String::as_str).unwrap_or("chrome"),
            "password": params.get("pbk").context("missing Reality public key")?,
            "shortId": params.get("sid").map(String::as_str).unwrap_or(""),
            "spiderX": params.get("spx").map(String::as_str).unwrap_or("/")
        });
    } else if security == "tls" {
        stream["tlsSettings"] = json!({
            "serverName": params.get("sni").map(String::as_str).unwrap_or(host),
            "allowInsecure": false
        });
    }

    Ok(json!({
        "tag": "proxy",
        "protocol": "vless",
        "settings": {
            "address": host,
            "port": url.port().unwrap_or(443),
            "id": id,
            "encryption": params.get("encryption").map(String::as_str).unwrap_or("none"),
            "flow": params.get("flow").map(String::as_str).unwrap_or("")
        },
        "streamSettings": stream
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_current_xray_tun_mask_and_explicit_default_route() {
        let routing = RoutingConfig {
            mode: 1,
            ..Default::default()
        };
        let generated = generate_xray_config(
            "hy2://secret@127.0.0.1:443?obfs=salamander&obfs-password=mask&sni=example.com",
            &routing,
        )
        .unwrap();
        let config: Value = serde_json::from_str(&generated).unwrap();

        assert_eq!(
            config["inbounds"][0]["settings"]["autoSystemRoutingTable"][0],
            "0.0.0.0/0"
        );
        assert_eq!(
            config["outbounds"][0]["streamSettings"]["finalmask"]["udp"][0]["type"],
            "salamander"
        );
        assert_eq!(
            config["routing"]["rules"]
                .as_array()
                .unwrap()
                .last()
                .unwrap()["outboundTag"],
            "proxy"
        );
        assert!(config.to_string().find("udpmasks").is_none());

        assert_xray_accepts(config);
    }

    #[test]
    fn vless_reality_uses_current_simplified_schema() {
        let config: Value = serde_json::from_str(
            &generate_xray_config(
                "vless://00000000-0000-4000-8000-000000000000@127.0.0.1:443?security=reality&type=tcp&sni=example.com&fp=chrome&pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&sid=0123456789abcdef&flow=xtls-rprx-vision",
                &RoutingConfig::default(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(config["outbounds"][0]["settings"]["address"], "127.0.0.1");
        assert_eq!(
            config["outbounds"][0]["streamSettings"]["realitySettings"]["password"],
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
        assert_xray_accepts(config);
    }

    fn assert_xray_accepts(mut config: Value) {
        let xray = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("xray.exe");
        if xray.exists() {
            let path = std::env::temp_dir().join("pulsar-xray-config-test.json");
            // Parsing a real TUN asks Windows for elevation even with `-test`.
            config["inbounds"] = json!([{
                "tag": "tun-in", "listen": "127.0.0.1", "port": 0,
                "protocol": "socks", "settings": { "udp": true }
            }]);
            std::fs::write(&path, serde_json::to_vec(&config).unwrap()).unwrap();
            let output = std::process::Command::new(xray)
                .args(["run", "-test", "-config"])
                .arg(&path)
                .env(
                    "XRAY_LOCATION_ASSET",
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../pulsar-ui/src-tauri/binaries"),
                )
                .output()
                .unwrap();
            let _ = std::fs::remove_file(path);
            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
