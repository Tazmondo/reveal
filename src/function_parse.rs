use full_moon::ast::{FunctionCall, Prefix};

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

    println!("{}: {} {}", file_path, prefix, is_require)
}
