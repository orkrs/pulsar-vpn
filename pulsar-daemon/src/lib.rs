//! Pulsar VPN daemon — переиспользуемое ядро.
//!
//! Этот crate предоставляет бизнес-логику VPN в виде библиотеки, которая
//! используется в двух сценариях:
//!
//! 1. **Windows**: бинарь `pulsar-daemon` поднимает gRPC-сервер на
//!    `127.0.0.1:50051` и оборачивает [`VpnEngine`] в tonic-сервис.
//!    Tauri-приложение на Windows общается с ним по gRPC.
//!
//! 2. **Android**: Tauri-приложение линкуется с этой библиотекой
//!    напрямую (через cargo feature `android-engine`) и использует
//!    [`VpnEngine`] in-process, без gRPC. На Android движок работает
//!    через Android `VpnService` + нативный sing-box, см.
//!    [`vpn_engine::android`].
//!
//! Публичный API библиотеки намеренно узкий:
//! - [`config_gen`] — генерация JSON-конфига sing-box
//! - [`vpn_engine`] — основной движок (состояние, процесс, метрики)
//! - [`logger`] — инициализация логирования (используется бинарём)
//!
//! Этих трёх модулей достаточно и gRPC-обёртке, и будущему Android-интерфейсу.

pub mod config_gen;
pub mod logger;
pub mod vpn_engine;
