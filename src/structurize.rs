// This module is for converting the require lists into either a tree format, or a JSON graph format.
use crate::sourcemap::{SourcemapKey, SourcemapNode};
use serde::Serialize;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

#[derive(Debug, Serialize)]
struct Node {
    id: i32,
    name: String,
}

#[derive(Debug, Serialize)]
struct Link {
    source: i32,
    target: i32,
}

#[derive(Debug, Serialize)]
struct Graph {
    nodes: Vec<Node>,
    links: Vec<Link>,
}

impl Graph {
    fn from_map(map: &RequireMap) -> Graph {
        // We iterate twice in this function, since we need to know the assigned id for each node
        // in order to generate the links.
        // So on the first pass we generate all the nodes and link the ids to the unique file path of each one.
        // Then on the second pass we generate all links, using the map to find the correct id.

        let mut node_id_map: HashMap<&PathBuf, i32> = HashMap::new();

        let mut node_count = 0;

        let mut nodes: Vec<Node> = Vec::new();
        let mut links: Vec<Link> = Vec::new();

        map.keys().for_each(|key| {
            node_id_map.insert(&key.file_path, node_count);
            nodes.push(Node {
                id: node_count,
                name: key.name.clone(),
            });

            node_count += 1;
        });

        map.iter().for_each(|(key, nodes)| {
            nodes
                .iter()
                .filter_map(|node| node.lua_file())
                .for_each(|path| {
                    let source_id = node_id_map.get(path).unwrap();
                    let target_id = node_id_map.get(&key.file_path).unwrap();

                    links.push(Link {
                        source: *source_id,
                        target: *target_id,
                    });
                });
        });

        Graph {
            nodes: nodes,
            links: links,
        }
    }
}

pub type RequireMap<'a> = HashMap<SourcemapKey, Vec<&'a SourcemapNode>>;

pub fn create_require_tree(
    resolve_map: &RequireMap,
    root: &SourcemapKey,
    max_depth: Option<u32>,
    depth: Option<u32>,
) -> termtree::Tree<String> {
    let depth = depth.unwrap_or(0) + 1;

    if max_depth.is_some() && depth >= max_depth.unwrap() {
        return termtree::Tree::new(root.name.clone());
    }

    let requires = resolve_map.get(root).unwrap();

    termtree::Tree::new(root.name.clone()).with_leaves(requires.iter().map(|require| {
        create_require_tree(
            resolve_map,
            &require.as_key().unwrap(),
            max_depth,
            Some(depth),
        )
    }))
}

pub fn output_map(map: &RequireMap, path: &PathBuf) {
    let mut file = File::create(path).unwrap();

    let output =
        serde_json::to_string(&Graph::from_map(map)).expect("Could not convert graph data to JSON");

    file.write(output.as_bytes()).unwrap();
}
