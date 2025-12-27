//! Ftail 自定义 logger
//!
//! 提供简约格式的彩色控制台输出支持
pub struct ColorfulConsoleLogger {
    pub config: ftail::Config,
}

impl log::Log for ColorfulConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.config.level_filter
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let level_str = match level {
            log::Level::Error => "\x1b[31m[ERROR]\x1b[0m", // Red
            // 对齐5字符
            log::Level::Warn => " \x1b[33m[WARN]\x1b[0m", // Yellow
            log::Level::Info => " \x1b[32m[INFO]\x1b[0m", // Green
            log::Level::Debug => "\x1b[34m[DEBUG]\x1b[0m", // Blue
            log::Level::Trace => "\x1b[37m[TRACE]\x1b[0m", // White
        };
        eprintln!("{} {}", level_str, record.args());
    }

    fn flush(&self) {}
}
