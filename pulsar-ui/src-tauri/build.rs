use std::path::Path;

fn main() {
    // ── Проверка наличия сайдкар-бинарников ────────────────────────────────
    //
    // pulsar-daemon.exe и sing-box.exe лежат в ./binaries/ и НЕ собираются
    // через cargo (демон — отдельный workspace-член, sing-box — внешний бинарь).
    // Чтобы пользователь не получал непонятную ошибку Tauri-бандла в самом конце,
    // проверяем наличие демона здесь и падаем с внятным сообщением.
    //
    // Чтобы обойти проверку (например, при `cargo check` в CI, где бинарь
    // ещё не собран), выставьте PULSAR_SKIP_DAEMON_CHECK=1.

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let binaries_dir = Path::new(&manifest_dir).join("binaries");
    let daemon = binaries_dir.join(if cfg!(windows) { "pulsar-daemon.exe" } else { "pulsar-daemon" });
    let singbox = binaries_dir.join(if cfg!(windows) { "sing-box.exe" } else { "sing-box" });

    println!("cargo:rerun-if-changed={}", binaries_dir.display());
    println!("cargo:rerun-if-env-changed=PULSAR_SKIP_DAEMON_CHECK");

    if std::env::var("PULSAR_SKIP_DAEMON_CHECK").ok().as_deref() != Some("1") && !daemon.exists() {
        panic!(
            "\n\n  pulsar-daemon binary not found at: {}\n\n\
             Запустите `cargo xtask build` (или `cargo xtask dev`) из корня\n\
             воркспейса — он соберёт демона и положит его в нужное место.\n\n\
             Чтобы обойти эту проверку (например, для `cargo check`), выставьте\n\
             PULSAR_SKIP_DAEMON_CHECK=1.\n",
            daemon.display(),
        );
    }

    // sing-box — внешний бинарь, не критичный для компиляции Tauri, но без
    // него рантайм не сможет поднять VPN. Просто предупреждаем.
    if !singbox.exists() {
        eprintln!(
            "cargo:warning=sing-box не найден в {}. Приложение соберётся, \
             но VPN-туннель запустить не удастся. Положите sing-box.exe \
             в эту папку и пересоберите.",
            singbox.display()
        );
    }

    tauri_build::build()
}
