use iced_layout_core::Node;
use quote::quote;

use crate::StyleMaps;
use crate::interpolation::generate_text_arg;
use crate::style_var_name;
use crate::types::{
    generate_horizontal, generate_length, generate_line_height, generate_padding,
    generate_shaping, generate_text_alignment, generate_vertical, generate_wrapping,
};

#[derive(Clone, Default)]
pub struct GenerateContext {
    pub local_vars: Vec<String>,
}

pub fn resolve_field_path(path: &str, ctx: &GenerateContext) -> syn::Expr {
    let first_segment = path.split('.').next().unwrap_or(path);
    if ctx.local_vars.iter().any(|v| v == first_segment) {
        syn::parse_str(path)
            .unwrap_or_else(|e| panic!("invalid field path \"{}\": {}", path, e))
    } else {
        syn::parse_str(&format!("self.{}", path))
            .unwrap_or_else(|e| panic!("invalid field path \"{}\": {}", path, e))
    }
}

pub enum Generated {
    Widget(proc_macro2::TokenStream),
    Optional(proc_macro2::TokenStream),
    Multi(proc_macro2::TokenStream),
}

impl Generated {
    pub fn into_widget(self) -> proc_macro2::TokenStream {
        match self {
            Generated::Widget(ts) => ts,
            Generated::Optional(_) => panic!(
                "<if> without <false> branch cannot be used as a single child (e.g. inside <container> or <button>)"
            ),
            Generated::Multi(_) => panic!(
                "<foreach> cannot be used as a single child (e.g. inside <container> or <button>)"
            ),
        }
    }
}

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

fn generate_condition(condition: &str, ctx: &GenerateContext) -> proc_macro2::TokenStream {
    let stripped = condition.trim();
    if let Some(inner) = stripped.strip_prefix('!') {
        let inner = inner.trim();
        let expr = resolve_field_path(inner, ctx);
        quote! { !#expr }
    } else {
        let expr = resolve_field_path(stripped, ctx);
        quote! { #expr }
    }
}

/// Generates a block expression that builds a multi-child container using `let` rebinding.
/// For `Generated::Widget` children, uses `.push()`.
/// For `Generated::Optional` children, uses conditional push to avoid adding empty elements.
/// For `Generated::Multi` children, uses `.extend()` with the iterator mapped to `.into()`.
fn generate_children_block(
    constructor: proc_macro2::TokenStream,
    children: &[Generated],
) -> proc_macro2::TokenStream {
    let mut stmts = vec![quote! { let __w = #constructor; }];
    for child in children {
        match child {
            Generated::Widget(ts) => {
                stmts.push(quote! { let __w = __w.push(#ts); });
            }
            Generated::Optional(ts) => {
                stmts.push(quote! {
                    let __w = { let __opt: Option<iced::Element<'_, _>> = #ts; if let Some(__child) = __opt { __w.push(__child) } else { __w } };
                });
            }
            Generated::Multi(ts) => {
                stmts.push(quote! {
                    let __w = __w.extend(#ts);
                });
            }
        }
    }
    quote! { { #(#stmts)* __w } }
}

fn needs_block(children: &[Generated]) -> bool {
    children.iter().any(|c| matches!(c, Generated::Optional(_) | Generated::Multi(_)))
}

pub fn generate(node: &Node, styles: &StyleMaps, ctx: &GenerateContext) -> Generated {
    match node {
        Node::Text {
            content,
            style,
            attrs,
        } => {
            let text_arg = generate_text_arg(content, ctx);
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
            if let Some(font_name) = &attrs.font {
                assert!(
                    styles.font.contains_key(font_name.as_str()),
                    "unknown font: \"{}\"",
                    font_name
                );
                let var = style_var_name("font", font_name);
                expr = quote! { #expr.font(#var) };
            }
            Generated::Widget(expr)
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
            let child = generate(&children[0], styles, ctx).into_widget();
            let mut expr = quote! { iced::widget::container(#child) };

            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(id_val) = id {
                expr = quote! { #expr.id(#id_val) };
            }
            if let Some(style_name) = style {
                assert!(
                    styles.container.contains_key(style_name.as_str()),
                    "unknown container style: \"{}\"",
                    style_name
                );
                let var = style_var_name("container", style_name);
                expr = quote! { #expr.style(#var) };
            }
            Generated::Widget(expr)
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
            let generated_children: Vec<_> = children.iter().map(|c| generate(c, styles, ctx)).collect();
            let mut expr = if needs_block(&generated_children) {
                generate_children_block(quote! { iced::widget::Row::new() }, &generated_children)
            } else {
                let child_tokens: Vec<_> = generated_children.into_iter().map(|c| match c {
                    Generated::Widget(ts) => ts,
                    _ => unreachable!(),
                }).collect();
                quote! { iced::widget::row![#(#child_tokens),*] }
            };
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
            Generated::Widget(expr)
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
            let generated_children: Vec<_> = children.iter().map(|c| generate(c, styles, ctx)).collect();
            let mut expr = if needs_block(&generated_children) {
                generate_children_block(quote! { iced::widget::Column::new() }, &generated_children)
            } else {
                let child_tokens: Vec<_> = generated_children.into_iter().map(|c| match c {
                    Generated::Widget(ts) => ts,
                    _ => unreachable!(),
                }).collect();
                quote! { iced::widget::column![#(#child_tokens),*] }
            };
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
            Generated::Widget(expr)
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
                generate(&children[0], styles, ctx).into_widget()
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
                assert!(
                    styles.button.contains_key(style_name.as_str()),
                    "unknown button style: \"{}\"",
                    style_name
                );
                let var = style_var_name("button", style_name);
                expr = quote! { #expr.style(#var) };
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
            Generated::Widget(expr)
        }
        Node::Stack {
            width,
            height,
            clip,
            children,
        } => {
            let generated_children: Vec<_> = children.iter().map(|c| generate(c, styles, ctx)).collect();
            let mut expr = if needs_block(&generated_children) {
                generate_children_block(quote! { iced::widget::Stack::new() }, &generated_children)
            } else {
                let child_tokens: Vec<_> = generated_children.into_iter().map(|c| match c {
                    Generated::Widget(ts) => ts,
                    _ => unreachable!(),
                }).collect();
                quote! { iced::widget::stack![#(#child_tokens),*] }
            };
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
            Generated::Widget(expr)
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
            Generated::Widget(expr)
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
            font,
        } => {
            let value_expr = resolve_field_path(value, ctx);
            let mut expr = quote! { iced::widget::text_input(#placeholder, &#value_expr) };
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
            if let Some(font_name) = font {
                assert!(
                    styles.font.contains_key(font_name.as_str()),
                    "unknown font: \"{}\"",
                    font_name
                );
                let var = style_var_name("font", font_name);
                expr = quote! { #expr.font(#var) };
            }
            if let Some(style_name) = style {
                assert!(
                    styles.text_input.contains_key(style_name.as_str()),
                    "unknown text-input style: \"{}\"",
                    style_name
                );
                let var = style_var_name("text_input", style_name);
                expr = quote! { #expr.style(#var) };
            }
            Generated::Widget(expr)
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
            font,
        } => {
            let is_checked_field = resolve_field_path(is_checked, ctx);
            let mut expr = quote! { iced::widget::checkbox(#is_checked_field) };
            if !label.is_empty() {
                let label_arg = generate_text_arg(label, ctx);
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
            if let Some(font_name) = font {
                assert!(
                    styles.font.contains_key(font_name.as_str()),
                    "unknown font: \"{}\"",
                    font_name
                );
                let var = style_var_name("font", font_name);
                expr = quote! { #expr.font(#var) };
            }
            if let Some(style_name) = style {
                assert!(
                    styles.checkbox.contains_key(style_name.as_str()),
                    "unknown checkbox style: \"{}\"",
                    style_name
                );
                let var = style_var_name("checkbox", style_name);
                expr = quote! { #expr.style(#var) };
            }
            Generated::Widget(expr)
        }
        Node::If {
            condition,
            true_branch,
            false_branch,
        } => {
            let cond = generate_condition(condition, ctx);
            let true_expr = generate(true_branch, styles, ctx).into_widget();

            match false_branch {
                Some(false_node) => {
                    let false_expr = generate(false_node, styles, ctx).into_widget();
                    Generated::Widget(quote! {
                        {
                            let __if_result: iced::Element<'_, _> = if #cond {
                                (#true_expr).into()
                            } else {
                                (#false_expr).into()
                            };
                            __if_result
                        }
                    })
                }
                None => {
                    Generated::Optional(quote! {
                        if #cond { Some((#true_expr).into()) } else { None }
                    })
                }
            }
        }
        Node::Widget { method, args, child } => {
            let method_ident: syn::Ident = syn::parse_str(method)
                .unwrap_or_else(|e| panic!("invalid widget method name \"{}\": {}", method, e));
            let mut call_args: Vec<proc_macro2::TokenStream> = Vec::new();
            if let Some(child_node) = child {
                let child_tokens = generate(child_node, styles, ctx).into_widget();
                call_args.push(quote! { (#child_tokens).into() });
            }
            for arg in args {
                let expr = resolve_field_path(arg, ctx);
                call_args.push(quote! { #expr });
            }
            let expr = quote! { self.#method_ident(#(#call_args),*) };
            Generated::Widget(expr)
        }
        Node::VerticalSlider {
            range_start,
            range_end,
            value,
            on_change,
            default,
            on_release,
            width,
            height,
            step,
            shift_step,
        } => {
            let value_expr = resolve_field_path(value, ctx);
            let on_change_ts = if on_change.contains("::") {
                let msg: syn::Expr = syn::parse_str(on_change)
                    .unwrap_or_else(|e| panic!("invalid on-change expression '{}': {}", on_change, e));
                quote! { #msg }
            } else {
                let handler: syn::Ident = syn::parse_str(on_change)
                    .unwrap_or_else(|e| panic!("invalid on-change method name '{}': {}", on_change, e));
                quote! { |v| self.#handler(v) }
            };
            let mut expr = quote! { iced::widget::vertical_slider(#range_start..=#range_end, #value_expr, #on_change_ts) };
            if let Some(d) = default {
                expr = quote! { #expr.default(#d) };
            }
            if let Some(w) = width {
                expr = quote! { #expr.width(#w) };
            }
            if let Some(h) = height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(s) = step {
                let step_expr: syn::Expr = syn::parse_str(s)
                    .unwrap_or_else(|e| panic!("invalid step expression '{}': {}", s, e));
                expr = quote! { #expr.step(#step_expr) };
            }
            if let Some(ss) = shift_step {
                expr = quote! { #expr.shift_step(#ss) };
            }
            if let Some(val) = on_release {
                let handler = generate_event_handler(val, "on-release", "on_release", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            Generated::Widget(expr)
        }
        Node::ForEach { iterable, body } => {
            let iter_field = resolve_field_path(iterable, ctx);
            let mut inner_ctx = ctx.clone();
            inner_ctx.local_vars.push("item".to_string());
            let body_tokens = generate(body, styles, &inner_ctx).into_widget();
            Generated::Multi(quote! {
                #iter_field.iter().map(|item| { (#body_tokens).into() })
            })
        }
    }
}
