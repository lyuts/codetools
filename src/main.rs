use anyhow::{self, Context};
use std::collections::HashMap;
use tree_sitter::{Node, Parser};

fn main() -> anyhow::Result<()> {
    let code = r#"
    fn example() {
        obj.foo();
        let x = bar();
    }

    fn f2() {
        baz(quax(3));
    }
    "#;

    let call_map = find_function_calls(code.to_string())?;

    for (k, v) in call_map.iter() {
        for f in v {
            println!("{} calls {}", k, f);
        }
    }
    Ok(())
}

fn find_function_calls(code: String) -> anyhow::Result<HashMap<String, Vec<String>>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::language();
    parser
        .set_language(&language)
        .context("Failed to load Rust grammar")?;

    let tree = parser
        .parse(code.clone(), None)
        .context("Failed to parse source code")?;
    let mut cursor = tree.walk();
    cursor.goto_first_child();

    let function_name = "".to_string();
    let level = 0;
    let mut calls: HashMap<String, Vec<String>> = HashMap::new();
    loop {
        inspect(
            cursor.node(),
            &code,
            level,
            function_name.clone(),
            &mut calls,
        );
        if !cursor.goto_next_sibling() {
            break;
        }
    }

    Ok(calls)
}

fn inspect(
    node: Node,
    code: &str,
    level: usize,
    caller: String,
    call_map: &mut HashMap<String, Vec<String>>,
) {
    let mut function_name = caller.clone();
    if node.kind() == "function_item" {
        if let Some(function_name_node) = node.child_by_field_name("name") {
            function_name = code[function_name_node.byte_range()].to_string();
        }
    }
    for child in node.children(&mut node.walk()) {
        inspect(child, code, level + 1, function_name.clone(), call_map);
    }
    if node.kind() == "call_expression" {
        if let Some(function_name_node) = node.child_by_field_name("function") {
            let function_name = code[function_name_node.byte_range()].to_string();
            if !call_map.contains_key(&caller) {
                call_map.insert(caller.clone(), vec![]);
            }
            call_map.get_mut(&caller).unwrap().push(function_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn empty_function() {
        let call_map = find_function_calls("fn f1() { }".to_string());
        assert_matches!(call_map, Ok(m) if m.is_empty());
    }

    #[test]
    fn single_call() {
        let call_map = find_function_calls("fn f1() { foo(); }".to_string());
        assert_matches!(call_map, Ok(m) => assert_eq!(m, HashMap::from([("f1".to_string(), vec!["foo".to_string()])])));
    }

    #[test]
    fn two_functions() {
        let call_map = find_function_calls("fn f1() { foo(); } fn f2() { bar(); }".to_string());
        assert_matches!(call_map, Ok(m) => assert_eq!(m, HashMap::from([
            ("f1".to_string(), vec!["foo".to_string()]),
            ("f2".to_string(), vec!["bar".to_string()])
        ])));
    }

    #[test]
    fn nested_calls() {
        let call_map = find_function_calls("fn f1() { foo(bar()); }".to_string());
        let expected_map = HashMap::from([(
            "f1".to_string(),
            vec!["bar", "foo"].iter().map(ToString::to_string).collect(),
        )]);
        assert_matches!(call_map, Ok(m) => assert_eq!(m,expected_map));
    }
}
