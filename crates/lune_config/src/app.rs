use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfigLayer {
    pub name: Option<String>,
}
