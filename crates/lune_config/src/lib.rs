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
pub use renderer::{RendererBackend, RendererConfig, RendererConfigLayer};
pub use window::{WindowConfig, WindowConfigLayer};

pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

pub trait MergeInto<T> {
    fn merge_into(&self, target: &mut T);
}

impl<L, T> MergeInto<T> for Option<L>
where
    L: MergeInto<T>,
{
    fn merge_into(&self, target: &mut T) {
        if let Some(layer) = self {
            layer.merge_into(target);
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to parse `{source_name}`")]
    Parse {
        source_name: String,
        #[source]
        cause: toml::de::Error,
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
                backend: RendererBackend::Auto,
                clear_color: [0.02, 0.02, 0.025, 1.0],
            },
            assets: AssetsConfig {
                search_paths: vec!["assets".to_owned()],
            },
        }
    }
}

pub fn merge_config_layers(layers: &[ConfigLayer]) -> LuneConfig {
    let mut config = LuneConfig::default();
    apply_config_layers(&mut config, layers);
    config
}

pub fn merge_config_layers_onto(base: &mut LuneConfig, layers: &[ConfigLayer]) {
    apply_config_layers(base, layers);
}

fn apply_config_layers(config: &mut LuneConfig, layers: &[ConfigLayer]) {
    for layer in layers {
        layer.app.merge_into(&mut config.app);
        layer.diagnostics.merge_into(&mut config.diagnostics);
        layer.window.merge_into(&mut config.window);
        layer.renderer.merge_into(&mut config.renderer);
        layer.assets.merge_into(&mut config.assets);
    }
}
