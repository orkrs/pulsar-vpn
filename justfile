# Pulsar VPN — шорткаты для самых частых команд.
# Это тонкая обёртка над `cargo xtask`. Если `just` не установлен,
# используйте `cargo xtask <subcommand>` напрямую.

set shell := ["powershell", "-NoProfile", "-Command"]

# По умолчанию — список доступных рецептов.
default:
    @just --list

# Полная prod-сборка (эквивалент `cargo xtask build`).
build:
    cargo xtask build

# Полная prod-сборка в release-режиме.
build-release:
    cargo xtask build --release

# Режим разработки (debug-демон + tauri dev с hot-reload).
dev:
    cargo xtask dev

# cargo check + clippy + vue-tsc.
lint:
    cargo xtask lint

# Чистка артефактов. С `deep=yes` — также node_modules.
clean deep="no":
    cargo xtask clean {{ if deep == "yes" { "--deep" } else { "" } }}

# Показать пути, версии, статус бинарников.
info:
    cargo xtask info

# Справка по xtask.
help:
    cargo xtask help
