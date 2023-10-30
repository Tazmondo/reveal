mod ast_parse;
mod function_parse;
mod sourcemap;

use ast_parse::get_functions;

use function_parse::print_function;
use sourcemap::{parse_sourcemap, SourcemapNode};
use std::{fs, path::PathBuf};

fn parse_source(project_root: &str, source_root: &SourcemapNode) {
    if source_root.class_name == "ModuleScript"
        || source_root.class_name == "Script"
        || source_root.class_name == "LocalScript"
    {
        let Some(lua_file) = source_root.lua_file() else {
            return;
        };

        let lua_file = PathBuf::new().join(project_root).join(lua_file);

        let funcs = get_functions(&lua_file).unwrap();

        funcs
            .iter()
            .for_each(|item| print_function(&source_root.name, item));
    } else {
        if source_root.name == "_Index" {
            return;
        }

        source_root
            .children
            .iter()
            .for_each(|node| parse_source(project_root, node))
    }
}

pub fn run(root: &str) {
    let root_string = String::from(root);

    let sourcemap_contents = fs::read_to_string(root_string + "/sourcemap.json")
        .expect("Could not read sourcemap.json.");

    let source_root: SourcemapNode =
        parse_sourcemap(&sourcemap_contents).expect("Could not parse sourcemap.");

    println!("Beginning parse!");
    let start = std::time::Instant::now();

    parse_source(root, &source_root);

    let elapsed = start.elapsed();
    println!("Parse finished in {}ms", elapsed.as_millis());
}
