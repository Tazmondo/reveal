use std::{fs, vec};

use full_moon::ast::{
    self, Assignment, Block, Expression, FunctionCall, If, LastStmt, Stmt, Value,
};

use crate::sourcemap::SourcemapNode;

type Functions<'a> = Vec<&'a FunctionCall>;

// fn handle_expression(expression: &Expression) -> RequireArgs {
//     match expression {
//         Expression::Parentheses {
//             contained: _,
//             expression,
//         } => handle_expression(expression),
//         Expression::Value {
//             value,
//             type_assertion: _,
//         } => handle_value(value),
//         _ => None,
//     }
// }

// fn handle_prefix(prefix: &Prefix) -> RequireArgs {
//     match prefix {
//         Prefix::Name(name) => vec![name.to_string()],
//         Prefix::Expression(expression) => handle_expression(expression),
//         _ => vec![],
//     }
// }

// fn handle_function_call(call: &FunctionCall) -> RequireArgs {
//     // println!("{}", call.prefix());
//     let prefix = handle_prefix(call.prefix());

//     let Some(prefix) = prefix else { return None };

//     if prefix.len() != 1 {
//         return None;
//     }

//     let prefix = &prefix[0];

//     return if prefix == "require" {
//         let mut suffixes = call.suffixes();
//         let suffix = suffixes.next().expect("Require did not have a suffix.");

//         match suffix {
//             Suffix::Call(call) => match call {
//                 Call::AnonymousCall(args) => match args {
//                     FunctionArgs::Parentheses {
//                         parentheses: _,
//                         arguments,
//                     } => {
//                         let require_expression = arguments
//                             .first()
//                             .expect("Require found without an argument!");
//                         let require_expression = require_expression.value();

//                         let parsed_expression = handle_expression(require_expression);

//                         println!("Required: {:?}", parsed_expression);

//                         parsed_expression
//                     }
//                     _ => None,
//                 },
//                 _ => None,
//             },
//             _ => None,
//         }
//     } else {
//         None
//     };
// }

fn local_assignment_get_functions(assignment: &ast::LocalAssignment) -> Functions {
    assignment
        .expressions()
        .iter()
        .map(expression_get_functions)
        .flatten()
        .collect()
}

fn assignment_get_functions(assignment: &Assignment) -> Functions {
    assignment
        .expressions()
        .iter()
        .map(expression_get_functions)
        .flatten()
        .collect()
}

fn if_get_functions(if_statement: &If) -> Functions {
    block_get_functions(if_statement.block())
}

fn expression_get_functions(expression: &Expression) -> Functions {
    match expression {
        Expression::Parentheses {
            contained: _,
            expression,
        } => expression_get_functions(expression),
        Expression::Value {
            value,
            type_assertion: _,
        } => match &**value {
            Value::FunctionCall(call) => vec![call],
            Value::IfExpression(if_expression) => {
                let first = expression_get_functions(if_expression.if_expression());
                let last = expression_get_functions(if_expression.else_expression());

                first.into_iter().chain(last).collect()
            }
            Value::ParenthesesExpression(parentheses_expression) => {
                expression_get_functions(parentheses_expression)
            }
            _ => vec![],
        },
        _ => vec![],
    }
}

fn stmt_get_functions(stmt: &Stmt) -> Functions {
    match stmt {
        Stmt::FunctionCall(call) => vec![call],
        Stmt::LocalAssignment(assignment) => local_assignment_get_functions(assignment),
        Stmt::Assignment(assignment) => assignment_get_functions(assignment),
        Stmt::Do(do_statement) => block_get_functions(do_statement.block()),
        Stmt::If(if_statement) => if_get_functions(if_statement),
        _ => vec![],
    }
}

fn last_stmt_get_functions(stmt: &LastStmt) -> Functions {
    match stmt {
        LastStmt::Return(return_statement) => return_statement
            .returns()
            .iter()
            .map(expression_get_functions)
            .flatten()
            .collect(),
        _ => vec![],
    }
}

fn block_get_functions(block: &Block) -> Functions {
    let functions = block.stmts().map(stmt_get_functions).flatten();

    let functions: Functions = match block.last_stmt() {
        Some(stmt) => functions.chain(last_stmt_get_functions(stmt)).collect(),
        None => functions.collect(),
    };

    functions
}

pub fn get_functions(root: &str, script_node: &SourcemapNode) -> Result<Vec<FunctionCall>, String> {
    let path = script_node.file_paths.iter().find(|item| {
        if let Some(extension) = item.extension() {
            extension == "lua"
        } else {
            false
        }
    });

    if let Some(path) = path {
        let path = String::from(root) + &path.to_string_lossy();

        let contents = fs::read_to_string(&path);

        match contents {
            Ok(contents) => {
                let ast = full_moon::parse(&contents).unwrap().clone();

                let funcs = block_get_functions(ast.nodes())
                    .into_iter()
                    .map(|func| func.clone())
                    .collect();

                Ok(funcs)
            }
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(String::from("Could not get path with .lua extension"))
    }
}
