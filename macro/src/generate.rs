use iced_layout_core::Node;
use quote::quote;

use crate::StyleMaps;
use crate::interpolation::generate_text_arg;
use crate::style::{
    generate_button_style_closure, generate_checkbox_style_closure,
    generate_container_style, generate_text_input_style_closure,
};
use crate::types::{
    generate_horizontal, generate_length, generate_line_height, generate_padding,
    generate_shaping, generate_text_alignment, generate_vertical, generate_wrapping,
};

/// Describes how a method-name handler is called.
enum HandlerStyle {
    /// `.method(self.handler())`
    SelfCall,
    /// `.method(|arg| self.handler(arg))`
    Closure(&'static str),
    /// `.method(|| self.handler())`
    Wrapped,
}

/// Generates tokens for an event handler attribute.
///
/// If `val` contains `::`, treat as a direct expression (e.g. `Msg::Click`).
/// Otherwise treat as a method name and wrap according to `handler_style`.
fn generate_event_handler(
    val: &str,
    attr_name: &str,
    method_name: &str,
    handler_style: HandlerStyle,
) -> proc_macro2::TokenStream {
    let iced_method: syn::Ident =
        syn::parse_str(method_name).expect("invalid iced method name");

    if val.contains("::") {
        let msg: syn::Expr = syn::parse_str(val)
            .unwrap_or_else(|e| panic!("invalid {} expression \"{}\": {}", attr_name, val, e));
        quote! { .#iced_method(#msg) }
    } else {
        let handler: syn::Ident = syn::parse_str(val)
            .unwrap_or_else(|e| panic!("invalid {} method name \"{}\": {}", attr_name, val, e));
        match handler_style {
            HandlerStyle::SelfCall => quote! { .#iced_method(self.#handler()) },
            HandlerStyle::Closure(param) => {
                let param: syn::Ident = syn::parse_str(param).unwrap();
                quote! { .#iced_method(|#param| self.#handler(#param)) }
            }
            HandlerStyle::Wrapped => quote! { .#iced_method(|| self.#handler()) },
        }
    }
}

pub fn generate(node: &Node, styles: &StyleMaps) -> proc_macro2::TokenStream {
    match node {
        Node::Text {
            content,
            style,
            attrs,
        } => {
            let text_arg = generate_text_arg(content);
            let mut expr = quote! { iced::widget::text(#text_arg) };
            if let Some(size) = attrs.size {
                expr = quote! { #expr.size(#size) };
            }
            if let Some(ref lh) = attrs.line_height {
                let lh = generate_line_height(lh);
                expr = quote! { #expr.line_height(#lh) };
            }
            if let Some(ref w) = attrs.width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(ref h) = attrs.height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(ref a) = attrs.align_x {
                let a = generate_text_alignment(a);
                expr = quote! { #expr.align_x(#a) };
            }
            if let Some(ref v) = attrs.align_y {
                let v = generate_vertical(v);
                expr = quote! { #expr.align_y(#v) };
            }
            // Inline color wins over style color
            let effective_color = attrs.color.as_ref().or_else(|| {
                style
                    .as_ref()
                    .and_then(|name| styles.text.get(name.as_str()).and_then(|ts| ts.color.as_ref()))
            });
            if let Some(c) = effective_color {
                let c = crate::types::generate_color(c);
                expr = quote! { #expr.color(#c) };
            }
            expr
        }
        Node::Container {
            id,
            style,
            padding,
            children,
        } => {
            assert_eq!(
                children.len(),
                1,
                "<container> must have exactly 1 child element, found {}",
                children.len()
            );
            let child = generate(&children[0], styles);
            let mut expr = quote! { iced::widget::container(#child) };

            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(id_val) = id {
                expr = quote! { #expr.id(#id_val) };
            }
            if let Some(style_name) = style {
                let cs = styles
                    .container
                    .get(style_name.as_str())
                    .unwrap_or_else(|| panic!("unknown container style: \"{}\"", style_name));
                let style_closure = generate_container_style(cs);
                expr = quote! { #expr.style(#style_closure) };
            }
            expr
        }
        Node::Row {
            spacing,
            padding,
            width,
            height,
            align_y,
            clip,
            children,
        } => {
            let child_tokens: Vec<_> = children.iter().map(|c| generate(c, styles)).collect();
            let mut expr = quote! { iced::widget::row![#(#child_tokens),*] };
            if let Some(s) = spacing {
                expr = quote! { #expr.spacing(#s) };
            }
            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(h) = height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(v) = align_y {
                let v = generate_vertical(v);
                expr = quote! { #expr.align_y(#v) };
            }
            if let Some(c) = clip {
                expr = quote! { #expr.clip(#c) };
            }
            expr
        }
        Node::Column {
            spacing,
            padding,
            width,
            height,
            max_width,
            align_x,
            clip,
            children,
        } => {
            let child_tokens: Vec<_> = children.iter().map(|c| generate(c, styles)).collect();
            let mut expr = quote! { iced::widget::column![#(#child_tokens),*] };
            if let Some(s) = spacing {
                expr = quote! { #expr.spacing(#s) };
            }
            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(h) = height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(mw) = max_width {
                expr = quote! { #expr.max_width(#mw) };
            }
            if let Some(h) = align_x {
                let h = generate_horizontal(h);
                expr = quote! { #expr.align_x(#h) };
            }
            if let Some(c) = clip {
                expr = quote! { #expr.clip(#c) };
            }
            expr
        }
        Node::Button {
            style,
            padding,
            width,
            height,
            clip,
            on_press,
            on_press_with,
            on_press_maybe,
            children,
        } => {
            assert!(
                children.len() <= 1,
                "<button> must have at most 1 child element, found {}",
                children.len()
            );
            let child = if children.is_empty() {
                quote! { iced::widget::text("") }
            } else {
                generate(&children[0], styles)
            };
            let mut expr = quote! { iced::widget::button(#child) };

            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(h) = height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(c) = clip {
                expr = quote! { #expr.clip(#c) };
            }
            if let Some(style_name) = style {
                let bs = styles
                    .button
                    .get(style_name.as_str())
                    .unwrap_or_else(|| panic!("unknown button style: \"{}\"", style_name));
                let style_closure = generate_button_style_closure(bs);
                expr = quote! { #expr.style(#style_closure) };
            }
            if let Some(val) = on_press {
                let handler =
                    generate_event_handler(val, "on-press", "on_press", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_press_with {
                let handler = generate_event_handler(
                    val,
                    "on-press-with",
                    "on_press_with",
                    HandlerStyle::Wrapped,
                );
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_press_maybe {
                let handler = generate_event_handler(
                    val,
                    "on-press-maybe",
                    "on_press_maybe",
                    HandlerStyle::SelfCall,
                );
                expr = quote! { #expr #handler };
            }
            expr
        }
        Node::Stack {
            width,
            height,
            clip,
            children,
        } => {
            let child_tokens: Vec<_> = children.iter().map(|c| generate(c, styles)).collect();
            let mut expr = quote! { iced::widget::stack![#(#child_tokens),*] };
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(h) = height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(c) = clip {
                expr = quote! { #expr.clip(#c) };
            }
            expr
        }
        Node::Space { width, height } => {
            let mut expr = quote! { iced::widget::Space::new() };
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(h) = height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            expr
        }
        Node::TextInput {
            placeholder,
            value,
            id,
            secure,
            on_input,
            on_submit,
            on_submit_maybe,
            on_paste,
            width,
            padding,
            size,
            line_height,
            align_x,
            style,
        } => {
            let value_field: syn::Expr = syn::parse_str(&format!("&self.{}", value))
                .unwrap_or_else(|e| panic!("invalid value field path \"{}\": {}", value, e));
            let mut expr = quote! { iced::widget::text_input(#placeholder, #value_field) };
            if let Some(id_val) = id {
                expr = quote! { #expr.id(#id_val) };
            }
            if let Some(s) = secure {
                expr = quote! { #expr.secure(#s) };
            }
            if let Some(val) = on_input {
                let handler = generate_event_handler(
                    val,
                    "on-input",
                    "on_input",
                    HandlerStyle::Closure("s"),
                );
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_submit {
                let handler = generate_event_handler(
                    val,
                    "on-submit",
                    "on_submit",
                    HandlerStyle::SelfCall,
                );
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_submit_maybe {
                let handler = generate_event_handler(
                    val,
                    "on-submit-maybe",
                    "on_submit_maybe",
                    HandlerStyle::SelfCall,
                );
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_paste {
                let handler = generate_event_handler(
                    val,
                    "on-paste",
                    "on_paste",
                    HandlerStyle::Closure("s"),
                );
                expr = quote! { #expr #handler };
            }
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(s) = size {
                expr = quote! { #expr.size(#s) };
            }
            if let Some(lh) = line_height {
                let lh = generate_line_height(lh);
                expr = quote! { #expr.line_height(#lh) };
            }
            if let Some(h) = align_x {
                let h = generate_horizontal(h);
                expr = quote! { #expr.align_x(#h) };
            }
            if let Some(style_name) = style {
                let tis = styles
                    .text_input
                    .get(style_name.as_str())
                    .unwrap_or_else(|| panic!("unknown text-input style: \"{}\"", style_name));
                let style_closure = generate_text_input_style_closure(tis);
                expr = quote! { #expr.style(#style_closure) };
            }
            expr
        }
        Node::Checkbox {
            label,
            is_checked,
            on_toggle,
            on_toggle_maybe,
            size,
            width,
            spacing,
            text_size,
            text_line_height,
            text_shaping,
            text_wrapping,
            style,
        } => {
            let is_checked_field: syn::Expr = syn::parse_str(&format!("self.{}", is_checked))
                .unwrap_or_else(|e| {
                    panic!("invalid is-checked field path \"{}\": {}", is_checked, e)
                });
            let mut expr = quote! { iced::widget::checkbox(#is_checked_field) };
            if !label.is_empty() {
                let label_arg = generate_text_arg(label);
                expr = quote! { #expr.label(#label_arg) };
            }
            if let Some(val) = on_toggle {
                let handler = generate_event_handler(
                    val,
                    "on-toggle",
                    "on_toggle",
                    HandlerStyle::Closure("checked"),
                );
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_toggle_maybe {
                let handler = generate_event_handler(
                    val,
                    "on-toggle-maybe",
                    "on_toggle_maybe",
                    HandlerStyle::SelfCall,
                );
                expr = quote! { #expr #handler };
            }
            if let Some(s) = size {
                expr = quote! { #expr.size(#s) };
            }
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(s) = spacing {
                expr = quote! { #expr.spacing(#s) };
            }
            if let Some(ts) = text_size {
                expr = quote! { #expr.text_size(#ts) };
            }
            if let Some(lh) = text_line_height {
                let lh = generate_line_height(lh);
                expr = quote! { #expr.text_line_height(#lh) };
            }
            if let Some(sh) = text_shaping {
                let sh = generate_shaping(sh);
                expr = quote! { #expr.text_shaping(#sh) };
            }
            if let Some(wr) = text_wrapping {
                let wr = generate_wrapping(wr);
                expr = quote! { #expr.text_wrapping(#wr) };
            }
            if let Some(style_name) = style {
                let cs = styles
                    .checkbox
                    .get(style_name.as_str())
                    .unwrap_or_else(|| panic!("unknown checkbox style: \"{}\"", style_name));
                let style_closure = generate_checkbox_style_closure(cs);
                expr = quote! { #expr.style(#style_closure) };
            }
            expr
        }
    }
}
