// Written by Tazm0ndo
// Thanks to the team behind full-moon, and the team behind Rojo!

use std::{fs, path::PathBuf, thread, vec};

use full_moon::ast::{
    self, Block, Call, Expression, FunctionArgs, Index, LastStmt, Prefix, Stmt, Suffix, Value, Var,
};
use serde::Deserialize;
use serde_json::from_str;

// From https://github.com/JohnnyMorganz/wally-package-types/blob/ffb59821dbc3c2868525f8cf06f853d29301f983/src/command.rs#L20
// Cheers Mr Morganz <3
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SourcemapNode {
    name: String,
    class_name: String,
    #[serde(default)]
    file_paths: Vec<PathBuf>,
    #[serde(default)]
    children: Vec<SourcemapNode>,
}

const STACK_SIZE: usize = 4 * 1024 * 1024;
const ROOT: &str = "./example/";

fn handle_prefix(prefix: &Prefix) -> Option<Vec<String>> {
    match prefix {
        Prefix::Name(name) => Some(vec![name.to_string()]),
        Prefix::Expression(expression) => handle_expression(expression),
        _ => None,
    }
}

fn handle_var(var: &Var) -> Option<Vec<String>> {
    match var {
        Var::Name(token) => Some(vec![token.to_string()]),
        Var::Expression(expression) => {
            let Some(mut out_vec) = handle_prefix(expression.prefix()) else {
                return None;
            };

            out_vec.extend(expression.suffixes().filter_map(|suffix| match suffix {
                Suffix::Index(index) => match index {
                    Index::Dot { dot, name } => Some(name.to_string()),
                    _ => None,
                },
                _ => None,
            }));

            Some(out_vec)
        }
        _ => None,
    }
}

fn handle_value(value: &Value) -> Option<Vec<String>> {
    match value {
        Value::FunctionCall(call) => {
            handle_function_call(call);
            return None;
        }
        Value::Number(number) => Some(vec![number.to_string()]),
        Value::String(string) => Some(vec![string.to_string()]),
        Value::Symbol(symbol) => Some(vec![symbol.to_string()]),
        Value::Var(var) => handle_var(var),
        _ => None,
    }
}

fn handle_expression(expression: &Expression) -> Option<Vec<String>> {
    match expression {
        Expression::Parentheses {
            contained: _,
            expression,
        } => handle_expression(expression),
        Expression::Value {
            value,
            type_assertion: _,
        } => handle_value(value),
        _ => None,
    }
}

fn handle_local_assignment(assignment: &ast::LocalAssignment) {
    assignment.expressions().iter().for_each(|expression| {
        handle_expression(expression);
    })
}

fn handle_assignment(assignment: &ast::Assignment) {
    assignment.expressions().iter().for_each(|expression| {
        handle_expression(expression);
    })
}

fn handle_function_call(call: &ast::FunctionCall) {
    // println!("{}", call.prefix());
    let prefix = handle_prefix(call.prefix());

    let Some(prefix) = prefix else { return };

    if prefix.len() != 1 {
        return;
    }

    let prefix = &prefix[0];

    if prefix == "require" {
        let mut suffixes = call.suffixes();
        let suffix = suffixes.next().expect("Require did not have a suffix.");

        let index: Option<i32> = match suffix {
            Suffix::Call(call) => match call {
                Call::AnonymousCall(args) => match args {
                    FunctionArgs::Parentheses {
                        parentheses,
                        arguments,
                    } => {
                        let require_expression = arguments
                            .first()
                            .expect("Require found without an argument!");
                        let require_expression = require_expression.value();

                        let parsed_expression = handle_expression(require_expression);

                        println!("Required: {:?}", parsed_expression);

                        None
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };
    }
}

fn handle_if(if_statement: &ast::If) {
    process_block(if_statement.block());
}

fn process_stmt(stmt: &Stmt) {
    match stmt {
        Stmt::FunctionCall(call) => handle_function_call(call),
        Stmt::LocalAssignment(assignment) => handle_local_assignment(assignment),
        Stmt::Assignment(assignment) => handle_assignment(assignment),
        Stmt::Do(do_statement) => process_block(do_statement.block()),
        Stmt::If(if_statement) => handle_if(if_statement),
        _ => {}
    }
}

fn process_last_stmt(stmt: &LastStmt) {
    match stmt {
        LastStmt::Return(return_statement) => {
            return_statement.returns().iter().for_each(|expression| {
                handle_expression(expression);
            })
        }
        _ => {}
    }
}

fn process_block(block: &Block) {
    block.stmts().for_each(process_stmt);

    if let Some(last_statement) = block.last_stmt() {
        process_last_stmt(last_statement);
    };
}

fn parse_script(script_node: &SourcemapNode) {
    let path = script_node.file_paths.iter().find(|item| {
        if let Some(extension) = item.extension() {
            extension == "lua"
        } else {
            false
        }
    });

    if let Some(path) = path {
        let path = String::from(ROOT) + &path.to_string_lossy();

        println!("Parsing: {}", path);
        let contents = fs::read_to_string(&path);

        match contents {
            Err(err) => println!("Could not read file: {}. Error: {}", path, err),
            Ok(contents) => {
                let ast = full_moon::parse(&contents).unwrap();

                process_block(ast.nodes())
            }
        }
    }
}

fn parse_source(root: &SourcemapNode) {
    if root.class_name == "ModuleScript"
        || root.class_name == "Script"
        || root.class_name == "LocalScript"
    {
        parse_script(root);
    } else {
        if root.name == "_Index" {
            return;
        }

        root.children.iter().for_each(|node| parse_source(node))
    }
}

fn run() {
    let root = String::from(ROOT);

    let sourcemap =
        fs::read_to_string(root + "sourcemap.json").expect("Could not read sourcemap.json.");

    let sourcemap: SourcemapNode = from_str(&sourcemap).expect("Could not parse sourcemap.json.");

    parse_source(&sourcemap)
}

fn run_with_bigger_stack(func: fn() -> ()) {
    // Spawn thread with explicit stack size
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(func)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}

fn main() {
    // In debug mode, the full moon parser creates a stack overflow
    // So we run with a larger stack size
    if cfg!(debug_assertions) {
        run_with_bigger_stack(run)
        // let x = full_moon::parse("require(script.Parent.Test)").unwrap();
        // println!("{:?}", x);
    } else {
        run()
    }
}
