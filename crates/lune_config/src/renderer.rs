use crate::MergeInto;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub struct RendererConfig {
    pub backend: RendererBackend,
    pub clear_color: [f32; 4],
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct RendererConfigLayer {
    pub backend: Option<RendererBackend>,
    pub clear_color: Option<[f32; 4]>,
}

impl MergeInto<RendererConfig> for RendererConfigLayer {
    fn merge_into(&self, target: &mut RendererConfig) {
        if let Some(backend) = self.backend {
            target.backend = backend;
        }
        if let Some(clear_color) = self.clear_color {
            target.clear_color = clear_color;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RendererBackend {
    #[default]
    Auto,
    Vulkan,
    Metal,
    Dx12,
    Gl,
}
