use tree_sitter::{Parser, Node};

fn main() {
    let source_code = r#"mod config;
mod server;
mod parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Konfigürasyonu yükle
    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    Ok(())
}
"#;

    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language()).unwrap();
    let tree = parser.parse(&source_code, None).unwrap();
    let root = tree.root_node();

    println!("Tree: {}", root.to_sexp());

    for line in 0..15 {
        let smallest = find_smallest_node_containing_line(root, line);
        let ancestor = smallest.and_then(find_meaningful_ancestor);
        println!(
            "Line {}: smallest={:?}, ancestor={:?}",
            line + 1,
            smallest.map(|n| n.kind()),
            ancestor.map(|n| n.kind())
        );
    }
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
