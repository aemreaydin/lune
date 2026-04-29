//! Diagnostics, errors, and tracing setup for Lune.
//!
//! Production setup logic lands in the diagnostics learning milestone.
//!
//! Library crates should expose typed errors instead of erasing everything into
//! strings. A typical Lune library error should look like this:
//!
//! ```
//! use thiserror::Error;
//!
//! #[derive(Debug, Error, PartialEq, Eq)]
//! enum ConfigError {
//!     #[error("missing required config file `{path}`")]
//!     MissingFile { path: String },
//! }
//!
//! fn load_config(path: &str) -> Result<(), ConfigError> {
//!     Err(ConfigError::MissingFile {
//!         path: path.to_owned(),
//!     })
//! }
//!
//! let err = load_config("Lune.toml").unwrap_err();
//! assert_eq!(
//!     err,
//!     ConfigError::MissingFile {
//!         path: "Lune.toml".to_owned()
//!     }
//! );
//! ```

#![forbid(unsafe_code)]

use thiserror::Error;

pub type LuneResult<T> = std::result::Result<T, DiagnosticsError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiagnosticsError {
    #[error("diagnostics have already been initialized")]
    AlreadyInitialized,

    #[error("failed to install log bridge")]
    LogBridgeInstall,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiagnosticsLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsConfig {
    pub level: DiagnosticsLevel,
    pub format: LogFormat,
    pub log_bridge: LogBridge,
}

impl DiagnosticsConfig {
    pub fn developer() -> Self {
        Self {
            level: DiagnosticsLevel::Info,
            format: LogFormat::Compact,
            log_bridge: LogBridge::Enabled,
        }
    }

    pub fn with_level(mut self, level: DiagnosticsLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_log_bridge(mut self, log_bridge: LogBridge) -> Self {
        self.log_bridge = log_bridge;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogFormat {
    #[default]
    Compact,
    Pretty,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogBridge {
    Disabled,

    #[default]
    Enabled,
}

pub fn init_diagnostics(config: DiagnosticsConfig) -> LuneResult<()> {
    if config.log_bridge == LogBridge::Enabled {
        tracing_log::LogTracer::init().map_err(|_| DiagnosticsError::LogBridgeInstall)?
    }
    let level = match config.level {
        DiagnosticsLevel::Trace => tracing::Level::TRACE,
        DiagnosticsLevel::Debug => tracing::Level::DEBUG,
        DiagnosticsLevel::Info => tracing::Level::INFO,
        DiagnosticsLevel::Warn => tracing::Level::WARN,
        DiagnosticsLevel::Error => tracing::Level::ERROR,
    };
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_level(true);

    match config.format {
        LogFormat::Compact => {
            tracing::subscriber::set_global_default(subscriber.compact().finish())
        }
        LogFormat::Pretty => tracing::subscriber::set_global_default(subscriber.pretty().finish()),
    }
    .map_err(|_| DiagnosticsError::AlreadyInitialized)
}
