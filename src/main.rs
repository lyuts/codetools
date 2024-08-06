use anyhow::{self, Context};
use std::collections::HashMap;
use tree_sitter::{Node, Parser};

fn main() -> anyhow::Result<()> {
    let code = r#"
    fn example() {
        obj.foo();
        let x = bar();
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

    if cursor.node().kind() == "source_file" {
        cursor.goto_first_child();
    }

    let node = cursor.node();
    let node_type = node.kind();
    assert_eq!(node_type, "function_item");

    let mut function_name = "".to_string();
    if let Some(function_name_node) = node.child_by_field_name("name") {
        function_name = code[function_name_node.byte_range()].to_string();
        println!("Function call graph belongs to {}", function_name);
    }

    cursor.goto_first_child();
    let level = 0;
    let mut calls: HashMap<String, Vec<String>> = HashMap::new();
    while cursor.goto_next_sibling() {
        inspect(
            cursor.node(),
            &code,
            level,
            function_name.clone(),
            &mut calls,
        );
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
    for child in node.children(&mut node.walk()) {
        if node.kind() == "function_item" {
            if let Some(function_name_node) = node.child_by_field_name("name") {
                function_name = code[function_name_node.byte_range()].to_string();
            }
        }
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
