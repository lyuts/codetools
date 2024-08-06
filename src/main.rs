use tree_sitter::{Node, Parser, TreeCursor};

fn main() {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::language();
    parser.set_language(&language);

    let code = r#"
    fn example() {
        obj.foo();
        let x = bar();
    }
    "#;

    let tree = parser.parse(code, None).unwrap();
    let mut cursor = tree.walk();

    if cursor.node().kind() == "source_file" {
        cursor.goto_first_child();
    }

    let node = cursor.node();
    let node_type = node.kind();
    assert_eq!(node_type, "function_item");

    find_function_calls(&mut cursor, code);
}

fn find_function_calls(cursor: &mut TreeCursor, code: &str) {
    cursor.goto_first_child();
    let starting_node = cursor.node();
    let level = 0;
    while cursor.goto_next_sibling() {
        inspect(cursor.node(), code, level);
    }
}

fn inspect(node: Node, code: &str, level: usize) {
    for child in node.children(&mut node.walk()) {
        inspect(child, code, level + 1);
    }

    if node.kind() == "call_expression" {
        if let Some(function_name_node) = node.child_by_field_name("function") {
            let function_name = code[function_name_node.byte_range()].to_string();
            println!("{}Function call: {}", "\t".repeat(level), function_name);
        }
    }
}
