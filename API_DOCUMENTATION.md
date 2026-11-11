# Pulsar VPN - API Documentation

Автоматически сгенерированная документация публичных функций проекта.

---

## 📦 Rust Backend

### pulsar-daemon (gRPC Server)

#### Модуль: `main.rs`

##### `check_token<T>`
```rust
fn check_token<T>(request: &Request<T>, expected: Option<&Arc<str>>) -> Result<(), Status>
```
**Описание:** Проверяет токен аутентификации в метаданных gRPC запроса.

**Параметры:**
- `request` - gRPC запрос
- `expected` - ожидаемый токен (опционально)

**Возвращает:** `Ok(())` если токен валиден, иначе `Status::unauthenticated`

---

##### `toggle_connection` (gRPC метод)
```rust
async fn toggle_connection(&self, request: Request<ConnectionRequest>) -> Result<Response<StatusResponse>, Status>
```
**Описание:** Подключает или отключает VPN соединение.

**Параметры:**
- `connect: bool` - true для подключения, false для отключения
- `profile_link: String` - ссылка на профиль (vless://, wireguard://, и т.д.)
- `routing_config: RoutingConfig` - настройки маршрутизации

**Возвращает:** `StatusResponse` с состоянием ("Connected", "Disconnected", "Error")

**Действия:**
- Генерирует конфиг sing-box
- Запускает/останавливает процесс sing-box.exe
- Перехватывает stdout/stderr процесса для логирования

---

##### `get_status` (gRPC метод)
```rust
async fn get_status(&self, request: Request<Empty>) -> Result<Response<StatusResponse>, Status>
```
**Описание:** Возвращает текущий статус VPN соединения.

**Возвращает:** `StatusResponse` с состоянием "Connected" или "Disconnected"

---

##### `stream_metrics` (gRPC метод)
```rust
async fn stream_metrics(&self, request: Request<Empty>) -> Result<Response<Self::StreamMetricsStream>, Status>
```
**Описание:** Стримит метрики скорости загрузки/выгрузки в реальном времени.

**Возвращает:** Поток `Metrics` с полями:
- `download_speed: u64` - скорость загрузки (байт/сек)
- `upload_speed: u64` - скорость выгрузки (байт/сек)

**Частота:** Обновление каждую секунду

---

##### `shutdown` (gRPC метод)
```rust
async fn shutdown(&self, request: Request<Empty>) -> Result<Response<Empty>, Status>
```
**Описание:** Корректно завершает работу демона.

**Действия:**
- Останавливает процесс sing-box
- Отправляет сигнал завершения серверу

---

#### Модуль: `config_gen.rs`

##### `generate_singbox_config`
```rust
pub fn generate_singbox_config(link: &str, routing: &RoutingConfig) -> Result<String>
```
**Описание:** Генерирует конфигурационный файл для sing-box на основе профиля.

**Параметры:**
- `link` - URL профиля (vless://, trojan://, hysteria2://, wireguard://)
- `routing` - настройки маршрутизации (режим, домены, приложения, GeoIP)

**Возвращает:** JSON строку с конфигурацией sing-box

**Поддерживаемые протоколы:**
- VLESS (с Reality и TLS)
- WireGuard
- Trojan
- Hysteria2

**Особенности:**
- Кэширование DNS запросов (5 минут)
- Автоматическое определение IP сервера
- Генерация правил маршрутизации
- Поддержка GeoIP через rule_set

---

##### `generate_vless_outbound`
```rust
fn generate_vless_outbound(
    url: &Url,
    connection_address: &str,
    port: u16,
    server_host: &str,
    allow_insecure: bool,
) -> Result<Value>
```
**Описание:** Генерирует outbound конфигурацию для VLESS протокола.

**Поддерживает:**
- Reality (с public_key и short_id)
- TLS с UTLS fingerprint
- Flow control (xtls-rprx-vision)

---

##### `generate_wireguard_outbound`
```rust
fn generate_wireguard_outbound(url: &Url, connection_address: &str, port: u16) -> Result<Value>
```
**Описание:** Генерирует outbound конфигурацию для WireGuard протокола.

**Параметры из URL:**
- `private_key` / `pk` - приватный ключ
- `public_key` / `peer_key` - публичный ключ сервера
- `address` - локальный IP адрес

---

##### `generate_trojan_outbound`
```rust
fn generate_trojan_outbound(
    url: &Url,
    connection_address: &str,
    port: u16,
    server_host: &str,
    allow_insecure: bool,
) -> Result<Value>
```
**Описание:** Генерирует outbound конфигурацию для Trojan протокола.

---

##### `generate_hysteria2_outbound`
```rust
fn generate_hysteria2_outbound(
    url: &Url,
    connection_address: &str,
    port: u16,
    server_host: &str,
    allow_insecure: bool,
) -> Result<Value>
```
**Описание:** Генерирует outbound конфигурацию для Hysteria2 протокола.

**Поддерживает:**
- Salamander обфускацию
- Настройку bandwidth (up_mbps, down_mbps)

---

#### Модуль: `logger.rs`

##### `init_logger`
```rust
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>>
```
**Описание:** Инициализирует систему логирования для демона.

**Настройки:**
- Ротация логов по дням
- Путь: `%LOCALAPPDATA%\PulsarVPN\logs\pulsar-daemon-YYYY-MM-DD.log`
- Формат: timestamp, уровень, модуль, файл:строка, thread, сообщение
- Уровень из `PULSAR_LOG_LEVEL` или INFO по умолчанию
- Автоматическая очистка логов старше 30 дней

---

##### `cleanup_old_logs`
```rust
async fn cleanup_old_logs(log_dir: &PathBuf)
```
**Описание:** Удаляет лог-файлы старше 30 дней.

---

### pulsar-ui (Tauri Application)

#### Модуль: `src-tauri/src/main.rs`

##### `toggle_vpn` (Tauri Command)
```rust
#[tauri::command]
async fn toggle_vpn(payload: TogglePayload, state: State<'_, VpnState>, app: tauri::AppHandle) -> Result<String, String>
```
**Описание:** Подключает/отключает VPN через gRPC клиент.

**Параметры:**
- `connect: bool` - флаг подключения
- `link: String` - URL профиля
- `routing_config: UIRoutingConfig` - настройки маршрутизации

**Возвращает:** Статус ("Connected", "Disconnected", "Error")

**Действия:**
- Конвертирует UI конфиг в gRPC запрос
- Отправляет уведомление о статусе
- Логирует операцию

---

##### `start_metrics_stream` (Tauri Command)
```rust
#[tauri::command]
async fn start_metrics_stream(app: tauri::AppHandle, state: State<'_, VpnState>) -> Result<(), String>
```
**Описание:** Запускает стриминг метрик скорости от демона.

**События:** Отправляет событие `vpn_metrics` с данными:
```json
{
  "download": number,
  "upload": number
}
```

---

##### `check_ping` (Tauri Command)
```rust
#[tauri::command]
async fn check_ping(link: String) -> Result<u32, String>
```
**Описание:** Проверяет пинг до сервера из URL профиля.

**Параметры:**
- `link` - URL профиля

**Возвращает:** Пинг в миллисекундах

**Таймаут:** 3 секунды

---

##### `fetch_subscription` (Tauri Command)
```rust
#[tauri::command]
async fn fetch_subscription(url: String) -> Result<SubscriptionResponse, String>
```
**Описание:** Загружает подписку по URL и парсит узлы.

**Параметры:**
- `url` - URL подписки

**Возвращает:**
- `nodes: Vec<String>` - список URL профилей
- `userinfo: Option<String>` - информация о трафике из заголовка

**Форматы:** Base64 или plain text

---

##### `set_minimize_to_tray` (Tauri Command)
```rust
#[tauri::command]
fn set_minimize_to_tray(enable: bool)
```
**Описание:** Включает/выключает сворачивание в трей.

---

##### `log_message` (Tauri Command)
```rust
#[tauri::command]
fn log_message(level: String, module: String, message: String, request_id: Option<String>)
```
**Описание:** Логирует сообщение из фронтенда в файл.

**Параметры:**
- `level` - уровень логирования (TRACE, DEBUG, INFO, WARN, ERROR, FATAL)
- `module` - имя модуля
- `message` - сообщение
- `request_id` - ID запроса (опционально)

---

#### Модуль: `src-tauri/src/logger.rs`

##### `init_logger`
```rust
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>>
```
**Описание:** Инициализирует логирование для UI приложения.

**Настройки:**
- Ротация по дням
- Путь: `%LOCALAPPDATA%\PulsarVPN\logs\pulsar-ui-YYYY-MM-DD.log`
- Защита от повторной инициализации через `try_init()`

---

#### Модуль: `src-tauri/src/lib.rs`

##### `greet`
```rust
#[tauri::command]
fn greet(name: &str) -> String
```
**Описание:** Тестовая команда, возвращает приветствие.

---

##### `run`
```rust
pub fn run()
```
**Описание:** Точка входа для Tauri приложения (mobile entry point).

---

## 🎨 TypeScript Frontend

### Модуль: `logger.ts`

#### Класс: `Logger`

```typescript
class Logger
```

**Методы:**

##### `trace`
```typescript
trace(module: string, message: string, ...args: any[]): void
```
Логирование на уровне TRACE.

---

##### `debug`
```typescript
debug(module: string, message: string, ...args: any[]): void
```
Логирование на уровне DEBUG.

---

##### `info`
```typescript
info(module: string, message: string, ...args: any[]): void
```
Логирование на уровне INFO.

---

##### `warn`
```typescript
warn(module: string, message: string, ...args: any[]): void
```
Логирование на уровне WARN.

---

##### `error`
```typescript
error(module: string, message: string, ...args: any[]): void
```
Логирование на уровне ERROR.

---

##### `fatal`
```typescript
fatal(module: string, message: string, ...args: any[]): void
```
Логирование на уровне FATAL.

---

##### `setLevel`
```typescript
setLevel(level: LogLevel): void
```
Устанавливает минимальный уровень логирования.

---

##### `getRequestId`
```typescript
getRequestId(): string | null
```
Возвращает ID текущего запроса.

---

#### Экземпляр: `logger`
```typescript
export const logger = new Logger();
```
Глобальный экземпляр логгера.

---

### Модуль: `App.vue`

#### Функции состояния

##### `getFilteredAndSortedNodes`
```typescript
const getFilteredAndSortedNodes = (nodes: Profile[]): Profile[]
```
**Описание:** Фильтрует и сортирует список профилей.

**Параметры:**
- `nodes` - массив профилей

**Возвращает:** Отфильтрованный и отсортированный массив

---

##### `toggleSort`
```typescript
const toggleSort = (field: 'name' | 'ping'): void
```
**Описание:** Переключает сортировку по полю.

---

##### `addLog`
```typescript
const addLog = (msg: string): void
```
**Описание:** Добавляет сообщение в лог-консоль UI.

---

##### `toggleAutoStart`
```typescript
const toggleAutoStart = async (): Promise<void>
```
**Описание:** Включает/выключает автозапуск приложения.

---

##### `toggleTray`
```typescript
const toggleTray = async (): Promise<void>
```
**Описание:** Переключает режим сворачивания в трей.

---

##### `refreshPings`
```typescript
const refreshPings = async (): Promise<void>
```
**Описание:** Обновляет пинги для всех профилей.

---

##### `toggleVpn`
```typescript
const toggleVpn = async (): Promise<void>
```
**Описание:** Подключает/отключает VPN.

**Действия:**
- Проверяет наличие активного профиля
- Вызывает `toggle_vpn` через Tauri
- Обновляет состояние подключения
- Запускает стриминг метрик

---

##### `saveData`
```typescript
const saveData = async (): Promise<void>
```
**Описание:** Сохраняет данные в Tauri Store.

---

##### `openEditModal`
```typescript
const openEditModal = (profile: Profile): void
```
**Описание:** Открывает модальное окно настроек профиля.

---

##### `saveProfileSettings`
```typescript
const saveProfileSettings = async (): Promise<void>
```
**Описание:** Сохраняет настройки профиля (DNS тип, allow_insecure).

---

##### `completeWizard`
```typescript
const completeWizard = async (): Promise<void>
```
**Описание:** Завершает мастер первоначальной настройки.

---

##### `handleWizardImport`
```typescript
const handleWizardImport = async (): Promise<void>
```
**Описание:** Импортирует подписку в мастере настройки.

---

##### `addDomain` / `removeDomain`
```typescript
const addDomain = (): void
const removeDomain = (idx: number): void
```
**Описание:** Добавляет/удаляет домен из списка маршрутизации.

---

##### `addApp` / `removeApp`
```typescript
const addApp = (): void
const removeApp = (idx: number): void
```
**Описание:** Добавляет/удаляет приложение из списка маршрутизации.

---

##### `setActiveProfile`
```typescript
const setActiveProfile = async (id: number): Promise<void>
```
**Описание:** Устанавливает активный профиль.

---

##### `deleteProfile`
```typescript
const deleteProfile = async (id: number): Promise<void>
```
**Описание:** Удаляет профиль.

---

##### `deleteSubscription`
```typescript
const deleteSubscription = async (id: string): Promise<void>
```
**Описание:** Удаляет подписку со всеми профилями.

---

##### `parseLocationFromName`
```typescript
const parseLocationFromName = (name: string): string
```
**Описание:** Извлекает страну из названия профиля по флагу эмодзи.

---

##### `createProfileObj`
```typescript
const createProfileObj = (url: string, subId?: string): Profile | null
```
**Описание:** Создает объект профиля из URL.

**Парсит:**
- Протокол (VLESS, Trojan, и т.д.)
- Название из URL параметров
- Страну из флага эмодзи
- Хост и порт

---

##### `handleImport`
```typescript
const handleImport = async (): Promise<void>
```
**Описание:** Импортирует подписку или отдельный профиль.

**Поддерживает:**
- URL подписки (Base64)
- Одиночный URL профиля

---

##### `refreshSubscription`
```typescript
const refreshSubscription = async (sub: Subscription): Promise<void>
```
**Описание:** Обновляет подписку с сервера.

---

##### `connectToLocation`
```typescript
const connectToLocation = (countryName: string): void
```
**Описание:** Быстрое подключение к серверу по стране.

---

### Модуль: `PulsarSphere.vue`

3D визуализация состояния VPN (Vue компонент с Three.js).

**Props:**
- `speed: number` - текущая скорость
-