// Quick test: generate sing-box config from a hy2 link
use pulsar_shared::vpn::{DnsType, RoutingConfig};
use pulsar_daemon::config_gen::generate_singbox_config;

fn main() {
    let link = "hy2://d110dc919800a1b4@2.27.52.157:443?mport&security=tls&sni=web.max.ru&allowInsecure=true&alpn&obfs=salamander&obfs-password=dd7c80b47f00eb4e#User1";

    let routing = RoutingConfig {
        mode: pulsar_shared::vpn::routing_config::Mode::Global,
        domains: vec![],
        apps: vec![],
        dns_type: DnsType::DnsHttps,
        use_ipv6: false,
        allow_insecure: true,
        enable_geoip: false,
        geoip_rules: Default::default(),
    };

    match generate_singbox_config(link, &routing) {
        Ok(config) => {
            println!("=== GENERATED CONFIG ===");
            println!("{}", config);
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
