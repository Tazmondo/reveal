// This module is for converting the require lists into either a tree format, or a JSON graph format.
use std::io::prelude::*;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use serde::Serialize;

use crate::ast_parse::get_functions;
use crate::function_parse::get_require_argument;
use crate::sourcemap::{resolve_require, SourcemapKey, SourcemapNode};

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
    fn new() -> Graph {
        Graph {
            nodes: vec![],
            links: vec![],
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

pub fn generate_file(text: &String) {
    let mut file = File::create("./testdata").unwrap();

    let output_buffer = text.as_bytes();

    file.write(output_buffer).unwrap();
}
