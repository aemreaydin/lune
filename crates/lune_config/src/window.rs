use crate::MergeInto;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct WindowConfigLayer {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub vsync: Option<bool>,
}

impl MergeInto<WindowConfig> for WindowConfigLayer {
    fn merge_into(&self, target: &mut WindowConfig) {
        if let Some(title) = &self.title {
            target.title = title.clone();
        }
        if let Some(width) = self.width {
            target.width = width;
        }
        if let Some(height) = self.height {
            target.height = height;
        }
        if let Some(vsync) = self.vsync {
            target.vsync = vsync;
        }
    }
}
