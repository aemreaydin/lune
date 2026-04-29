use lune_diagnostics::{
    DiagnosticsConfig, DiagnosticsError, DiagnosticsLevel, LogBridge, LogFormat, init_diagnostics,
};

#[test]
fn developer_config_uses_human_local_defaults() {
    let config = DiagnosticsConfig::developer();

    assert_eq!(config.level, DiagnosticsLevel::Info);
    assert_eq!(config.format, LogFormat::Compact);
    assert_eq!(config.log_bridge, LogBridge::Enabled);
}

#[test]
fn config_builders_override_independent_fields() {
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
fn valid_setup_installs_global_subscriber_and_rejects_second_initialization() {
    let config = DiagnosticsConfig {
        level: DiagnosticsLevel::Info,
        format: LogFormat::Compact,
        log_bridge: LogBridge::Disabled,
    };

    init_diagnostics(config.clone()).unwrap();

    let err = init_diagnostics(config).unwrap_err();
    assert_eq!(err, DiagnosticsError::AlreadyInitialized);

    let manual_global_install =
        tracing::subscriber::set_global_default(tracing::subscriber::NoSubscriber::default());
    assert!(
        manual_global_install.is_err(),
        "init_diagnostics should install the process-global tracing subscriber",
    );
}

#[test]
fn enabled_log_bridge_installs_log_tracer() {
    let config = DiagnosticsConfig {
        level: DiagnosticsLevel::Info,
        format: LogFormat::Compact,
        log_bridge: LogBridge::Enabled,
    };

    init_diagnostics(config).unwrap();

    let manual_log_bridge_install = tracing_log::LogTracer::init();
    assert!(
        manual_log_bridge_install.is_err(),
        "LogBridge::Enabled should install tracing_log::LogTracer",
    );
}
