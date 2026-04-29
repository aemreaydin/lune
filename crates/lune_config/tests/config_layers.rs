use lune_config::diagnostics::*;
use lune_config::*;

fn fixture_config() -> LuneConfig {
    LuneConfig {
        app: AppConfig {
            name: "Fixture".to_owned(),
        },
        diagnostics: DiagnosticsConfig {
            level: DiagnosticsLevel::Info,
            format: LogFormat::Compact,
            log_bridge: LogBridge::Enabled,
        },
        window: WindowConfig {
            title: "Fixture Window".to_owned(),
            width: 1280,
            height: 720,
            vsync: true,
        },
        renderer: RendererConfig {
            backend: "auto".to_owned(),
            clear_color: [0.02, 0.02, 0.025, 1.0],
        },
        assets: AssetsConfig {
            search_paths: vec!["assets".to_owned(), "assets/common".to_owned()],
        },
    }
}

#[test]
fn built_in_defaults_are_stable() {
    let config = LuneConfig::default();

    assert_eq!(config.app.name, "Lune");
    assert_eq!(config.diagnostics, DiagnosticsConfig::developer());
    assert_eq!(config.window.title, "Lune");
    assert_eq!(config.window.width, 1280);
    assert_eq!(config.window.height, 720);
    assert!(config.window.vsync);
    assert_eq!(config.renderer.backend, "auto");
    assert_eq!(config.renderer.clear_color, [0.02, 0.02, 0.025, 1.0]);
    assert_eq!(config.assets.search_paths, vec!["assets"]);
}

#[test]
fn config_layer_deserializes_from_toml() {
    let layer = parse_config_layer(
        "showcases/smoke/Showcase.toml",
        r#"
        [app]
        name = "Renderer Smoke"

        [diagnostics]
        level = "debug"
        format = "pretty"
        log_bridge = "disabled"

        [window]
        title = "Smoke"
        width = 1920
        height = 1080
        vsync = false

        [renderer]
        backend = "vulkan"
        clear_color = [0.1, 0.2, 0.3, 1.0]

        [assets]
        search_paths = ["assets", "showcases/smoke/assets"]
        "#,
    )
    .unwrap();

    assert_eq!(layer.app.unwrap().name.as_deref(), Some("Renderer Smoke"));
    let diagnostics = layer.diagnostics.unwrap();
    assert_eq!(diagnostics.level, Some(DiagnosticsLevel::Debug));
    assert_eq!(diagnostics.format, Some(LogFormat::Pretty));
    assert_eq!(diagnostics.log_bridge, Some(LogBridge::Disabled));

    let window = layer.window.unwrap();
    assert_eq!(window.title.as_deref(), Some("Smoke"));
    assert_eq!(window.width, Some(1920));
    assert_eq!(window.height, Some(1080));
    assert_eq!(window.vsync, Some(false));

    let renderer = layer.renderer.unwrap();
    assert_eq!(renderer.backend.as_deref(), Some("vulkan"));
    assert_eq!(renderer.clear_color, Some([0.1, 0.2, 0.3, 1.0]));

    assert_eq!(
        layer.assets.unwrap().search_paths.unwrap(),
        vec!["assets", "showcases/smoke/assets"],
    );
}

#[test]
fn unknown_root_fields_are_rejected() {
    let err = parse_config_layer(
        "Lune.toml",
        r#"
        typo = true
        "#,
    )
    .unwrap_err();

    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn unknown_nested_fields_are_rejected() {
    let err = parse_config_layer(
        "Lune.toml",
        r#"
        [window]
        widht = 1920
        "#,
    )
    .unwrap_err();

    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn parse_config_layer_reports_source_name_on_parse_error() {
    let err = parse_config_layer("broken/Showcase.toml", "[window").unwrap_err();

    match err {
        ConfigError::Parse {
            source_name,
            message,
        } => {
            assert_eq!(source_name, "broken/Showcase.toml");
            assert!(!message.is_empty());
        }
    }
}

#[test]
fn defaults_root_and_showcase_merge_in_order() {
    let root = parse_config_layer(
        "Lune.toml",
        r#"
        [app]
        name = "Root Name"

        [window]
        title = "Root Window"
        width = 1600
        "#,
    )
    .unwrap();
    let showcase = parse_config_layer(
        "showcases/smoke/Showcase.toml",
        r#"
        [app]
        name = "Showcase Name"

        [window]
        height = 900
        "#,
    )
    .unwrap();

    let merged = merge_config_layers(&[root, showcase]).unwrap();

    assert_eq!(merged.app.name, "Showcase Name");
    assert_eq!(merged.window.title, "Root Window");
    assert_eq!(merged.window.width, 1600);
    assert_eq!(merged.window.height, 900);
}

#[test]
fn nested_tables_merge_without_clearing_sibling_fields() {
    let root = parse_config_layer(
        "Lune.toml",
        r#"
        [window]
        width = 1600

        [renderer]
        backend = "vulkan"
        "#,
    )
    .unwrap();
    let showcase = parse_config_layer(
        "showcases/smoke/Showcase.toml",
        r#"
        [window]
        height = 900

        [renderer]
        clear_color = [0.3, 0.2, 0.1, 1.0]
        "#,
    )
    .unwrap();

    let mut merged = fixture_config();
    merge_config_layers_onto(&mut merged, &[root, showcase]).unwrap();

    assert_eq!(merged.window.title, "Fixture Window");
    assert_eq!(merged.window.width, 1600);
    assert_eq!(merged.window.height, 900);
    assert!(merged.window.vsync);
    assert_eq!(merged.renderer.backend, "vulkan");
    assert_eq!(merged.renderer.clear_color, [0.3, 0.2, 0.1, 1.0]);
}

#[test]
fn diagnostics_table_merges_without_clearing_sibling_fields() {
    let root = parse_config_layer(
        "Lune.toml",
        r#"
        [diagnostics]
        level = "debug"
        "#,
    )
    .unwrap();
    let showcase = parse_config_layer(
        "showcases/smoke/Showcase.toml",
        r#"
        [diagnostics]
        format = "pretty"
        log_bridge = "disabled"
        "#,
    )
    .unwrap();

    let mut merged = fixture_config();
    merge_config_layers_onto(&mut merged, &[root, showcase]).unwrap();

    assert_eq!(merged.diagnostics.level, DiagnosticsLevel::Debug);
    assert_eq!(merged.diagnostics.format, LogFormat::Pretty);
    assert_eq!(merged.diagnostics.log_bridge, LogBridge::Disabled);
}

#[test]
fn arrays_replace_instead_of_append() {
    let root = parse_config_layer(
        "Lune.toml",
        r#"
        [assets]
        search_paths = ["assets/root", "assets/shared"]
        "#,
    )
    .unwrap();
    let showcase = parse_config_layer(
        "showcases/smoke/Showcase.toml",
        r#"
        [assets]
        search_paths = ["showcases/smoke/assets"]
        "#,
    )
    .unwrap();

    let mut merged = fixture_config();
    merge_config_layers_onto(&mut merged, &[root, showcase]).unwrap();

    assert_eq!(merged.assets.search_paths, vec!["showcases/smoke/assets"],);
}

#[test]
fn empty_layers_keep_existing_values() {
    let mut merged = fixture_config();
    merge_config_layers_onto(&mut merged, &[ConfigLayer::default()]).unwrap();

    assert_eq!(merged, fixture_config());
}
