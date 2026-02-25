use iced_layout_core::Node;
use quote::quote;

use crate::StyleMaps;
use crate::interpolation::generate_text_arg;
use crate::style_var_name;
use crate::types::{
    generate_horizontal, generate_interaction, generate_length, generate_line_height,
    generate_padding, generate_shaping, generate_text_alignment, generate_tooltip_position,
    generate_vertical, generate_wrapping,
};
use iced_layout_core::PickListHandle;


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
            assert_eq!(
                children.len(),
                1,
                "<button> must have exactly 1 child element, found {}",
                children.len()
            );
            let child = generate(&children[0], styles, ctx).into_widget();
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
            icon,
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
            if let Some(icon_name) = icon {
                assert!(
                    styles.text_input_icons.contains_key(icon_name.as_str()),
                    "unknown text-input-icon: \"{}\"",
                    icon_name
                );
                let var = style_var_name("text_input_icon", icon_name);
                expr = quote! { #expr.icon(#var) };
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
            icon,
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
            if let Some(icon_name) = icon {
                assert!(
                    styles.checkbox_icons.contains_key(icon_name.as_str()),
                    "unknown checkbox-icon: \"{}\"",
                    icon_name
                );
                let var = style_var_name("checkbox_icon", icon_name);
                expr = quote! { #expr.icon(#var) };
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
        Node::Tooltip {
            position,
            gap,
            padding,
            delay,
            snap_within_viewport,
            style,
            children,
        } => {
            assert_eq!(
                children.len(),
                2,
                "<tooltip> must have exactly 2 children (content, tooltip), found {}",
                children.len()
            );
            let content = generate(&children[0], styles, ctx).into_widget();
            let tooltip_widget = generate(&children[1], styles, ctx).into_widget();
            let pos = generate_tooltip_position(position);
            let mut expr = quote! { iced::widget::tooltip(#content, #tooltip_widget, #pos) };
            if let Some(g) = gap {
                expr = quote! { #expr.gap(#g) };
            }
            if let Some(p) = padding {
                expr = quote! { #expr.padding(#p) };
            }
            if let Some(d) = delay {
                expr = quote! { #expr.delay(::std::time::Duration::from_millis(#d)) };
            }
            if let Some(snap) = snap_within_viewport {
                expr = quote! { #expr.snap_within_viewport(#snap) };
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
        Node::Toggler {
            is_toggled,
            label,
            on_toggle,
            on_toggle_maybe,
            size,
            width,
            text_size,
            text_line_height,
            text_alignment,
            text_shaping,
            text_wrapping,
            spacing,
            font,
            style,
        } => {
            let is_toggled_field = resolve_field_path(is_toggled, ctx);
            let mut expr = quote! { iced::widget::toggler(#is_toggled_field) };
            if let Some(lbl) = label {
                let lbl_arg = generate_text_arg(lbl, ctx);
                expr = quote! { #expr.label(#lbl_arg) };
            }
            if let Some(val) = on_toggle {
                let handler = generate_event_handler(
                    val,
                    "on-toggle",
                    "on_toggle",
                    HandlerStyle::Closure("toggled"),
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
            if let Some(ts) = text_size {
                expr = quote! { #expr.text_size(#ts) };
            }
            if let Some(lh) = text_line_height {
                let lh = generate_line_height(lh);
                expr = quote! { #expr.text_line_height(#lh) };
            }
            if let Some(ta) = text_alignment {
                let ta = generate_text_alignment(ta);
                expr = quote! { #expr.text_alignment(#ta) };
            }
            if let Some(sh) = text_shaping {
                let sh = generate_shaping(sh);
                expr = quote! { #expr.text_shaping(#sh) };
            }
            if let Some(wr) = text_wrapping {
                let wr = generate_wrapping(wr);
                expr = quote! { #expr.text_wrapping(#wr) };
            }
            if let Some(s) = spacing {
                expr = quote! { #expr.spacing(#s) };
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
                    styles.toggler.contains_key(style_name.as_str()),
                    "unknown toggler style: \"{}\"",
                    style_name
                );
                let var = style_var_name("toggler", style_name);
                expr = quote! { #expr.style(#var) };
            }
            Generated::Widget(expr)
        }
        Node::Sensor {
            on_show,
            on_resize,
            on_hide,
            anticipate,
            delay,
            children,
        } => {
            assert_eq!(
                children.len(),
                1,
                "<sensor> must have exactly 1 child element, found {}",
                children.len()
            );
            let child = generate(&children[0], styles, ctx).into_widget();
            let mut expr = quote! { iced::widget::sensor(#child) };
            if let Some(val) = on_show {
                let handler = generate_event_handler(
                    val,
                    "on-show",
                    "on_show",
                    HandlerStyle::Closure("size"),
                );
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_resize {
                let handler = generate_event_handler(
                    val,
                    "on-resize",
                    "on_resize",
                    HandlerStyle::Closure("size"),
                );
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_hide {
                let handler =
                    generate_event_handler(val, "on-hide", "on_hide", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(a) = anticipate {
                expr = quote! { #expr.anticipate(#a) };
            }
            if let Some(d) = delay {
                expr = quote! { #expr.delay(::std::time::Duration::from_millis(#d)) };
            }
            Generated::Widget(expr)
        }
        Node::ComboBox {
            state,
            placeholder,
            selection,
            on_selected,
            on_input,
            on_option_hovered,
            on_open,
            on_close,
            padding,
            font,
            size,
            line_height,
            width,
            menu_height,
            text_shaping,
            input_style,
            menu_style,
            icon,
        } => {
            let state_expr = resolve_field_path(state, ctx);
            let selection_expr = resolve_field_path(selection, ctx);
            let on_selected_ts = if on_selected.contains("::") {
                let msg: syn::Expr = syn::parse_str(on_selected)
                    .unwrap_or_else(|e| panic!("invalid on-selected expression '{}': {}", on_selected, e));
                quote! { #msg }
            } else {
                let handler: syn::Ident = syn::parse_str(on_selected)
                    .unwrap_or_else(|e| panic!("invalid on-selected method name '{}': {}", on_selected, e));
                quote! { |item| self.#handler(item) }
            };
            let mut expr = quote! {
                iced::widget::combo_box(&#state_expr, #placeholder, #selection_expr.as_ref(), #on_selected_ts)
            };
            if let Some(val) = on_input {
                let handler = generate_event_handler(val, "on-input", "on_input", HandlerStyle::Closure("s"));
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_option_hovered {
                let handler = generate_event_handler(val, "on-option-hovered", "on_option_hovered", HandlerStyle::Closure("item"));
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_open {
                let handler = generate_event_handler(val, "on-open", "on_open", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_close {
                let handler = generate_event_handler(val, "on-close", "on_close", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
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
            if let Some(s) = size {
                expr = quote! { #expr.size(#s) };
            }
            if let Some(lh) = line_height {
                let lh = generate_line_height(lh);
                expr = quote! { #expr.line_height(#lh) };
            }
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(mh) = menu_height {
                let mh = generate_length(mh);
                expr = quote! { #expr.menu_height(#mh) };
            }
            if let Some(sh) = text_shaping {
                let sh = generate_shaping(sh);
                expr = quote! { #expr.text_shaping(#sh) };
            }
            if let Some(icon_name) = icon {
                assert!(
                    styles.text_input_icons.contains_key(icon_name.as_str()),
                    "unknown text-input-icon: \"{}\"",
                    icon_name
                );
                let var = style_var_name("text_input_icon", icon_name);
                expr = quote! { #expr.icon(#var) };
            }
            if let Some(style_name) = input_style {
                assert!(
                    styles.text_input.contains_key(style_name.as_str()),
                    "unknown text-input style: \"{}\"",
                    style_name
                );
                let var = style_var_name("text_input", style_name);
                expr = quote! { #expr.input_style(#var) };
            }
            if let Some(style_name) = menu_style {
                assert!(
                    styles.overlay_menu.contains_key(style_name.as_str()),
                    "unknown overlay-menu style: \"{}\"",
                    style_name
                );
                let var = style_var_name("overlay_menu", style_name);
                expr = quote! { #expr.menu_style(#var) };
            }
            Generated::Widget(expr)
        }
        Node::TextEditor {
            content,
            id,
            placeholder,
            width,
            height,
            min_height,
            max_height,
            on_action,
            font,
            size,
            line_height,
            padding,
            wrapping,
            key_binding,
            style,
        } => {
            let content_expr = resolve_field_path(content, ctx);
            let mut expr = quote! { iced::widget::text_editor(&#content_expr) };
            if let Some(id_val) = id {
                expr = quote! { #expr.id(#id_val) };
            }
            if let Some(ph) = placeholder {
                expr = quote! { #expr.placeholder(#ph) };
            }
            if let Some(w) = width {
                expr = quote! { #expr.width(#w) };
            }
            if let Some(h) = height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(mh) = min_height {
                expr = quote! { #expr.min_height(#mh) };
            }
            if let Some(mh) = max_height {
                expr = quote! { #expr.max_height(#mh) };
            }
            if let Some(val) = on_action {
                let handler = generate_event_handler(
                    val,
                    "on-action",
                    "on_action",
                    HandlerStyle::Closure("action"),
                );
                expr = quote! { #expr #handler };
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
            if let Some(s) = size {
                expr = quote! { #expr.size(#s) };
            }
            if let Some(lh) = line_height {
                let lh = generate_line_height(lh);
                expr = quote! { #expr.line_height(#lh) };
            }
            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(wr) = wrapping {
                let wr = generate_wrapping(wr);
                expr = quote! { #expr.wrapping(#wr) };
            }
            if let Some(kb) = key_binding {
                let kb_handler = generate_event_handler(
                    kb,
                    "key-binding",
                    "key_binding",
                    HandlerStyle::Closure("key_press"),
                );
                expr = quote! { #expr #kb_handler };
            }
            if let Some(style_name) = style {
                assert!(
                    styles.text_editor.contains_key(style_name.as_str()),
                    "unknown text-editor style: \"{}\"",
                    style_name
                );
                let var = style_var_name("text_editor", style_name);
                expr = quote! { #expr.style(#var) };
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
        Node::Float { scale, translate, style, children } => {
            assert_eq!(
                children.len(),
                1,
                "<float> must have exactly 1 child element, found {}",
                children.len()
            );
            let child = generate(&children[0], styles, ctx).into_widget();
            let mut expr = quote! { iced::widget::Float::new(#child) };
            if let Some(s) = scale {
                expr = quote! { #expr.scale(#s) };
            }
            if let Some(t) = translate {
                let handler: syn::Ident = syn::parse_str(t)
                    .unwrap_or_else(|e| panic!("invalid translate method name \"{}\": {}", t, e));
                expr = quote! { #expr.translate(|bounds, viewport| self.#handler(bounds, viewport)) };
            }
            if let Some(style_name) = style {
                assert!(
                    styles.float.contains_key(style_name.as_str()),
                    "unknown float style: \"{}\"",
                    style_name
                );
                let var = style_var_name("float", style_name);
                expr = quote! { #expr.style(#var) };
            }
            Generated::Widget(expr)
        }
        Node::MouseArea {
            on_press,
            on_release,
            on_double_click,
            on_right_press,
            on_right_release,
            on_middle_press,
            on_middle_release,
            on_scroll,
            on_enter,
            on_move,
            on_exit,
            interaction,
            children,
        } => {
            assert_eq!(
                children.len(),
                1,
                "<mouse-area> must have exactly 1 child element, found {}",
                children.len()
            );
            let child = generate(&children[0], styles, ctx).into_widget();
            let mut expr = quote! { iced::widget::mouse_area(#child) };
            if let Some(val) = on_press {
                let handler = generate_event_handler(val, "on-press", "on_press", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_release {
                let handler = generate_event_handler(val, "on-release", "on_release", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_double_click {
                let handler = generate_event_handler(val, "on-double-click", "on_double_click", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_right_press {
                let handler = generate_event_handler(val, "on-right-press", "on_right_press", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_right_release {
                let handler = generate_event_handler(val, "on-right-release", "on_right_release", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_middle_press {
                let handler = generate_event_handler(val, "on-middle-press", "on_middle_press", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_middle_release {
                let handler = generate_event_handler(val, "on-middle-release", "on_middle_release", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_scroll {
                let handler = generate_event_handler(val, "on-scroll", "on_scroll", HandlerStyle::Closure("delta"));
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_enter {
                let handler = generate_event_handler(val, "on-enter", "on_enter", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_move {
                let handler = generate_event_handler(val, "on-move", "on_move", HandlerStyle::Closure("point"));
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_exit {
                let handler = generate_event_handler(val, "on-exit", "on_exit", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(i) = interaction {
                let i = generate_interaction(i);
                expr = quote! { #expr.interaction(#i) };
            }
            Generated::Widget(expr)
        }
        Node::PickList {
            options,
            selected,
            on_select,
            placeholder,
            width,
            menu_height,
            padding,
            text_size,
            text_line_height,
            text_shaping,
            font,
            handle,
            on_open,
            on_close,
            style,
            menu_style,
        } => {
            let options_expr = resolve_field_path(options, ctx);
            let selected_expr = resolve_field_path(selected, ctx);
            let on_select_ts = if on_select.contains("::") {
                let msg: syn::Expr = syn::parse_str(on_select)
                    .unwrap_or_else(|e| panic!("invalid on-select expression '{}': {}", on_select, e));
                quote! { #msg }
            } else {
                let handler: syn::Ident = syn::parse_str(on_select)
                    .unwrap_or_else(|e| panic!("invalid on-select method name '{}': {}", on_select, e));
                quote! { |item| self.#handler(item) }
            };
            let mut expr = quote! {
                iced::widget::pick_list(#options_expr.clone(), #selected_expr.clone(), #on_select_ts)
            };
            if let Some(ph) = placeholder {
                expr = quote! { #expr.placeholder(#ph) };
            }
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(mh) = menu_height {
                let mh = generate_length(mh);
                expr = quote! { #expr.menu_height(#mh) };
            }
            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
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
            if let Some(font_name) = font {
                assert!(
                    styles.font.contains_key(font_name.as_str()),
                    "unknown font: \"{}\"",
                    font_name
                );
                let var = style_var_name("font", font_name);
                expr = quote! { #expr.font(#var) };
            }
            if let Some(h) = handle {
                let handle_ts = match h {
                    PickListHandle::Arrow { size } => {
                        let sz = match size {
                            Some(s) => quote! { Some(iced::Pixels(#s)) },
                            None => quote! { None },
                        };
                        quote! { iced::widget::pick_list::Handle::Arrow { size: #sz } }
                    }
                    PickListHandle::Static { icon } => {
                        assert!(
                            styles.pick_list_icons.contains_key(icon.as_str()),
                            "unknown pick-list-icon: \"{}\"",
                            icon
                        );
                        let var = style_var_name("pick_list_icon", icon);
                        quote! { iced::widget::pick_list::Handle::Static(#var) }
                    }
                    PickListHandle::Dynamic { closed, open } => {
                        assert!(
                            styles.pick_list_icons.contains_key(closed.as_str()),
                            "unknown pick-list-icon: \"{}\"",
                            closed
                        );
                        assert!(
                            styles.pick_list_icons.contains_key(open.as_str()),
                            "unknown pick-list-icon: \"{}\"",
                            open
                        );
                        let closed_var = style_var_name("pick_list_icon", closed);
                        let open_var = style_var_name("pick_list_icon", open);
                        quote! { iced::widget::pick_list::Handle::Dynamic { open: #open_var, closed: #closed_var } }
                    }
                    PickListHandle::None => quote! { iced::widget::pick_list::Handle::None },
                };
                expr = quote! { #expr.handle(#handle_ts) };
            }
            if let Some(val) = on_open {
                let handler = generate_event_handler(val, "on-open", "on_open", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(val) = on_close {
                let handler = generate_event_handler(val, "on-close", "on_close", HandlerStyle::SelfCall);
                expr = quote! { #expr #handler };
            }
            if let Some(style_name) = style {
                assert!(
                    styles.pick_list.contains_key(style_name.as_str()),
                    "unknown pick-list style: \"{}\"",
                    style_name
                );
                let var = style_var_name("pick_list", style_name);
                expr = quote! { #expr.style(#var) };
            }
            if let Some(style_name) = menu_style {
                assert!(
                    styles.overlay_menu.contains_key(style_name.as_str()),
                    "unknown overlay-menu style: \"{}\"",
                    style_name
                );
                let var = style_var_name("overlay_menu", style_name);
                expr = quote! { #expr.menu_style(#var) };
            }
            Generated::Widget(expr)
        }
        Node::ProgressBar {
            range_start,
            range_end,
            value,
            length,
            girth,
            style,
        } => {
            let value_expr = resolve_field_path(value, ctx);
            let mut expr =
                quote! { iced::widget::progress_bar(#range_start..=#range_end, #value_expr) };
            if let Some(l) = length {
                let l = generate_length(l);
                expr = quote! { #expr.length(#l) };
            }
            if let Some(g) = girth {
                let g = generate_length(g);
                expr = quote! { #expr.girth(#g) };
            }
            if let Some(style_name) = style {
                assert!(
                    styles.progress_bar.contains_key(style_name.as_str()),
                    "unknown progress-bar style: \"{}\"",
                    style_name
                );
                let var = style_var_name("progress_bar", style_name);
                expr = quote! { #expr.style(#var) };
            }
            Generated::Widget(expr)
        }
        Node::Pin {
            width,
            height,
            x,
            y,
            children,
        } => {
            assert_eq!(
                children.len(),
                1,
                "<pin> must have exactly 1 child element, found {}",
                children.len()
            );
            let child = generate(&children[0], styles, ctx).into_widget();
            let mut expr = quote! { iced::widget::pin(#child) };
            if let Some(w) = width {
                let w = generate_length(w);
                expr = quote! { #expr.width(#w) };
            }
            if let Some(h) = height {
                let h = generate_length(h);
                expr = quote! { #expr.height(#h) };
            }
            if let Some(x) = x {
                expr = quote! { #expr.x(#x) };
            }
            if let Some(y) = y {
                expr = quote! { #expr.y(#y) };
            }
            Generated::Widget(expr)
        }
    }
}
