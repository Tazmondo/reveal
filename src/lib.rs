mod ast_parse;
mod sourcemap;

use ast_parse::parse_script;

use sourcemap::{parse_sourcemap, SourcemapNode};
use std::fs;

fn parse_source(file_root: &str, source_root: &SourcemapNode) {
    if source_root.class_name == "ModuleScript"
        || source_root.class_name == "Script"
        || source_root.class_name == "LocalScript"
    {
        parse_script(file_root, source_root);
    } else {
        if source_root.name == "_Index" {
            return;
        }

        source_root
            .children
            .iter()
            .for_each(|node| parse_source(file_root, node))
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
