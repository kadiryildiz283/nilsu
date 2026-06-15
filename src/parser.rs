use std::fs;
use std::path::Path;
use serde::Serialize;
use tree_sitter::{Parser, Node};

#[derive(Debug, Clone, Serialize)]
pub struct ParsedContext {
    pub code_snippet: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub fn get_context(filepath: &str, cursor_line: usize) -> Option<ParsedContext> {
    let path = Path::new(filepath);
    if !path.exists() {
        return None;
    }

    let source_code = fs::read_to_string(path).ok()?;
    
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language()).ok()?;
    let tree = parser.parse(&source_code, None)?;
    let root = tree.root_node();

    // Convert 1-indexed cursor line to 0-indexed row
    let cursor_line_0 = cursor_line.saturating_sub(1);

    // Find the smallest node containing the cursor line
    let smallest_node = find_smallest_node_containing_line(root, cursor_line_0)?;

    // Walk up to find the nearest meaningful ancestor
    let ancestor = find_meaningful_ancestor(smallest_node)?;

    let start_pos = ancestor.start_position();
    let end_pos = ancestor.end_position();

    let start_byte = ancestor.start_byte();
    let end_byte = ancestor.end_byte();
    if start_byte > source_code.len() || end_byte > source_code.len() || start_byte > end_byte {
        return None;
    }
    let code_snippet = source_code[start_byte..end_byte].to_string();

    Some(ParsedContext {
        code_snippet,
        start_line: start_pos.row + 1,
        end_line: end_pos.row + 1,
    })
}

fn find_smallest_node_containing_line(node: Node, line: usize) -> Option<Node> {
    let start_row = node.start_position().row;
    let end_row = node.end_position().row;

    if start_row <= line && end_row >= line {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(n) = find_smallest_node_containing_line(child, line) {
                return Some(n);
            }
        }
        Some(node)
    } else {
        None
    }
}

fn resolve_item(mut node: Node) -> Node {
    while node.kind() == "attribute_item" {
        if let Some(sibling) = node.next_sibling() {
            node = sibling;
        } else {
            break;
        }
    }
    node
}

fn find_meaningful_ancestor(mut node: Node) -> Option<Node> {
    loop {
        let resolved = resolve_item(node);
        let kind = resolved.kind();
        if kind == "function_item"
            || kind == "impl_item"
            || kind == "struct_item"
            || kind == "enum_item"
            || kind == "trait_item"
            || kind == "mod_item"
        {
            return Some(resolved);
        }
        if let Some(parent) = node.parent() {
            node = parent;
        } else {
            return None;
        }
    }
}
