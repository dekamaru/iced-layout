extern crate proc_macro;

use proc_macro::TokenStream;
use std::path::PathBuf;

use iced_layout_core::{Color, Length, Node, Padding};
use quote::quote;
use syn::{LitStr, parse_macro_input};

fn generate_padding(padding: &Padding) -> Option<proc_macro2::TokenStream> {
    if padding.top.is_none() && padding.right.is_none()
        && padding.bottom.is_none() && padding.left.is_none()
    {
        return None;
    }

    let top = padding.top.unwrap_or(0.0);
    let right = padding.right.unwrap_or(0.0);
    let bottom = padding.bottom.unwrap_or(0.0);
    let left = padding.left.unwrap_or(0.0);
    Some(quote! {
        iced::Padding { top: #top, right: #right, bottom: #bottom, left: #left }
    })
}

fn generate_length(len: &Length) -> proc_macro2::TokenStream {
    match len {
        Length::Fill => quote! { iced::Length::Fill },
        Length::FillPortion(v) => quote! { iced::Length::FillPortion(#v) },
        Length::Shrink => quote! { iced::Length::Shrink },
        Length::Fixed(v) => quote! { iced::Length::Fixed(#v) },
    }
}

fn generate_color(c: &Color) -> proc_macro2::TokenStream {
    let r = c.r;
    let g = c.g;
    let b = c.b;
    let a = c.a;
    quote! { iced::Color { r: #r, g: #g, b: #b, a: #a } }
}

fn generate(node: &Node) -> proc_macro2::TokenStream {
    match node {
        Node::Text { content, attrs } => {
            let mut expr = quote! { iced::widget::text(#content) };
            if let Some(size) = attrs.size {
                expr = quote! { #expr.size(#size) };
            }
            if let Some(ref w) = attrs.width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(ref h) = attrs.height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(ref c) = attrs.color {
                let c = generate_color(c);
                expr = quote! { #expr.color(#c) };
            }
            expr
        }
        Node::Container { id, padding, children } => {
            assert_eq!(children.len(), 1, "<container> must have exactly 1 child element, found {}", children.len());
            let child = generate(&children[0]);
            let mut expr = quote! { iced::widget::container(#child) };

            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(id_val) = id {
                expr = quote! { #expr.id(#id_val) };
            }
            expr
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
