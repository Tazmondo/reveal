mod sourcemap;

use sourcemap::{parse_sourcemap, SourcemapNode};
use std::{fs, vec};

use full_moon::ast::{
    self, Block, Call, Expression, FunctionArgs, Index, LastStmt, Prefix, Stmt, Suffix, Value, Var,
};

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
                    Index::Dot { dot: _, name } => Some(name.to_string()),
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

        match suffix {
            Suffix::Call(call) => match call {
                Call::AnonymousCall(args) => match args {
                    FunctionArgs::Parentheses {
                        parentheses: _,
                        arguments,
                    } => {
                        let require_expression = arguments
                            .first()
                            .expect("Require found without an argument!");
                        let require_expression = require_expression.value();

                        let parsed_expression = handle_expression(require_expression);

                        println!("Required: {:?}", parsed_expression);
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
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

pub fn run(root: &str) {
    let root = String::from(root);
    let sourcemap_contents =
        fs::read_to_string(root + "/sourcemap.json").expect("Could not read sourcemap.json.");

    let source_root: SourcemapNode =
        parse_sourcemap(&sourcemap_contents).expect("Could not parse sourcemap.");

    println!("Beginning parse!");
    let start = std::time::Instant::now();
    parse_source(&source_root);
    let elapsed = start.elapsed();
    println!("Parse finished in {}ms", elapsed.as_millis());
}
