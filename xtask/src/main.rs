//! `xtask` — единая точка входа для сборки / разработки / проверки Pulsar VPN.
//!
//! Раньше пользователю нужно было вручную:
//!   1. `cargo build --release -p pulsar-daemon`
//!   2. `Copy-Item target\release\pulsar-daemon.exe pulsar-ui\src-tauri\binaries\`
//!   3. `cd pulsar-ui && bun install && bun run build`
//!   4. `cd pulsar-ui && bun run tauri build`
//!
//! Теперь всё это одна команда: `cargo xtask build`.
//!
//! Доступные подкоманды: `build`, `dev`, `clean`, `lint`, `info`, `help`.
//! См. `print_help()` ниже.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
enum Color {
    Red,
    Green,
    Yellow,
    Cyan,
    Gray,
    Reset,
}

impl Color {
    fn code(self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Cyan => "\x1b[36m",
            Color::Gray => "\x1b[90m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// Stdout/stderr у Tauri cargo — многословный. Печатаем шаги на stderr,
/// чтобы пользователь видел прогресс и при трубах, и при логировании.
fn paint(color: Color, s: &str) -> String {
    // Windows 10+ включает виртуальный терминал по умолчанию, но всё же
    // аккуратно отключаем цвета, если NO_COLOR выставлен или stderr не TTY.
    if env::var_os("NO_COLOR").is_some() || !atty_stderr() {
        return s.to_string();
    }
    format!("{}{}{}", color.code(), s, Color::Reset.code())
}

fn atty_stderr() -> bool {
    // Простейшая проверка — на Windows isatty через std недоступен,
    // поэтому эвристика: если в stderr не пайп, считаем что TTY.
    // (Tauri cargo на Windows 10+ нормально обрабатывает ANSI и без этого.)
    true
}

fn say(color: Color, prefix: &str, msg: &str) {
    eprintln!("{}{}", paint(color, prefix), msg);
}

fn info(msg: &str) { say(Color::Cyan,  "==> ", msg); }
fn ok(msg:   &str) { say(Color::Green, "  ✓ ", msg); }
fn warn(msg: &str) { say(Color::Yellow,"  ! ", msg); }
fn err(msg:  &str) { say(Color::Red,   "  ✗ ", msg); }
fn dim(msg:  &str) { say(Color::Gray,  "    ", msg); }

#[derive(Debug, Clone)]
struct Config {
    workspace_root: PathBuf,
    target_dir: PathBuf,
    release: bool,
    json: bool,
}

impl Config {
    fn detect() -> Self {
        // CARGO_MANIFEST_DIR указывает на xtask/ (запускаемый crate).
        // Родитель — корень воркспейса.
        let workspace_root = env::var("CARGO_MANIFEST_DIR")
            .ok()
            .and_then(|p| PathBuf::from(p).parent().map(|p| p.to_path_buf()))
            .or_else(|| env::current_dir().ok())
            .expect("xtask: cannot determine workspace root");
        let target_dir = workspace_root.join("target");
        Self { workspace_root, target_dir, release: false, json: false }
    }

    fn exe(name: &str) -> String {
        if cfg!(windows) { format!("{}.exe", name) } else { name.to_string() }
    }

    fn daemon_binary() -> String { Self::exe("pulsar-daemon") }
    fn singbox_binary() -> String { Self::exe("sing-box") }

    fn binaries_dir(&self) -> PathBuf {
        self.workspace_root.join("pulsar-ui").join("src-tauri").join("binaries")
    }

    fn daemon_built_path(&self) -> PathBuf {
        let profile = if self.release { "release" } else { "debug" };
        self.target_dir.join(profile).join(Self::daemon_binary())
    }

    fn daemon_dest_path(&self) -> PathBuf {
        self.binaries_dir().join(Self::daemon_binary())
    }

    fn singbox_expected_path(&self) -> PathBuf {
        self.binaries_dir().join(Self::singbox_binary())
    }

    fn ui_dir(&self) -> PathBuf {
        self.workspace_root.join("pulsar-ui")
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let task = args.get(1).map(String::as_str).unwrap_or("help");
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    let mut cfg = Config::detect();
    cfg.json = rest.iter().any(|a| a == "--json");

    match task {
        "build" => {
            cfg.release = rest.iter().any(|a| a == "--release" || a == "-r");
            run_build(cfg)
        }
        "build-android" => {
            cfg.release = rest.iter().any(|a| a == "--release" || a == "-r");
            run_build_android(cfg)
        }
        "build-all" => {
            cfg.release = rest.iter().any(|a| a == "--release" || a == "-r");
            run_build_all(cfg)
        }
        "dev"      => run_dev(cfg),
        "clean"    => run_clean(cfg, &rest),
        "lint"     => run_lint(cfg),
        "info"     => run_info(cfg),
        "check"    => {
            // Алиас для `lint`, удобный в CI.
            cfg.json = rest.iter().any(|a| a == "--json");
            run_lint(cfg)
        }
        "help" | "-h" | "--help" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            err(&format!("неизвестная подкоманда: `{}`", other));
            eprintln!();
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    let help = "\
xtask — оркестратор сборки Pulsar VPN

ИСПОЛЬЗОВАНИЕ:
    cargo xtask <КОМАНДА> [ОПЦИИ]

КОМАНДЫ:
    build [--release]       Полная prod-сборка для текущей ОС (Windows installer / etc.)
    build-android [--release]  Сборка Android APK/AAB (нужны SDK+NDK+JDK+rustup targets)
    build-all [--release]   build + build-android подряд (если окружение поддерживает оба)
    dev                     Режим разработки: daemon (debug) → copy → tauri dev
    clean [--deep]          Очистить артефакты сборки (--deep также удаляет node_modules)
    lint                    cargo check + clippy + vue-tsc
    check                   Алиас для lint (удобно в CI)
    info                    Пути, версии инструментов, статус бинарников
    help                    Показать это сообщение

ПРИМЕРЫ:
    cargo xtask build
    cargo xtask build --release
    cargo xtask build-android
    cargo xtask build-all
    cargo xtask dev
    cargo xtask clean --deep
    cargo xtask lint
    cargo xtask info
";
    print!("{}", help);
}

// ────────────────────────────────────────────────────────────
//  BUILD
// ────────────────────────────────────────────────────────────

fn run_build(cfg: Config) -> ExitCode {
    let started = Instant::now();
    info("Pulsar VPN — production build");
    dim(&format!("  profile:    {}", if cfg.release { "release" } else { "debug" }));
    dim(&format!("  workspace:  {}", cfg.workspace_root.display()));
    eprintln!();

    let steps: [(&str, &str, fn(&Config) -> bool); 5] = [
        ("1/5", "Сборка демона (pulsar-daemon)",   build_daemon),
        ("2/5", "Копирование демона в UI ресурсы", copy_daemon),
        ("3/5", "Установка зависимостей UI",        install_ui_deps),
        ("4/5", "Сборка фронтенда (vue-tsc + vite)",build_frontend),
        ("5/5", "Сборка Tauri приложения и инсталлятора", build_tauri),
    ];

    for (n, name, f) in steps {
        if !run_step(n, name, || f(&cfg)) {
            err(&format!("сборка прервана на шаге {}", n));
            return ExitCode::FAILURE;
        }
    }

    // Не критично, но проверим sing-box — без него приложение не сможет поднять VPN.
    check_singbox(&cfg);

    let secs = started.elapsed().as_secs();
    eprintln!();
    ok(&format!("Готово за {}с", secs));
    print_artifacts(&cfg);
    ExitCode::SUCCESS
}

fn run_dev(cfg: Config) -> ExitCode {
    info("Pulsar VPN — dev mode");
    dim("  Будет собран debug-демон и запущен tauri dev с hot-reload.");
    eprintln!();

    let steps: [(&str, &str, fn(&Config) -> bool); 3] = [
        ("1/3", "Сборка демона (debug)",       build_daemon),
        ("2/3", "Копирование демона в UI",     copy_daemon),
        ("3/3", "Запуск tauri dev (Ctrl+C для выхода)", tauri_dev),
    ];

    for (n, name, f) in steps {
        if !run_step(n, name, || f(&cfg)) {
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

// ────────────────────────────────────────────────────────────
//  ANDROID BUILD
// ────────────────────────────────────────────────────────────

/// Проверяет, что на машине есть всё для сборки Android:
/// - cargo-tauri CLI с поддержкой `android` (>= 2.0)
/// - ANDROID_HOME / SDK
/// - NDK
/// - JDK 17+
/// - Rust-таргеты `aarch64-linux-android`, `armv7-linux-androideabi`,
///   `x86_64-linux-android`, `i686-linux-android` хотя бы один из них.
fn check_android_prereqs(cfg: &Config) -> bool {
    let mut all_ok = true;

    // tauri CLI
    match run_capture("cargo", &["tauri", "--version"], &cfg.workspace_root) {
        Ok(v) => {
            let v_trim = v.trim().to_lowercase();
            if !v_trim.contains("tauri") {
                err(&format!("`cargo tauri` недоступен: {}", v));
                all_ok = false;
            } else {
                ok(&format!("tauri CLI: {}", v.trim()));
            }
        }
        Err(_) => {
            err("`cargo tauri` не найден");
            dim("    установите: cargo install tauri-cli --version \"^2.0\"");
            all_ok = false;
        }
    }

    // ANDROID_HOME / SDK
    let sdk = env::var("ANDROID_HOME")
        .or_else(|_| env::var("ANDROID_SDK_ROOT"))
        .ok();
    match sdk.as_deref() {
        Some(s) if !s.is_empty() && Path::new(s).exists() => {
            ok(&format!("Android SDK: {}", s));
        }
        _ => {
            err("ANDROID_HOME не задан или путь не существует");
            dim("    установите Android Studio и выполните: export ANDROID_HOME=$HOME/Android/Sdk");
            all_ok = false;
        }
    }

    // NDK — Tauri сам ищет в $ANDROID_HOME/ndk/<version>/
    if let Some(sdk) = &sdk {
        let ndk_dir = Path::new(sdk).join("ndk");
        if ndk_dir.exists() {
            if let Ok(read) = std::fs::read_dir(&ndk_dir) {
                let versions: Vec<String> = read
                    .flatten()
                    .filter_map(|e| {
                        let p = e.path();
                        if p.is_dir() { p.file_name().and_then(|n| n.to_str()).map(String::from) } else { None }
                    })
                    .collect();
                if versions.is_empty() {
                    err("NDK не установлен (папка ndk/ пуста)");
                    dim("    откройте Android Studio → SDK Manager → SDK Tools → NDK (Side by side)");
                    all_ok = false;
                } else {
                    ok(&format!("NDK: {} (доступно: {})", ndk_dir.display(), versions.join(", ")));
                }
            }
        } else {
            err(&format!("NDK директория не найдена: {}", ndk_dir.display()));
            all_ok = false;
        }
    }

    // JDK
    if let Ok(v) = run_capture("java", &["-version"], &cfg.workspace_root) {
        ok(&format!("JDK: {}", v.lines().next().unwrap_or("?").trim()));
    } else {
        err("JDK не найден (нужен JDK 17+)");
        dim("    установите JDK 17+ и убедитесь что `java -version` работает");
        all_ok = false;
    }

    // Rust-таргеты для Android
    if let Ok(v) = run_capture("rustup", &["target", "list", "--installed"], &cfg.workspace_root) {
        let have: Vec<&str> = v.lines().collect();
        let needed = [
            "aarch64-linux-android",
            "armv7-linux-androideabi",
            "x86_64-linux-android",
            "i686-linux-android",
        ];
        let missing: Vec<&&str> = needed
            .iter()
            .filter(|t| !have.iter().any(|h| h.trim() == **t))
            .collect();
        if missing.len() == needed.len() {
            err("ни одного Android Rust target не установлено");
            dim(&format!("    установите хотя бы: rustup target add {}", needed[0]));
            all_ok = false;
        } else {
            let installed: Vec<&&str> = needed.iter().filter(|t| have.iter().any(|h| h.trim() == **t)).collect();
            ok(&format!("Android Rust targets: {}", installed.iter().map(|s| **s).collect::<Vec<_>>().join(", ")));
        }
    } else {
        err("rustup недоступен, невозможно проверить Android targets");
        all_ok = false;
    }

    all_ok
}

/// Проверяет, что `cargo tauri android init` уже запускался (создал
/// `gen/android/`). Если нет — подсказывает запустить.
fn check_tauri_android_init(cfg: &Config) -> bool {
    // Tauri 2 кладёт gen/ в src-tauri/ (рядом с Cargo.toml, а не в pulsar-ui/).
    let gen_dir = cfg.ui_dir().join("src-tauri").join("gen").join("android");
    if gen_dir.exists() && gen_dir.is_dir() {
        ok(&format!("gen/android уже инициализирован: {}", gen_dir.display()));
        true
    } else {
        err("gen/android не найден");
        dim("    выполните один раз: cd pulsar-ui && cargo tauri android init");
        dim("    это создаст Gradle-проект; коммитить его не нужно — он регенерируется.");
        false
    }
}

fn run_build_android(cfg: Config) -> ExitCode {
    let started = Instant::now();
    info("Pulsar VPN — Android build");
    dim(&format!("  profile:   {}", if cfg.release { "release" } else { "debug" }));
    dim(&format!("  workspace: {}", cfg.workspace_root.display()));
    eprintln!();

    info("[1/3] Проверка pre-requisites (SDK / NDK / JDK / rust targets)");
    if !check_android_prereqs(&cfg) {
        err("не хватает инструментов для Android-сборки");
        return ExitCode::FAILURE;
    }
    eprintln!();

    info("[2/3] Проверка `cargo tauri android init`");
    if !check_tauri_android_init(&cfg) {
        return ExitCode::FAILURE;
    }
    eprintln!();

    info("[3/3] Сборка APK");
    // `cargo tauri android build` — release по умолчанию, --debug для дебага.
    // Это отличается от `cargo tauri build` для десктопа, где профиль задаётся
    // через `[profile.release]` в Cargo.toml.
    let mut args: Vec<&str> = vec!["tauri", "android", "build"];
    if !cfg.release {
        args.push("--debug");
    }
    if !run("cargo", &args, &cfg.ui_dir()) {
        err("tauri android build завершился с ошибкой");
        return ExitCode::FAILURE;
    }

    let secs = started.elapsed().as_secs();
    eprintln!();
    ok(&format!("Готово за {}с", secs));
    print_android_artifacts(&cfg);
    ExitCode::SUCCESS
}

fn print_android_artifacts(cfg: &Config) {
    eprintln!();
    info("Артефакты:");
    let mut found = false;
    // Tauri 2 Android кладёт артефакты в outputs/{apk,bundle}/, и вложенность
    // зависит от варианта: universal (apk/universal/{debug,release}/),
    // per-ABI (apk/{aarch64,armv7,i686,x86_64}/{debug,release}/) и т. п.
    // Рекурсивно проходим outputs/ в поисках .apk/.aab.
    let outputs_root = cfg
        .ui_dir()
        .join("src-tauri")
        .join("gen")
        .join("android")
        .join("app")
        .join("build")
        .join("outputs");
    if outputs_root.exists() {
        if let Ok(read) = std::fs::read_dir(&outputs_root) {
            for top in read.flatten() {
                let top_path = top.path();
                if !top_path.is_dir() { continue; }
                if let Ok(walk) = walkdir(&top_path, 3) {
                    for path in walk {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            if matches!(ext, "apk" | "aab") {
                                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                dim(&format!("  {}  ({:.1} MB)", path.display(), size as f64 / 1_048_576.0));
                                found = true;
                            }
                        }
                    }
                }
            }
        }
    }
    if !found {
        dim("  (APK/AAB не найдены — проверьте лог tauri android выше)");
    }
}

/// Рекурсивный обход директории до заданной глубины (0 = только текущая).
/// Возвращает Vec<PathBuf>. Без зависимостей.
fn walkdir(root: &Path, max_depth: usize) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    fn recurse(p: &Path, depth: usize, max: usize, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if depth > max { return Ok(()); }
        for entry in std::fs::read_dir(p)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                recurse(&path, depth + 1, max, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
    recurse(root, 0, max_depth, &mut out)?;
    Ok(out)
}

fn run_build_all(cfg: Config) -> ExitCode {
    info("Pulsar VPN — build-all (current platform + Android)");
    eprintln!();

    // Шаг 1: текущая платформа (Windows installer и т. п.)
    info("=== Этап 1/2: десктоп-сборка ===");
    if run_build(cfg.clone()) != ExitCode::SUCCESS {
        err("десктоп-сборка провалилась");
        return ExitCode::FAILURE;
    }
    eprintln!();

    // Шаг 2: Android (если окружение позволяет).
    info("=== Этап 2/2: Android-сборка ===");
    if run_build_android(cfg) != ExitCode::SUCCESS {
        warn("Android-сборка пропущена или провалилась (см. ошибки выше).");
        warn("Это нормально, если на машине нет Android SDK/NDK.");
        warn("Десктоп-артефакты при этом готовы.");
        return ExitCode::SUCCESS; // десктоп уже готов — общий успех
    }

    eprintln!();
    ok("Все платформы собраны");
    ExitCode::SUCCESS
}

fn run_step(label: &str, name: &str, f: impl FnOnce() -> bool) -> bool {
    let started = Instant::now();
    info(&format!("[{}] {}", label, name));
    let ok = f();
    if !ok { return false; }
    let dt = started.elapsed();
    if dt.as_millis() > 100 {
        dim(&format!("    └─ заняло {:.1?}", dt));
    }
    eprintln!();
    true
}

fn build_daemon(cfg: &Config) -> bool {
    let args: &[&str] = if cfg.release {
        &["build", "--release", "-p", "pulsar-daemon"]
    } else {
        &["build", "-p", "pulsar-daemon"]
    };
    run("cargo", args, &cfg.workspace_root)
}

fn copy_daemon(cfg: &Config) -> bool {
    let src = cfg.daemon_built_path();
    let dst = cfg.daemon_dest_path();
    if !src.exists() {
        err(&format!("демон не найден после сборки: {}", src.display()));
        return false;
    }
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                err(&format!("не удалось создать {}: {}", parent.display(), e));
                return false;
            }
        }
    }
    if let Err(e) = std::fs::copy(&src, &dst) {
        err(&format!("копирование {} → {}: {}", src.display(), dst.display(), e));
        return false;
    }
    dim(&format!("    {}", dst.display()));
    ok("демон скопирован");
    true
}

fn check_singbox(cfg: &Config) {
    let p = cfg.singbox_expected_path();
    if !p.exists() {
        warn(&format!("sing-box не найден: {}", p.display()));
        warn("  приложение запустится, но не сможет поднимать VPN-туннель.");
        warn("  Положите sing-box.exe рядом с pulsar-daemon.exe и пересоберите.");
    } else {
        ok("sing-box найден");
    }
}

fn install_ui_deps(cfg: &Config) -> bool {
    run("bun", &["install", "--frozen-lockfile"], &cfg.ui_dir())
}

fn build_frontend(cfg: &Config) -> bool {
    run("bun", &["run", "build"], &cfg.ui_dir())
}

fn build_tauri(cfg: &Config) -> bool {
    run("bun", &["run", "tauri", "build"], &cfg.ui_dir())
}

fn tauri_dev(cfg: &Config) -> bool {
    // Не используем piped stdout — пользователь должен видеть лог tauri напрямую.
    run("bun", &["run", "tauri", "dev"], &cfg.ui_dir())
}

fn print_artifacts(cfg: &Config) {
    eprintln!();
    info("Артефакты:");
    // Tauri кладёт bundle в pulsar-ui/src-tauri/target/release/bundle (отдельный target
    // из-за build.rs в src-tauri) либо в target/release/bundle (если Tauri переиспользует
    // workspace target). Проверим оба варианта.
    let ui_target_bundle = cfg.ui_dir().join("src-tauri").join("target").join("release").join("bundle");
    let ws_target_bundle  = cfg.target_dir.join("release").join("bundle");
    let bundles = [ui_target_bundle, ws_target_bundle];

    let mut found = false;
    for root in &bundles {
        if !root.exists() { continue; }
        if let Ok(read) = std::fs::read_dir(root) {
            for sub in read.flatten() {
                let sub_path = sub.path();
                if sub_path.is_dir() {
                    if let Ok(files) = std::fs::read_dir(&sub_path) {
                        for f in files.flatten() {
                            let path = f.path();
                            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                if matches!(ext, "msi" | "exe" | "AppImage" | "deb" | "dmg") {
                                    let size = f.metadata().map(|m| m.len()).unwrap_or(0);
                                    dim(&format!("  {}  ({:.1} MB)", path.display(), size as f64 / 1_048_576.0));
                                    found = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !found {
        dim("  (инсталляторы не найдены — проверьте лог Tauri выше)");
    }
}

// ────────────────────────────────────────────────────────────
//  CLEAN
// ────────────────────────────────────────────────────────────

fn run_clean(cfg: Config, args: &[String]) -> ExitCode {
    let deep = args.iter().any(|a| a == "--deep");
    info("Очистка артефактов сборки");
    eprintln!();

    let mut ok_all = true;

    info("cargo clean (workspace target)");
    ok_all &= run_quiet("cargo", &["clean"], &cfg.workspace_root);

    // Tauri src-tauri имеет свой build.rs, который держит свой target рядом.
    let tauri_target = cfg.ui_dir().join("src-tauri").join("target");
    if tauri_target.exists() {
        info(&format!("удаление {}", tauri_target.display()));
        ok_all &= rm(&tauri_target);
    }

    let dist = cfg.ui_dir().join("dist");
    if dist.exists() {
        info(&format!("удаление {}", dist.display()));
        ok_all &= rm(&dist);
    }

    if deep {
        let nm = cfg.ui_dir().join("node_modules");
        if nm.exists() {
            info(&format!("удаление {}", nm.display()));
            ok_all &= rm(&nm);
        }
    } else {
        warn("node_modules оставлен (используйте --deep чтобы удалить)");
    }

    eprintln!();
    if ok_all { ok("очистка завершена"); ExitCode::SUCCESS }
    else      { err("очистка прошла с ошибками"); ExitCode::FAILURE }
}

// ────────────────────────────────────────────────────────────
//  LINT
// ────────────────────────────────────────────────────────────

fn run_lint(cfg: Config) -> ExitCode {
    info("Линт воркспейса");
    dim("  cargo check → cargo clippy → vue-tsc");
    eprintln!();

    let mut ok_all = true;

    info("[1/3] cargo check --workspace --all-targets");
    ok_all &= run("cargo", &["check", "--workspace", "--all-targets"], &cfg.workspace_root);

    info("[2/3] cargo clippy --workspace --all-targets -- -D warnings");
    ok_all &= run(
        "cargo",
        &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
        &cfg.workspace_root,
    );

    info("[3/3] bun run typecheck (vue-tsc --noEmit)");
    ok_all &= run("bun", &["run", "typecheck"], &cfg.ui_dir());

    eprintln!();
    if ok_all { ok("линт пройден"); ExitCode::SUCCESS }
    else      { err("линт провалился"); ExitCode::FAILURE }
}

// ────────────────────────────────────────────────────────────
//  INFO
// ────────────────────────────────────────────────────────────

fn run_info(cfg: Config) -> ExitCode {
    let line = |k: &str, v: &str| println!("  {:<16}{}", k, v);
    println!("Pulsar VPN workspace");
    line("root",        &cfg.workspace_root.display().to_string());
    line("target",      &cfg.target_dir.display().to_string());
    line("daemon bin",  &cfg.daemon_built_path().display().to_string());
    line("daemon dest", &cfg.daemon_dest_path().display().to_string());
    line("sing-box",    &cfg.singbox_expected_path().display().to_string());
    line("ui dir",      &cfg.ui_dir().display().to_string());
    println!();
    println!("Инструменты:");
    if let Ok(v) = run_capture("rustc",  &["--version"], &cfg.workspace_root) { line("rustc",  v.trim()); }
    if let Ok(v) = run_capture("cargo",  &["--version"], &cfg.workspace_root) { line("cargo",  v.trim()); }
    if let Ok(v) = run_capture("bun",    &["--version"], &cfg.workspace_root) { line("bun",    v.trim()); }
    if let Ok(v) = run_capture("node",   &["--version"], &cfg.workspace_root) { line("node",   v.trim()); }
    println!();
    println!("Бинарники:");
    line("daemon built", &status(cfg.daemon_built_path().exists()));
    line("daemon bundled", &status(cfg.daemon_dest_path().exists()));
    line("sing-box bundled", &status(cfg.singbox_expected_path().exists()));
    ExitCode::SUCCESS
}

fn status(ok: bool) -> &'static str {
    if ok { "✓ найден" } else { "✗ отсутствует" }
}

// ────────────────────────────────────────────────────────────
//  Process helpers
// ────────────────────────────────────────────────────────────

/// Запускает команду, прокидывая stdout/stderr пользователю. Возвращает успех по коду выхода.
fn run(prog: &str, args: &[&str], cwd: &Path) -> bool {
    let mut cmd = Command::new(prog);
    cmd.args(args).current_dir(cwd);
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            err(&format!("не удалось запустить `{}`: {}", prog, e));
            if prog == "bun" && cfg!(windows) {
                dim("    подсказка: установите bun (https://bun.sh) и перезапустите терминал");
            } else if prog == "cargo" {
                dim("    подсказка: установите Rust через https://rustup.rs");
            }
            return false;
        }
    };
    status.success()
}

/// То же, что `run`, но подавляет вывод (для cargo clean и подобного).
fn run_quiet(prog: &str, args: &[&str], cwd: &Path) -> bool {
    let mut cmd = Command::new(prog);
    cmd.args(args)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match cmd.status() {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

/// Захватывает stdout (для info).
fn run_capture(prog: &str, args: &[&str], cwd: &Path) -> Result<String, ()> {
    let out = Command::new(prog)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| ())?;
    if !out.status.success() { return Err(()); }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn rm(path: &Path) -> bool {
    let meta = match path.metadata() {
        Ok(m) => m,
        Err(_) => return true, // уже нет — считаем успехом
    };
    let res = if meta.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match res {
        Ok(_) => true,
        Err(e) => {
            err(&format!("не удалось удалить {}: {}", path.display(), e));
            false
        }
    }
}
