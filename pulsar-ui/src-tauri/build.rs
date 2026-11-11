use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
    let binaries = Path::new(&manifest_dir).join("binaries");
    let daemon = binaries.join(if cfg!(windows) {
        "pulsar-daemon.exe"
    } else {
        "pulsar-daemon"
    });
    let xray = binaries.join(if cfg!(windows) { "xray.exe" } else { "xray" });

    println!("cargo:rerun-if-changed={}", binaries.display());
    println!("cargo:rerun-if-env-changed=PULSAR_SKIP_DAEMON_CHECK");

    if std::env::var("PULSAR_SKIP_DAEMON_CHECK").ok().as_deref() != Some("1") && !daemon.exists() {
        panic!(
            "pulsar-daemon was not found at {}. Run `cargo xtask build` or set PULSAR_SKIP_DAEMON_CHECK=1 for checks",
            daemon.display()
        );
    }
    if !xray.exists() {
        println!(
            "cargo:warning=Xray was not found at {}; the application will build but VPN cannot start",
            xray.display()
        );
    }

    tauri_build::build()
}
