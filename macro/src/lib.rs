extern crate proc_macro;

mod generate;
mod interpolation;
mod style;
mod types;

use proc_macro::TokenStream;
use std::collections::HashMap;
use std::path::PathBuf;

use iced_layout_core::{
    ButtonStyle, CheckboxStyle, ContainerStyle, FontDef, OverlayMenuStyle, TextEditorStyle,
    TextInputStyle, TextStyle, TogglerStyle,
};
use quote::{format_ident, quote};
use syn::{LitStr, parse_macro_input};

use crate::style::{
    generate_button_style_closure, generate_checkbox_style_closure, generate_container_style,
    generate_overlay_menu_style_closure, generate_text_editor_style_closure,
    generate_text_input_style_closure, generate_toggler_style_closure,
};
use crate::types::generate_font_def;

pub(crate) struct StyleMaps<'a> {
    pub container: HashMap<&'a str, &'a ContainerStyle>,
    pub text: HashMap<&'a str, &'a TextStyle>,
    pub button: HashMap<&'a str, &'a ButtonStyle>,
    pub checkbox: HashMap<&'a str, &'a CheckboxStyle>,
    pub text_input: HashMap<&'a str, &'a TextInputStyle>,
    pub toggler: HashMap<&'a str, &'a TogglerStyle>,
    pub text_editor: HashMap<&'a str, &'a TextEditorStyle>,
    pub overlay_menu: HashMap<&'a str, &'a OverlayMenuStyle>,
    pub font: HashMap<&'a str, &'a FontDef>,
}

pub(crate) fn style_var_name(prefix: &str, name: &str) -> syn::Ident {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    format_ident!("__style_{}_{}", prefix, sanitized)
}

/// Reads an XML layout file at compile time and generates iced widget code.
///
/// The path is relative to `CARGO_MANIFEST_DIR`.
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

    let layout = iced_layout_xml::parse(&xml);

    let style_maps = StyleMaps {
        container: layout.container_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        text: layout.text_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        button: layout.button_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        checkbox: layout.checkbox_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        text_input: layout.text_input_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        toggler: layout.toggler_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        text_editor: layout.text_editor_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        overlay_menu: layout.overlay_menu_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        font: layout.font_defs.iter().map(|(k, v)| (k.as_str(), v)).collect(),
    };

    let mut style_bindings = Vec::new();

    for (name, cs) in &style_maps.container {
        let var = style_var_name("container", name);
        let closure = generate_container_style(cs);
        style_bindings.push(quote! { let #var = #closure; });
    }
    for (name, bs) in &style_maps.button {
        let var = style_var_name("button", name);
        let closure = generate_button_style_closure(bs);
        style_bindings.push(quote! { let #var = #closure; });
    }
    for (name, cs) in &style_maps.checkbox {
        let var = style_var_name("checkbox", name);
        let closure = generate_checkbox_style_closure(cs);
        style_bindings.push(quote! { let #var = #closure; });
    }
    for (name, tis) in &style_maps.text_input {
        let var = style_var_name("text_input", name);
        let closure = generate_text_input_style_closure(tis);
        style_bindings.push(quote! { let #var = #closure; });
    }
    for (name, ts) in &style_maps.toggler {
        let var = style_var_name("toggler", name);
        let closure = generate_toggler_style_closure(ts);
        style_bindings.push(quote! { let #var = #closure; });
    }
    for (name, tes) in &style_maps.text_editor {
        let var = style_var_name("text_editor", name);
        let closure = generate_text_editor_style_closure(tes);
        style_bindings.push(quote! { let #var = #closure; });
    }
    for (name, oms) in &style_maps.overlay_menu {
        let var = style_var_name("overlay_menu", name);
        let closure = generate_overlay_menu_style_closure(oms);
        style_bindings.push(quote! { let #var = #closure; });
    }
    for (name, fd) in &style_maps.font {
        let var = style_var_name("font", name);
        let font_expr = generate_font_def(fd);
        style_bindings.push(quote! { let #var = #font_expr; });
    }

    let ctx = generate::GenerateContext::default();
    let tokens = generate::generate(&layout.root, &style_maps, &ctx).into_widget();

    let expanded = if style_bindings.is_empty() {
        quote! { #tokens.into() }
    } else {
        quote! { { #(#style_bindings)* #tokens.into() } }
    };
    expanded.into()
}
