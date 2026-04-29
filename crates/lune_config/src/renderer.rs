use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub struct RendererConfig {
    pub backend: String,
    pub clear_color: [f32; 4],
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct RendererConfigLayer {
    pub backend: Option<String>,
    pub clear_color: Option<[f32; 4]>,
}
