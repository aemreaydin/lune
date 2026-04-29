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
