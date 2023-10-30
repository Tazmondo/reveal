use std::path::PathBuf;

use serde::Deserialize;
use serde_json::{from_str, Error};

// From https://github.com/JohnnyMorganz/wally-package-types/blob/ffb59821dbc3c2868525f8cf06f853d29301f983/src/command.rs#L20
// Cheers Mr Morganz <3
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SourcemapNode {
    pub name: String,
    pub class_name: String,
    #[serde(default)]
    pub file_paths: Vec<PathBuf>,
    #[serde(default)]
    pub children: Vec<SourcemapNode>,
}

impl SourcemapNode {
    pub fn lua_file(&self) -> Option<&PathBuf> {
        let Some(path) = self.file_paths.iter().find(|item| {
            if let Some(extension) = item.extension() {
                extension == "lua"
            } else {
                false
            }
        }) else {
            return None;
        };

        Some(path)
    }
}

pub fn parse_sourcemap(sourcemap: &str) -> Result<SourcemapNode, Error> {
    from_str(&sourcemap)
}
