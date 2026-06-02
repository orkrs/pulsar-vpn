/**
 * Логирование для Vue frontend
 * Отправляет логи в Tauri backend для записи в файл
 */

export enum LogLevel {
  TRACE = 0,
  DEBUG = 1,
  INFO = 2,
  WARN = 3,
  ERROR = 4,
  FATAL = 5,
}

const LOG_LEVEL_NAMES: Record<LogLevel, string> = {
  [LogLevel.TRACE]: 'TRACE',
  [LogLevel.DEBUG]: 'DEBUG',
  [LogLevel.INFO]: 'INFO',
  [LogLevel.WARN]: 'WARN',
  [LogLevel.ERROR]: 'ERROR',
  [LogLevel.FATAL]: 'FATAL',
};

class Logger {
  private minLevel: LogLevel = LogLevel.INFO;
  private requestId: string | null = null;

  constructor() {
    // Уровень из переменной окружения или localStorage
    const envLevel = import.meta.env.VITE_LOG_LEVEL;
    const storedLevel = localStorage.getItem('pulsar_log_level');
    const levelStr = envLevel || storedLevel || 'info';
    
    this.minLevel = this.parseLevel(levelStr);
    
    // Генерируем request-id для сессии
    this.requestId = this.generateRequestId();
  }

  private parseLevel(level: string): LogLevel {
    const normalized = level.toLowerCase();
    switch (normalized) {
      case 'trace': return LogLevel.TRACE;
      case 'debug': return LogLevel.DEBUG;
      case 'info': return LogLevel.INFO;
      case 'warn': return LogLevel.WARN;
      case 'error': return LogLevel.ERROR;
      case 'fatal': return LogLevel.FATAL;
      default: return LogLevel.INFO;
    }
  }

  private generateRequestId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private shouldLog(level: LogLevel): boolean {
    return level >= this.minLevel;
  }

  private formatMessage(level: LogLevel, module: string, message: string, ...args: any[]): string {
    const timestamp = new Date().toISOString();
    const levelName = LOG_LEVEL_NAMES[level];
    const formattedMessage = args.length > 0 
      ? `${message} ${args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ')}`
      : message;
    
    return `[${timestamp}] [${levelName}] [${module}] ${formattedMessage}`;
  }

  private async sendToBackend(level: LogLevel, module: string, message: string, ...args: any[]): Promise<void> {
    if (!this.shouldLog(level)) return;

    try {
      const formatted = this.formatMessage(level, module, message, ...args);
      
      // Отправляем в Tauri backend для записи в файл
      if ((window as any).__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('log_message', {
          level: LOG_LEVEL_NAMES[level],
          module,
          message: formatted,
          requestId: this.requestId,
        }).catch(() => {
          // Fallback на console если backend недоступен
          console.log(formatted);
        });
      } else {
        // В браузере просто в console
        const consoleMethod = level >= LogLevel.ERROR ? console.error 
          : level >= LogLevel.WARN ? console.warn
          : level >= LogLevel.INFO ? console.info
          : console.debug;
        consoleMethod(formatted);
      }
    } catch (e) {
      console.error('Logger error:', e);
    }
  }

  trace(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.TRACE, module, message, ...args);
  }

  debug(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.DEBUG, module, message, ...args);
  }

  info(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.INFO, module, message, ...args);
  }

  warn(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.WARN, module, message, ...args);
  }

  error(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.ERROR, module, message, ...args);
  }

  fatal(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.FATAL, module, message, ...args);
  }

  setLevel(level: LogLevel): void {
    this.minLevel = level;
    localStorage.setItem('pulsar_log_level', LOG_LEVEL_NAMES[level].toLowerCase());
  }

  getRequestId(): string | null {
    return this.requestId;
  }
}

export const logger = new Logger();
