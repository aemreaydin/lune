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

use lune_config::diagnostics::{DiagnosticsConfig, DiagnosticsLevel, LogBridge, LogFormat};
use thiserror::Error;

pub type DiagnosticsResult<T> = std::result::Result<T, DiagnosticsError>;

#[derive(Debug, Error)]
pub enum DiagnosticsError {
    #[error("diagnostics have already been initialized")]
    AlreadyInitialized(#[from] tracing::subscriber::SetGlobalDefaultError),

    #[error("failed to install log bridge")]
    LogBridgeInstall(#[from] log::SetLoggerError),
}

pub fn init_diagnostics(config: DiagnosticsConfig) -> DiagnosticsResult<()> {
    let level = match config.level {
        DiagnosticsLevel::Trace => tracing::Level::TRACE,
        DiagnosticsLevel::Debug => tracing::Level::DEBUG,
        DiagnosticsLevel::Info => tracing::Level::INFO,
        DiagnosticsLevel::Warn => tracing::Level::WARN,
        DiagnosticsLevel::Error => tracing::Level::ERROR,
    };
    let subscriber = tracing_subscriber::fmt().with_max_level(level);

    match config.format {
        LogFormat::Compact => {
            tracing::subscriber::set_global_default(subscriber.compact().finish())?
        }
        LogFormat::Pretty => tracing::subscriber::set_global_default(subscriber.pretty().finish())?,
    }

    if config.log_bridge == LogBridge::Enabled {
        tracing_log::LogTracer::init()?;
    }

    Ok(())
}
