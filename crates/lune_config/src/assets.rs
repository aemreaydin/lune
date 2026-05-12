use crate::MergeInto;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetsConfig {
    pub search_paths: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct AssetsConfigLayer {
    pub search_paths: Option<Vec<String>>,
}

impl MergeInto<AssetsConfig> for AssetsConfigLayer {
    fn merge_into(&self, target: &mut AssetsConfig) {
        if let Some(search_paths) = &self.search_paths {
            target.search_paths = search_paths.clone();
        }
    }
}
