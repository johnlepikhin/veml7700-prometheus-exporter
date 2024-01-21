use std::net::SocketAddr;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use structdoc::StructDoc;

#[derive(Clone, Copy, Serialize, Deserialize, StructDoc)]
pub enum LogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for slog::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Critical => slog::Level::Critical,
            LogLevel::Error => slog::Level::Error,
            LogLevel::Warning => slog::Level::Warning,
            LogLevel::Info => slog::Level::Info,
            LogLevel::Debug => slog::Level::Debug,
            LogLevel::Trace => slog::Level::Trace,
        }
    }
}

#[derive(Serialize, Deserialize, StructDoc)]
pub struct Config {
    /// Max log level for syslog mode
    pub log_level: LogLevel,
    /// I2C device path, like /dev/i2c-1
    pub i2c_device: std::path::PathBuf,
    /// Prometheus exporter listen address
    pub exporter_listen: SocketAddr,
}

impl Config {
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    pub fn read(file: &str) -> Result<Self> {
        let config = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to load config file {:?}", file))?;
        let config: Self = serde_yaml::from_str(&config)
            .with_context(|| format!("Failed to parse config file {:?}", file))?;

        config.validate()?;
        Ok(config)
    }
}
