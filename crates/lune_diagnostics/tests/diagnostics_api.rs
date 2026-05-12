use lune_config::diagnostics::{DiagnosticsConfig, DiagnosticsLevel, LogBridge, LogFormat};
use lune_diagnostics::{DiagnosticsError, init_diagnostics};

#[test]
fn init_installs_subscriber_and_bridge_and_rejects_reinitialization() {
    let config = DiagnosticsConfig {
        level: DiagnosticsLevel::Info,
        format: LogFormat::Compact,
        log_bridge: LogBridge::Enabled,
    };

    init_diagnostics(config.clone()).unwrap();

    let err = init_diagnostics(config).unwrap_err();
    assert!(matches!(err, DiagnosticsError::AlreadyInitialized(_)));

    let manual_global_install =
        tracing::subscriber::set_global_default(tracing::subscriber::NoSubscriber::default());
    assert!(
        manual_global_install.is_err(),
        "init_diagnostics should install the process-global tracing subscriber",
    );

    let manual_log_bridge_install = tracing_log::LogTracer::init();
    assert!(
        manual_log_bridge_install.is_err(),
        "LogBridge::Enabled should install tracing_log::LogTracer",
    );
}
