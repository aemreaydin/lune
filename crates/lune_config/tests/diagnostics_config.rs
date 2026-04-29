use lune_config::diagnostics::{DiagnosticsConfig, DiagnosticsLevel, LogBridge, LogFormat};

#[test]
fn developer_diagnostics_config_uses_human_local_defaults() {
    let config = DiagnosticsConfig::developer();

    assert_eq!(config.level, DiagnosticsLevel::Info);
    assert_eq!(config.format, LogFormat::Compact);
    assert_eq!(config.log_bridge, LogBridge::Enabled);
}

#[test]
fn diagnostics_config_builders_override_independent_fields() {
    let config = DiagnosticsConfig {
        level: DiagnosticsLevel::Warn,
        format: LogFormat::Compact,
        log_bridge: LogBridge::Enabled,
    }
    .with_level(DiagnosticsLevel::Trace)
    .with_format(LogFormat::Pretty)
    .with_log_bridge(LogBridge::Disabled);

    assert_eq!(config.level, DiagnosticsLevel::Trace);
    assert_eq!(config.format, LogFormat::Pretty);
    assert_eq!(config.log_bridge, LogBridge::Disabled);
}

#[test]
fn diagnostics_level_defaults_to_info() {
    assert_eq!(DiagnosticsLevel::default(), DiagnosticsLevel::Info);
}

#[test]
fn diagnostics_config_deserializes_lowercase_strings() {
    let config: DiagnosticsConfig = toml::from_str(
        r#"
        level = "debug"
        format = "pretty"
        log_bridge = "disabled"
        "#,
    )
    .unwrap();

    assert_eq!(config.level, DiagnosticsLevel::Debug);
    assert_eq!(config.format, LogFormat::Pretty);
    assert_eq!(config.log_bridge, LogBridge::Disabled);
}

#[test]
fn diagnostics_config_serializes_enums_as_lowercase_strings() {
    let config = DiagnosticsConfig {
        level: DiagnosticsLevel::Trace,
        format: LogFormat::Compact,
        log_bridge: LogBridge::Enabled,
    };

    let serialized = toml::to_string(&config).unwrap();

    assert!(serialized.contains("level = \"trace\""));
    assert!(serialized.contains("format = \"compact\""));
    assert!(serialized.contains("log_bridge = \"enabled\""));
}
