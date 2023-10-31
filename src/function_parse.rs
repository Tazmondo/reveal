use full_moon::ast::{
    Call, Expression, FunctionArgs, FunctionCall, Index, Prefix, Suffix, Value, Var,
};

fn expression_to_string(expression: &Expression) -> Option<Vec<String>> {
    match expression {
        Expression::Parentheses {
            contained: _,
            expression,
        } => expression_to_string(expression),
        Expression::Value {
            value,
            type_assertion: _,
        } => match &**value {
            //  Get reference to box contents
            Value::Var(var) => match var {
                Var::Name(name) => Some(vec![name.to_string()]),
                Var::Expression(expression) => {
                    let prefix = match expression.prefix() {
                        Prefix::Name(name) => Some(name.to_string()),
                        _ => None,
                    };

                    let suffixes: Vec<Option<String>> = expression
                        .suffixes()
                        .map(|suffix| match suffix {
                            Suffix::Index(index) => match index {
                                Index::Dot { dot: _, name } => Some(name.to_string()),
                                _ => None,
                            },
                            _ => None,
                        })
                        .collect();

                    vec![prefix]
                        .into_iter()
                        .chain(suffixes.into_iter())
                        .collect()
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn get_require_tokens(func: &FunctionCall) -> Option<Vec<String>> {
    let mut suffixes = func.suffixes();

    let Some(suffix) = suffixes.next() else {
        return None;
    };

    let args: Option<Vec<String>> = match suffix {
        Suffix::Call(call) => match call {
            Call::AnonymousCall(args) => match args {
                FunctionArgs::Parentheses {
                    parentheses: _,
                    arguments,
                } => match arguments.iter().next() {
                    // Only take the first argument since it's require
                    None => None,
                    Some(expression) => expression_to_string(expression),
                },
                _ => None,
            },
            _ => None,
        },
        _ => None,
    };

    args
}

fn remove_preceding_comments(str: &String) -> Option<String> {
    str.lines()
        .last()
        .and_then(|str| Some(String::from(str.trim())))
}

pub fn print_function(file_path: &str, func: &FunctionCall) {
    let Some(prefix) = (match func.prefix() {
        Prefix::Name(prefix) => Some(prefix.to_string()),
        _ => None,
    }) else {
        return;
    };

    let Some(prefix) = remove_preceding_comments(&prefix) else {
        return;
    };

    let is_require = prefix == "require";

    if is_require {
        let Some(require_path) = get_require_tokens(func) else {
            return;
        };
        println!("{}: {:?} ", file_path, require_path);
    }
}
