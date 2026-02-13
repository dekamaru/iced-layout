extern crate proc_macro;

use proc_macro::TokenStream;
use std::path::PathBuf;

use iced_layout_core::Node;
use quote::quote;
use syn::{LitStr, parse_macro_input};

fn generate(node: &Node) -> proc_macro2::TokenStream {
    match node {
        Node::Text(content) => {
            quote! { iced::widget::text(#content) }
        }
        Node::Container { id, children } => {
            assert_eq!(children.len(), 1, "<container> must have exactly 1 child element, found {}", children.len());
            let child = generate(&children[0]);
            let inner = quote! { iced::widget::container(#child) };

            if let Some(id_val) = id {
                quote! { #inner.id(#id_val) }
            } else {
                inner
            }
        }
    }
}

/// Reads an XML layout file at compile time and generates iced widget code.
///
/// The path is relative to `CARGO_MANIFEST_DIR`.
/// Supports `<container>` and `<text>` tags.
///
/// # Example
/// ```ignore
/// layout!("src/page/test-layout.xml")
/// ```
#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let rel_path = lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let file_path = PathBuf::from(&manifest_dir).join(&rel_path);

    let xml = std::fs::read_to_string(&file_path).unwrap_or_else(|e| {
        panic!("failed to read {}: {}", file_path.display(), e)
    });

    let root = iced_layout_xml::parse(&xml);
    let tokens = generate(&root);

    let expanded = quote! { #tokens.into() };
    expanded.into()
}
