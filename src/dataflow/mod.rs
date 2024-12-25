use anyhow::Context;
use tracing::{debug, trace, warn};
use tree_sitter::Parser;

#[derive(Debug, PartialEq)]
pub struct Parameter {
    name: String,
    r#type: String,
}

pub fn find_accessible_data(function_name: &str, code: &str) -> anyhow::Result<Vec<Parameter>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser
        .set_language(&language.into())
        .context("Failed to load Rust grammar")?;

    let tree = parser
        .parse(code, None)
        .context("Failed to parse source code")?;

    let root = tree.root_node();
    let mut variables = Vec::new();
    let mut cursor = root.walk();

    if cursor.goto_first_child() {
        'outer: loop {
            if cursor.node().kind() == "function_item" {
                if let Some(name_node) = cursor.node().child_by_field_name("name") {
                    let start = name_node.start_byte();
                    let end = name_node.end_byte();
                    let actual_name = &code[start..end];
                    trace!("Found function definition {}", actual_name);
                    if actual_name == function_name {
                        trace!("Inspecting data...");
                        let params_node = cursor.node().child_by_field_name("parameters").unwrap();
                        trace!("parameters found {}", params_node);
                        for child in params_node.children(&mut cursor) {
                            if child.kind() == "parameter" {
                                let pattern_node = child.child_by_field_name("pattern").unwrap();
                                let type_node = child.child_by_field_name("type").unwrap();

                                let pattern =
                                    &code[pattern_node.start_byte()..pattern_node.end_byte()];
                                let type_name = &code[type_node.start_byte()..type_node.end_byte()];

                                trace!("pattern = {}, type = {}", pattern, type_name);
                                variables.push(Parameter {
                                    name: pattern.to_string(),
                                    r#type: type_name.to_string(),
                                });
                            }
                        }

                        // let body_node = cursor.node().child_by_field_name("body").unwrap();
                        // for statement in body_node.children(&mut cursor) {
                        //     if statement.kind() == "let_statement" {
                        //         if let Some(name_node) = statement.child_by_field_name("name") {
                        //             variables.push(name_node.to_string());
                        //         }
                        //     }
                        // }

                        debug!(
                            "Data accessible in function '{}': {:?}",
                            function_name, variables
                        );
                        break 'outer;
                    }
                }
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    } else {
        warn!("Function '{}' not found in the code.", function_name);
    }
    Ok(variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn empty_function() {
        let data = find_accessible_data("f1", "fn f1() { }");
        assert_matches!(data, Ok(v) if v.is_empty());
    }

    #[test]
    fn single_primitive() {
        let data = find_accessible_data("f1", "fn f1(x: u32) { }");
        assert_matches!(data, Ok(v) if v == vec![Parameter{name: "x".to_string(), r#type:"u32".to_string()}]);
    }

    #[test]
    fn multiple_arguments() {
        let data = find_accessible_data("f1", "fn f1(x: u32, f: bool) { }");
        assert_matches!(data, Ok(v) if v == vec![
            Parameter{name: "x".to_string(), r#type: "u32".to_string()},
            Parameter{name: "f".to_string(), r#type: "bool".to_string()},
        ]);
    }
}
