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

enum Segment {
    Literal(String),
    Variable(String),
}

fn parse_interpolation(text: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("${") {
        if start > 0 {
            segments.push(Segment::Literal(rest[..start].to_string()));
        }
        let after = &rest[start + 2..];
        let end = after.find('}').expect("unclosed ${...} in text content");
        let var_name = after[..end].trim();
        assert!(!var_name.is_empty(), "empty variable name in interpolation");
        segments.push(Segment::Variable(var_name.to_string()));
        rest = &after[end + 1..];
    }

    if !rest.is_empty() {
        segments.push(Segment::Literal(rest.to_string()));
    }
    segments
}

fn generate_text_arg(content: &str) -> proc_macro2::TokenStream {
    let segments = parse_interpolation(content);

    // No interpolation — plain string literal
    if segments.iter().all(|s| matches!(s, Segment::Literal(_))) {
        return quote! { #content };
    }

    // Single variable, no surrounding text
    if segments.len() == 1 {
        if let Segment::Variable(ref path) = segments[0] {
            let field: syn::Expr = syn::parse_str(&format!("&self.{}", path))
                .expect("invalid variable path in ${...}");
            return quote! { #field };
        }
    }

    // Mixed content — build a format!() call
    let mut fmt_str = String::new();
    let mut args: Vec<proc_macro2::TokenStream> = Vec::new();
    for seg in &segments {
        match seg {
            Segment::Literal(s) => {
                // Escape braces for format!()
                fmt_str.push_str(&s.replace('{', "{{").replace('}', "}}"));
            }
            Segment::Variable(path) => {
                fmt_str.push_str("{}");
                let field: syn::Expr = syn::parse_str(&format!("self.{}", path))
                    .expect("invalid variable path in ${...}");
                args.push(quote! { #field });
            }
        }
    }
    quote! { format!(#fmt_str, #(#args),*) }
}

fn generate(node: &Node) -> proc_macro2::TokenStream {
    match node {
        Node::Text { content, attrs } => {
            let text_arg = generate_text_arg(content);
            let mut expr = quote! { iced::widget::text(#text_arg) };
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
