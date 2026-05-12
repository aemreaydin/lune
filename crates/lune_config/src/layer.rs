use crate::app::AppConfigLayer;
use crate::assets::AssetsConfigLayer;
use crate::diagnostics::DiagnosticsConfigLayer;
use crate::renderer::RendererConfigLayer;
use crate::window::WindowConfigLayer;
use crate::{ConfigError, ConfigResult};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigLayer {
    pub app: Option<AppConfigLayer>,
    pub diagnostics: Option<DiagnosticsConfigLayer>,
    pub window: Option<WindowConfigLayer>,
    pub renderer: Option<RendererConfigLayer>,
    pub assets: Option<AssetsConfigLayer>,
}

pub fn parse_config_layer(source: impl Into<String>, toml_text: &str) -> ConfigResult<ConfigLayer> {
    let source_name = source.into();
    toml::from_str(toml_text).map_err(|cause| ConfigError::Parse { source_name, cause })
}
