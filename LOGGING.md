# Система логирования Pulsar VPN

## Обзор

Внедрена всеобъемлющая система логирования для всех компонентов приложения с поддержкой:
- Ротации логов по дням
- Хранения 30 дней
- Request-ID для трассировки запросов
- Уровней логирования (TRACE, DEBUG, INFO, WARN, ERROR, FATAL)
- Защиты чувствительных данных

## Расположение логов

Все логи сохраняются в: `%LOCALAPPDATA%\PulsarVPN\logs\`

- `pulsar-daemon-YYYY-MM-DD.log` - логи демона
- `pulsar-ui-YYYY-MM-DD.log` - логи Tauri backend и Vue frontend

## Уровни логирования

### Настройка уровня

Установите переменную окружения `PULSAR_LOG_LEVEL`:
```bash
# Windows PowerShell
$env:PULSAR_LOG_LEVEL="debug"

# Windows CMD
set PULSAR_LOG_LEVEL=debug
```

Доступные уровни: `trace`, `debug`, `info`, `warn`, `error`, `fatal`

По умолчанию: `info`

### Vue Frontend

В Vue также можно установить уровень через localStorage:
```javascript
localStorage.setItem('pulsar_log_level', 'debug')
```

## Формат логов

```
[2025-02-22T10:30:45.123Z] [INFO] [pulsar-daemon::main] toggle_connection RPC called request_id=abc-123
```

Формат: `[timestamp] [level] [module::file] message [key=value ...]`

## Request-ID

Каждый gRPC запрос от UI к демону получает уникальный `request-id` (UUID), который передается через метаданные и логируется во всех связанных операциях. Это позволяет отслеживать полный путь запроса через систему.

## Что логируется

### Демон (pulsar-daemon)

- ✅ Запуск/остановка демона
- ✅ Все gRPC вызовы (toggle_connection, get_status, stream_metrics, shutdown)
- ✅ Генерация конфига sing-box (параметры, режим маршрутизации)
- ✅ Запуск/мониторинг процесса sing-box (PID, статус)
- ✅ Работа с файлами (создание директорий, запись конфига)
- ✅ Ошибки с контекстом (пути файлов, коды ошибок)
- ✅ Метрики сети (интерфейсы, скорость)

### Tauri Backend

- ✅ Запуск приложения
- ✅ Все Tauri команды (toggle_vpn, start_metrics_stream, check_ping, fetch_subscription)
- ✅ Запуск демона (PID, путь к исполняемому файлу)
- ✅ Подключение к gRPC серверу (попытки, успех)
- ✅ HTTP запросы (подписки, пинг)
- ✅ Ошибки с деталями

### Vue Frontend

- ✅ Действия пользователя (клики кнопок, выбор серверов)
- ✅ Изменения состояния (подключение/отключение VPN)
- ✅ Импорт профилей (количество, источник)
- ✅ Изменения настроек (kill switch, DNS leak protection, автозапуск)
- ✅ HTTP запросы (подписки)
- ✅ Ошибки пользовательского интерфейса

## Защита чувствительных данных

**НЕ логируются:**
- ❌ Полные токены аутентификации (только длина)
- ❌ UUID профилей в открытом виде
- ❌ Содержимое трафика пользователя
- ❌ Пароли (если будут добавлены)

**Логируются безопасно:**
- ✅ Длина токенов
- ✅ Хосты и порты серверов
- ✅ Режимы маршрутизации
- ✅ Имена процессов
- ✅ Пути к файлам

## Ротация и очистка

- **Ротация:** По дням (новый файл каждый день)
- **Хранение:** 30 дней
- **Очистка:** Автоматическая при старте приложения (фоновая задача)

## Примеры использования

### Rust (демон/Tauri)

```rust
use tracing::{info, debug, error, warn};

info!(request_id = ?req_id, "Operation started");
debug!(param = value, "Detailed information");
warn!(error = %e, "Warning occurred");
error!(error = %e, "Operation failed");
```

### Vue/TypeScript

```typescript
import {logger} from './utils/logger'

logger.info('Module', 'User clicked connect button')
logger.debug('Module', 'State changed', {connected: true})
logger.error('Module', 'Failed to connect', error)
```

## Диагностика проблем

1. **Проблема с подключением VPN:**
   - Проверьте логи демона на ошибки генерации конфига
   - Проверьте логи Tauri на ошибки gRPC
   - Найдите request-id в логах и отследите весь путь запроса

2. **Метрики показывают 0:**
   - Проверьте логи stream_metrics на наличие интерфейсов
   - Убедитесь, что TUN интерфейс создан (логи sing-box)

3. **Сайты не грузятся:**
   - Проверьте логи генерации конфига (маршрутизация, DNS)
   - Проверьте логи sing-box (если доступны)

## Переменные окружения

- `PULSAR_LOG_LEVEL` - уровень логирования (trace/debug/info/warn/error/fatal)

## Структура логов

```
%LOCALAPPDATA%\PulsarVPN\logs\
├── pulsar-daemon-2025-02-22.log
├── pulsar-daemon-2025-02-23.log
├── pulsar-ui-2025-02-22.log
└── pulsar-ui-2025-02-23.log
```

Старые файлы (>30 дней) автоматически удаляются при старте приложения.
