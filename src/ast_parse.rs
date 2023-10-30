// This module extracts function calls from the AST.
// I could have written this to be more versatile, but I only care about function calls
// and this kind of code is just annoying to write
// also not sure how i'd handle the enums of enums properly

use std::{fs, path::Path, vec};

use full_moon::ast::{
    self, Assignment, Block, Expression, FunctionCall, If, LastStmt, Stmt, Value,
};

type Functions<'a> = Vec<&'a FunctionCall>;

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

pub fn get_functions(path: &Path) -> Result<Vec<FunctionCall>, String> {
    let contents = fs::read_to_string(path);

    match contents {
        Ok(contents) => {
            let ast = full_moon::parse(&contents).unwrap().clone();

            let funcs = block_get_functions(ast.nodes())
                .into_iter()
                .map(|func| func.clone())
                .collect();

            Ok(funcs)
        }
        Err(err) => Err(format!("{}: {}", path.display(), err.to_string())),
    }
}
