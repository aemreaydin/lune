use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiagnosticsConfig {
    pub level: DiagnosticsLevel,
    pub format: LogFormat,
    pub log_bridge: LogBridge,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct DiagnosticsConfigLayer {
    pub level: Option<DiagnosticsLevel>,
    pub format: Option<LogFormat>,
    pub log_bridge: Option<LogBridge>,
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

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self::developer()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticsLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Compact,
    Pretty,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogBridge {
    Disabled,
    #[default]
    Enabled,
}
