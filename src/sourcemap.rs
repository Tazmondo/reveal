use std::path::PathBuf;

use serde::Deserialize;
use serde_json::{from_str, Error};

// From https://github.com/JohnnyMorganz/wally-package-types/blob/ffb59821dbc3c2868525f8cf06f853d29301f983/src/command.rs#L20
// Cheers Mr Morganz <3
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourcemapNode {
    pub name: String,
    pub class_name: String,
    #[serde(default)]
    pub file_paths: Vec<PathBuf>,
    #[serde(default)]
    pub children: Vec<SourcemapNode>,
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SourcemapKey {
    pub name: String,
    pub class_name: String,
    pub file_path: PathBuf,
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

    pub fn as_key(&self) -> Option<SourcemapKey> {
        let file = self.lua_file();
        match file {
            Some(file) => Some(SourcemapKey {
                name: self.name.clone(),
                class_name: self.class_name.clone(),
                file_path: file.to_path_buf(),
            }),
            None => None,
        }
    }
}

pub fn parse_sourcemap(sourcemap: &str) -> Result<SourcemapNode, Error> {
    from_str(&sourcemap)
}

pub fn node_search_name<'a>(
    root: &'a SourcemapNode,
    search_name: &str,
) -> Option<&'a SourcemapNode> {
    root.children.iter().find(|child| child.name == search_name)
}

// more awful code
// TODO: Make this prettier and easier to follow
pub fn resolve_require<'a>(
    node_path: &Vec<&'a SourcemapNode>,
    require: &Vec<String>,
) -> Option<&'a SourcemapNode> {
    let Some(require_head) = require.first() else {
        return None;
    };

    let mut require_tail = &require[..];
    let mut node_path_index = node_path.len() - 1;

    let mut current_node = match require_head.as_str() {
        "script" => {
            require_tail = &require_tail[1..];

            node_path_index -= 1;
            node_path[node_path_index + 1]
        }
        _ => node_path.first().unwrap(),
    };

    for node_string in require_tail {
        match node_string.as_str() {
            "Parent" => {
                current_node = node_path[node_path_index];

                if node_path_index == 0 {
                    break;
                }
                node_path_index -= 1;
            }
            child_string => {
                current_node = match node_search_name(current_node, child_string) {
                    Some(node) => node,
                    None => {
                        return None;
                    }
                }
            }
        }
    }

    Some(current_node)
}
