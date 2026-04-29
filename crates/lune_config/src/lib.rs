//! Configuration loading and merge semantics for Lune.
//!
//! Production merge logic lands in the config learning milestone.
//!

#![forbid(unsafe_code)]

pub mod app;
pub mod assets;
pub mod diagnostics;
pub mod layer;
pub mod renderer;
pub mod window;

use thiserror::Error;

pub use app::{AppConfig, AppConfigLayer};
pub use assets::{AssetsConfig, AssetsConfigLayer};
pub use diagnostics::{
    DiagnosticsConfig, DiagnosticsConfigLayer, DiagnosticsLevel, LogBridge, LogFormat,
};
pub use layer::{ConfigLayer, parse_config_layer};
pub use renderer::{RendererConfig, RendererConfigLayer};
pub use window::{WindowConfig, WindowConfigLayer};

pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("failed to parse `{source_name}`: {message}")]
    Parse {
        source_name: String,
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct LuneConfig {
    pub app: AppConfig,
    pub diagnostics: DiagnosticsConfig,
    pub window: WindowConfig,
    pub renderer: RendererConfig,
    pub assets: AssetsConfig,
}

impl Default for LuneConfig {
    fn default() -> Self {
        Self {
            app: AppConfig {
                name: "Lune".to_owned(),
            },
            diagnostics: DiagnosticsConfig::developer(),
            window: WindowConfig {
                title: "Lune".to_owned(),
                width: 1280,
                height: 720,
                vsync: true,
            },
            renderer: RendererConfig {
                backend: "auto".to_owned(),
                clear_color: [0.02, 0.02, 0.025, 1.0],
            },
            assets: AssetsConfig {
                search_paths: vec!["assets".to_owned()],
            },
        }
    }
}

pub fn merge_config_layers(layers: &[ConfigLayer]) -> ConfigResult<LuneConfig> {
    let mut config = LuneConfig::default();
    apply_config_layers(&mut config, layers)?;
    Ok(config)
}

pub fn merge_config_layers_onto(base: &mut LuneConfig, layers: &[ConfigLayer]) -> ConfigResult<()> {
    apply_config_layers(base, layers)
}

fn apply_config_layers(config: &mut LuneConfig, layers: &[ConfigLayer]) -> ConfigResult<()> {
    for layer in layers {
        if let Some(app) = &layer.app
            && let Some(name) = &app.name
        {
            config.app.name = name.clone();
        }

        if let Some(diagnostics) = &layer.diagnostics {
            if let Some(level) = diagnostics.level {
                config.diagnostics.level = level;
            }

            if let Some(format) = diagnostics.format {
                config.diagnostics.format = format;
            }

            if let Some(log_bridge) = diagnostics.log_bridge {
                config.diagnostics.log_bridge = log_bridge;
            }
        }

        if let Some(window) = &layer.window {
            if let Some(title) = &window.title {
                config.window.title = title.clone();
            }

            if let Some(width) = window.width {
                config.window.width = width;
            }

            if let Some(height) = window.height {
                config.window.height = height;
            }

            if let Some(vsync) = window.vsync {
                config.window.vsync = vsync;
            }
        }

        if let Some(renderer) = &layer.renderer {
            if let Some(backend) = &renderer.backend {
                config.renderer.backend = backend.clone();
            }

            if let Some(clear_color) = renderer.clear_color {
                config.renderer.clear_color = clear_color;
            }
        }

        if let Some(assets) = &layer.assets
            && let Some(search_paths) = &assets.search_paths
        {
            config.assets.search_paths = search_paths.clone();
        }
    }

    Ok(())
}
