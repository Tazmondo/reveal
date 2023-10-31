mod ast_parse;
mod function_parse;
mod sourcemap;

use ast_parse::get_functions;

use function_parse::get_require_argument;
use sourcemap::{parse_sourcemap, resolve_require, SourcemapNode};
use std::{collections::HashMap, fs, path::PathBuf};

type RequireMap<'a> = HashMap<String, Vec<&'a SourcemapNode>>;

fn parse_source<'a>(
    project_root: &str,
    source_node: &'a SourcemapNode,
    node_path: &mut Vec<&'a SourcemapNode>, // Allows for resolving of "script" and "Parent" in requires
) -> RequireMap<'a> {
    node_path.push(source_node);

    let mut hashmap: RequireMap = HashMap::new();

    let mut closure = || {
        if source_node.class_name == "ModuleScript"
            || source_node.class_name == "Script"
            || source_node.class_name == "LocalScript"
        {
            let Some(lua_file) = source_node.lua_file() else {
                return;
            };

            let lua_file = PathBuf::new().join(project_root).join(lua_file);

            let key = lua_file.to_string_lossy().to_string();

            let mut resolved_vector: Vec<&SourcemapNode> = Vec::new();

            let funcs = get_functions(&lua_file).unwrap();

            funcs.iter().for_each(|item| {
                let req_args = get_require_argument(item);

                match req_args {
                    Some(args) => {
                        let resolved = resolve_require(node_path, &args);

                        if let Some(resolved) = resolved {
                            println!("{} <- {}", source_node.name, resolved.name);
                            resolved_vector.push(resolved);
                        } else {
                            println!(
                                "Failed to resolve file: {}. Require: {:?}",
                                source_node.name, args
                            );
                        }
                    }
                    _ => {}
                }
            });

            hashmap.insert(String::from(&key), resolved_vector);
        } else {
            if source_node.name == "_Index" {
                return;
            }

            source_node.children.iter().for_each(|node| {
                let child_map = parse_source(project_root, node, node_path);

                child_map.iter().for_each(|(key, value)| {
                    let current_vec = hashmap.entry(key.to_string()).or_default();

                    current_vec.extend(value);
                })
            });
        }
    };

    closure();

    node_path.pop();

    hashmap
}

pub fn run(root: &str) {
    let root_string = String::from(root);

    let sourcemap_contents = fs::read_to_string(root_string + "/sourcemap.json")
        .expect("Could not read sourcemap.json.");

    let source_root: SourcemapNode =
        parse_sourcemap(&sourcemap_contents).expect("Could not parse sourcemap.");

    println!("Beginning parse!");
    let start = std::time::Instant::now();

    // Keep a live array of the current path, pushing and popping when the parse-depth changes
    // There may be a better, more FP-style way of doing this but i cba and this works.
    let mut parent_nodes: Vec<&SourcemapNode> = vec![];

    let resolve_map = parse_source(root, &source_root, &mut parent_nodes);

    let elapsed = start.elapsed();
    println!("Parse finished in {}ms", elapsed.as_millis());

    resolve_map.iter().for_each(|(key, value)| {
        println!(
            "Script: {} <- {:?}",
            key,
            value
                .iter()
                .map(|node| &node.name)
                .collect::<Vec<&String>>()
        );
    })
}
