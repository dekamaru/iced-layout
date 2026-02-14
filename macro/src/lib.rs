extern crate proc_macro;

use proc_macro::TokenStream;
use std::collections::HashMap;
use std::path::PathBuf;

use iced_layout_core::{
    BorderRadius, ButtonStyle, ButtonStyleFields, CheckboxStyle, Color, ContainerStyle, Horizontal,
    Length, LineHeight, Node, Padding, Shaping, TextAlignment, TextStyle, Vertical, Wrapping,
};
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

fn generate_horizontal(h: &Horizontal) -> proc_macro2::TokenStream {
    match h {
        Horizontal::Left => quote! { iced::alignment::Horizontal::Left },
        Horizontal::Center => quote! { iced::alignment::Horizontal::Center },
        Horizontal::Right => quote! { iced::alignment::Horizontal::Right },
    }
}

fn generate_vertical(v: &Vertical) -> proc_macro2::TokenStream {
    match v {
        Vertical::Top => quote! { iced::alignment::Vertical::Top },
        Vertical::Center => quote! { iced::alignment::Vertical::Center },
        Vertical::Bottom => quote! { iced::alignment::Vertical::Bottom },
    }
}

fn generate_line_height(lh: &LineHeight) -> proc_macro2::TokenStream {
    match lh {
        LineHeight::Relative(v) => quote! { iced::widget::text::LineHeight::Relative(#v) },
        LineHeight::Absolute(v) => quote! { iced::widget::text::LineHeight::Absolute(iced::Pixels(#v)) },
    }
}

fn generate_text_alignment(a: &TextAlignment) -> proc_macro2::TokenStream {
    match a {
        TextAlignment::Default => quote! { iced::widget::text::Alignment::Default },
        TextAlignment::Left => quote! { iced::widget::text::Alignment::Left },
        TextAlignment::Center => quote! { iced::widget::text::Alignment::Center },
        TextAlignment::Right => quote! { iced::widget::text::Alignment::Right },
        TextAlignment::Justified => quote! { iced::widget::text::Alignment::Justified },
    }
}

fn generate_shaping(s: &Shaping) -> proc_macro2::TokenStream {
    match s {
        Shaping::Auto => quote! { iced::widget::text::Shaping::Auto },
        Shaping::Basic => quote! { iced::widget::text::Shaping::Basic },
        Shaping::Advanced => quote! { iced::widget::text::Shaping::Advanced },
    }
}

fn generate_wrapping(w: &Wrapping) -> proc_macro2::TokenStream {
    match w {
        Wrapping::None => quote! { iced::widget::text::Wrapping::None },
        Wrapping::Word => quote! { iced::widget::text::Wrapping::Word },
        Wrapping::Glyph => quote! { iced::widget::text::Wrapping::Glyph },
        Wrapping::WordOrGlyph => quote! { iced::widget::text::Wrapping::WordOrGlyph },
    }
}

fn generate_container_style(s: &ContainerStyle) -> proc_macro2::TokenStream {
    let text_color = match &s.text_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { Some(#c) }
        }
        None => quote! { None },
    };

    let background = match &s.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { Some(iced::Background::Color(#c)) }
        }
        None => quote! { None },
    };

    let border_color = match &s.border_color {
        Some(c) => generate_color(c),
        None => quote! { iced::Color::TRANSPARENT },
    };
    let border_width = s.border_width.unwrap_or(0.0);
    let border_radius = generate_border_radius(&s.border_radius);

    let shadow_color = match &s.shadow_color {
        Some(c) => generate_color(c),
        None => quote! { iced::Color::TRANSPARENT },
    };
    let shadow_ox = s.shadow_offset_x.unwrap_or(0.0);
    let shadow_oy = s.shadow_offset_y.unwrap_or(0.0);
    let shadow_blur = s.shadow_blur_radius.unwrap_or(0.0);

    let snap = s.snap.unwrap_or(false);

    quote! {
        |_theme| iced::widget::container::Style {
            text_color: #text_color,
            background: #background,
            border: iced::Border {
                color: #border_color,
                width: #border_width,
                radius: #border_radius,
            },
            shadow: iced::Shadow {
                color: #shadow_color,
                offset: iced::Vector::new(#shadow_ox, #shadow_oy),
                blur_radius: #shadow_blur,
            },
            snap: #snap,
        }
    }
}

fn generate_border_radius(br: &BorderRadius) -> proc_macro2::TokenStream {
    let tl = br.top_left.unwrap_or(0.0);
    let tr = br.top_right.unwrap_or(0.0);
    let brr = br.bottom_right.unwrap_or(0.0);
    let bl = br.bottom_left.unwrap_or(0.0);
    quote! { iced::border::Radius { top_left: #tl, top_right: #tr, bottom_right: #brr, bottom_left: #bl } }
}

struct StyleMaps<'a> {
    container: HashMap<&'a str, &'a ContainerStyle>,
    text: HashMap<&'a str, &'a TextStyle>,
    button: HashMap<&'a str, &'a ButtonStyle>,
    checkbox: HashMap<&'a str, &'a CheckboxStyle>,
}

fn generate_checkbox_style_closure(s: &CheckboxStyle) -> proc_macro2::TokenStream {
    let background = match &s.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { iced::Background::Color(#c) }
        }
        None => quote! { iced::Background::Color(iced::Color::TRANSPARENT) },
    };

    let icon_color = match &s.icon_color {
        Some(c) => generate_color(c),
        None => quote! { iced::Color::BLACK },
    };

    let border_color = match &s.border_color {
        Some(c) => generate_color(c),
        None => quote! { iced::Color::TRANSPARENT },
    };
    let border_width = s.border_width.unwrap_or(0.0);
    let border_radius = generate_border_radius(&s.border_radius);

    let text_color = match &s.text_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { Some(#c) }
        }
        None => quote! { None },
    };

    quote! {
        |_theme, _status| iced::widget::checkbox::Style {
            background: #background,
            icon_color: #icon_color,
            border: iced::Border {
                color: #border_color,
                width: #border_width,
                radius: #border_radius,
            },
            text_color: #text_color,
        }
    }
}

fn generate_button_fields_tokens(fields: &ButtonStyleFields) -> proc_macro2::TokenStream {
    let background = match &fields.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { Some(iced::Background::Color(#c)) }
        }
        None => quote! { None },
    };

    let text_color = match &fields.text_color {
        Some(c) => generate_color(c),
        None => quote! { iced::Color::BLACK },
    };

    let border_color = match &fields.border_color {
        Some(c) => generate_color(c),
        None => quote! { iced::Color::TRANSPARENT },
    };
    let border_width = fields.border_width.unwrap_or(0.0);
    let border_radius = generate_border_radius(&fields.border_radius);

    let shadow_color = match &fields.shadow_color {
        Some(c) => generate_color(c),
        None => quote! { iced::Color::TRANSPARENT },
    };
    let shadow_ox = fields.shadow_offset_x.unwrap_or(0.0);
    let shadow_oy = fields.shadow_offset_y.unwrap_or(0.0);
    let shadow_blur = fields.shadow_blur_radius.unwrap_or(0.0);
    let snap = fields.snap.unwrap_or(false);

    quote! {
        iced::widget::button::Style {
            background: #background,
            text_color: #text_color,
            border: iced::Border {
                color: #border_color,
                width: #border_width,
                radius: #border_radius,
            },
            shadow: iced::Shadow {
                color: #shadow_color,
                offset: iced::Vector::new(#shadow_ox, #shadow_oy),
                blur_radius: #shadow_blur,
            },
            snap: #snap,
        }
    }
}

fn merge_button_fields(base: &ButtonStyleFields, overlay: &ButtonStyleFields) -> ButtonStyleFields {
    ButtonStyleFields {
        text_color: overlay.text_color.or(base.text_color),
        background_color: overlay.background_color.or(base.background_color),
        border_color: overlay.border_color.or(base.border_color),
        border_width: overlay.border_width.or(base.border_width),
        border_radius: BorderRadius {
            top_left: overlay.border_radius.top_left.or(base.border_radius.top_left),
            top_right: overlay.border_radius.top_right.or(base.border_radius.top_right),
            bottom_right: overlay.border_radius.bottom_right.or(base.border_radius.bottom_right),
            bottom_left: overlay.border_radius.bottom_left.or(base.border_radius.bottom_left),
        },
        shadow_color: overlay.shadow_color.or(base.shadow_color),
        shadow_offset_x: overlay.shadow_offset_x.or(base.shadow_offset_x),
        shadow_offset_y: overlay.shadow_offset_y.or(base.shadow_offset_y),
        shadow_blur_radius: overlay.shadow_blur_radius.or(base.shadow_blur_radius),
        snap: overlay.snap.or(base.snap),
    }
}

fn generate_button_style_closure(bs: &ButtonStyle) -> proc_macro2::TokenStream {
    let base_style = generate_button_fields_tokens(&bs.base);

    let status_overrides: Vec<_> = [
        (&bs.active, quote! { iced::widget::button::Status::Active }),
        (&bs.hovered, quote! { iced::widget::button::Status::Hovered }),
        (&bs.pressed, quote! { iced::widget::button::Status::Pressed }),
        (&bs.disabled, quote! { iced::widget::button::Status::Disabled }),
    ]
    .into_iter()
    .filter_map(|(opt, status_token)| {
        opt.as_ref().map(|fields| {
            let merged = merge_button_fields(&bs.base, fields);
            let style = generate_button_fields_tokens(&merged);
            quote! { #status_token => #style }
        })
    })
    .collect();

    if status_overrides.is_empty() {
        quote! {
            |_theme, _status| #base_style
        }
    } else {
        quote! {
            |_theme, status| match status {
                #(#status_overrides,)*
                _ => #base_style
            }
        }
    }
}

fn generate(node: &Node, styles: &StyleMaps) -> proc_macro2::TokenStream {
    match node {
        Node::Text { content, style, attrs } => {
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
                style.as_ref().and_then(|name| {
                    styles.text.get(name.as_str()).and_then(|ts| ts.color.as_ref())
                })
            });
            if let Some(c) = effective_color {
                let c = generate_color(c);
                expr = quote! { #expr.color(#c) };
            }
            expr
        }
        Node::Container { id, style, padding, children } => {
            assert_eq!(children.len(), 1, "<container> must have exactly 1 child element, found {}", children.len());
            let child = generate(&children[0], styles);
            let mut expr = quote! { iced::widget::container(#child) };

            if let Some(padding_expr) = generate_padding(padding) {
                expr = quote! { #expr.padding(#padding_expr) };
            }
            if let Some(id_val) = id {
                expr = quote! { #expr.id(#id_val) };
            }
            if let Some(style_name) = style {
                let cs = styles.container.get(style_name.as_str())
                    .unwrap_or_else(|| panic!("unknown container style: \"{}\"", style_name));
                let style_closure = generate_container_style(cs);
                expr = quote! { #expr.style(#style_closure) };
            }
            expr
        }
        Node::Row { spacing, padding, width, height, align_y, clip, children } => {
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
        Node::Column { spacing, padding, width, height, max_width, align_x, clip, children } => {
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
        Node::Button { style, padding, width, height, clip, on_press, on_press_with, on_press_maybe, children } => {
            assert!(children.len() <= 1, "<button> must have at most 1 child element, found {}", children.len());
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
                let bs = styles.button.get(style_name.as_str())
                    .unwrap_or_else(|| panic!("unknown button style: \"{}\"", style_name));
                let style_closure = generate_button_style_closure(bs);
                expr = quote! { #expr.style(#style_closure) };
            }
            if let Some(val) = on_press {
                if val.contains("::") {
                    let msg: syn::Expr = syn::parse_str(val)
                        .unwrap_or_else(|e| panic!("invalid on-press expression \"{}\": {}", val, e));
                    expr = quote! { #expr.on_press(#msg) };
                } else {
                    let method: syn::Ident = syn::parse_str(val)
                        .unwrap_or_else(|e| panic!("invalid on-press method name \"{}\": {}", val, e));
                    expr = quote! { #expr.on_press(self.#method()) };
                }
            }
            if let Some(val) = on_press_with {
                let method: syn::Ident = syn::parse_str(val)
                    .unwrap_or_else(|e| panic!("invalid on-press-with method name \"{}\": {}", val, e));
                expr = quote! { #expr.on_press_with(|| self.#method()) };
            }
            if let Some(val) = on_press_maybe {
                let method: syn::Ident = syn::parse_str(val)
                    .unwrap_or_else(|e| panic!("invalid on-press-maybe method name \"{}\": {}", val, e));
                expr = quote! { #expr.on_press_maybe(self.#method()) };
            }
            expr
        }
        Node::Stack { width, height, clip, children } => {
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
        Node::Checkbox {
            label, is_checked, on_toggle, on_toggle_maybe,
            size, width, spacing, text_size, text_line_height,
            text_shaping, text_wrapping, style,
        } => {
            let is_checked_field: syn::Expr = syn::parse_str(&format!("self.{}", is_checked))
                .unwrap_or_else(|e| panic!("invalid is-checked field path \"{}\": {}", is_checked, e));
            let mut expr = quote! { iced::widget::checkbox(#is_checked_field) };
            if !label.is_empty() {
                let label_arg = generate_text_arg(label);
                expr = quote! { #expr.label(#label_arg) };
            }
            if let Some(val) = on_toggle {
                if val.contains("::") {
                    let msg: syn::Expr = syn::parse_str(val)
                        .unwrap_or_else(|e| panic!("invalid on-toggle expression \"{}\": {}", val, e));
                    expr = quote! { #expr.on_toggle(#msg) };
                } else {
                    let method: syn::Ident = syn::parse_str(val)
                        .unwrap_or_else(|e| panic!("invalid on-toggle method name \"{}\": {}", val, e));
                    expr = quote! { #expr.on_toggle(|checked| self.#method(checked)) };
                }
            }
            if let Some(val) = on_toggle_maybe {
                let method: syn::Ident = syn::parse_str(val)
                    .unwrap_or_else(|e| panic!("invalid on-toggle-maybe method name \"{}\": {}", val, e));
                expr = quote! { #expr.on_toggle_maybe(self.#method()) };
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
                let cs = styles.checkbox.get(style_name.as_str())
                    .unwrap_or_else(|| panic!("unknown checkbox style: \"{}\"", style_name));
                let style_closure = generate_checkbox_style_closure(cs);
                expr = quote! { #expr.style(#style_closure) };
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

    let layout = iced_layout_xml::parse(&xml);

    let style_maps = StyleMaps {
        container: layout.container_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        text: layout.text_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        button: layout.button_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        checkbox: layout.checkbox_styles.iter().map(|(k, v)| (k.as_str(), v)).collect(),
    };

    let tokens = generate(&layout.root, &style_maps);

    let expanded = quote! { #tokens.into() };
    expanded.into()
}
