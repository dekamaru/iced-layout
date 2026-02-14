use iced_layout_core::{
    BorderRadius, ButtonStyle, ButtonStyleFields, CheckboxStyle, Color, ContainerStyle, Horizontal,
    Layout, Length, LineHeight, Node, Padding, Shaping, TextAlignment, TextAttrs,
    TextInputStyle, TextInputStyleFields, TextStyle, Vertical, Wrapping,
};
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Reader;

fn parse_f32_attr(e: &BytesStart, name: &[u8]) -> Option<f32> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name)
        .map(|a| {
            String::from_utf8_lossy(&a.value)
                .parse::<f32>()
                .unwrap_or_else(|err| {
                    panic!(
                        "invalid f32 for {:?}: {}",
                        String::from_utf8_lossy(name),
                        err
                    )
                })
        })
}

fn parse_string_attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

fn parse_length_attr(e: &BytesStart, name: &[u8]) -> Option<Length> {
    parse_string_attr(e, name).map(|s| parse_length(&s))
}

fn parse_length(s: &str) -> Length {
    match s {
        "fill" => Length::Fill,
        "shrink" => Length::Shrink,
        _ if s.starts_with("fill-portion(") && s.ends_with(')') => {
            let inner = &s["fill-portion(".len()..s.len() - 1];
            let v: u16 = inner
                .parse()
                .unwrap_or_else(|e| panic!("invalid fill-portion value: {e}"));
            Length::FillPortion(v)
        }
        _ => {
            let v: f32 = s
                .parse()
                .unwrap_or_else(|e| panic!("invalid length \"{s}\": {e}"));
            Length::Fixed(v)
        }
    }
}

fn parse_hex_color(s: &str) -> Color {
    let hex = s
        .strip_prefix('#')
        .unwrap_or_else(|| panic!("color must start with #, got \"{s}\""));
    let chars: Vec<char> = hex.chars().collect();
    match chars.len() {
        3 => {
            let r = expand_nibble(parse_nibble(chars[0]));
            let g = expand_nibble(parse_nibble(chars[1]));
            let b = expand_nibble(parse_nibble(chars[2]));
            Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }
        }
        4 => {
            let r = expand_nibble(parse_nibble(chars[0]));
            let g = expand_nibble(parse_nibble(chars[1]));
            let b = expand_nibble(parse_nibble(chars[2]));
            let a = expand_nibble(parse_nibble(chars[3]));
            Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            }
        }
        6 => {
            let r = parse_byte(chars[0], chars[1]);
            let g = parse_byte(chars[2], chars[3]);
            let b = parse_byte(chars[4], chars[5]);
            Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }
        }
        8 => {
            let r = parse_byte(chars[0], chars[1]);
            let g = parse_byte(chars[2], chars[3]);
            let b = parse_byte(chars[4], chars[5]);
            let a = parse_byte(chars[6], chars[7]);
            Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            }
        }
        _ => panic!("invalid color format \"{s}\", expected #rgb, #rgba, #rrggbb, or #rrggbbaa"),
    }
}

fn parse_nibble(c: char) -> u8 {
    c.to_digit(16)
        .unwrap_or_else(|| panic!("invalid hex digit '{c}'")) as u8
}

fn expand_nibble(n: u8) -> u8 {
    n << 4 | n
}

fn parse_byte(hi: char, lo: char) -> u8 {
    parse_nibble(hi) << 4 | parse_nibble(lo)
}

fn parse_color_attr(e: &BytesStart, name: &[u8]) -> Option<Color> {
    parse_string_attr(e, name).map(|s| parse_hex_color(&s))
}

fn parse_bool_attr(e: &BytesStart, name: &[u8]) -> Option<bool> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "true" => true,
        "false" => false,
        _ => panic!(
            "invalid bool for {:?}: {}",
            String::from_utf8_lossy(name),
            s
        ),
    })
}

fn parse_horizontal_attr(e: &BytesStart, name: &[u8]) -> Option<Horizontal> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "left" => Horizontal::Left,
        "center" => Horizontal::Center,
        "right" => Horizontal::Right,
        _ => panic!("invalid horizontal alignment: {}", s),
    })
}

fn parse_vertical_attr(e: &BytesStart, name: &[u8]) -> Option<Vertical> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "top" => Vertical::Top,
        "center" => Vertical::Center,
        "bottom" => Vertical::Bottom,
        _ => panic!("invalid vertical alignment: {}", s),
    })
}

fn parse_padding(e: &BytesStart) -> Padding {
    if let Some(all) = parse_f32_attr(e, b"padding") {
        return Padding {
            top: Some(all),
            right: Some(all),
            bottom: Some(all),
            left: Some(all),
        };
    }
    Padding {
        top: parse_f32_attr(e, b"padding-top"),
        right: parse_f32_attr(e, b"padding-right"),
        bottom: parse_f32_attr(e, b"padding-bottom"),
        left: parse_f32_attr(e, b"padding-left"),
    }
}

fn parse_line_height_attr(e: &BytesStart, name: &[u8]) -> Option<LineHeight> {
    parse_string_attr(e, name).map(|s| {
        if s.starts_with("relative(") && s.ends_with(')') {
            let inner = &s["relative(".len()..s.len() - 1];
            let v: f32 = inner
                .parse()
                .unwrap_or_else(|e| panic!("invalid relative line-height value: {e}"));
            LineHeight::Relative(v)
        } else if s.starts_with("absolute(") && s.ends_with(')') {
            let inner = &s["absolute(".len()..s.len() - 1];
            let v: f32 = inner
                .parse()
                .unwrap_or_else(|e| panic!("invalid absolute line-height value: {e}"));
            LineHeight::Absolute(v)
        } else {
            panic!("invalid line-height \"{s}\", expected relative(N) or absolute(N)")
        }
    })
}

fn parse_text_alignment_attr(e: &BytesStart, name: &[u8]) -> Option<TextAlignment> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "default" => TextAlignment::Default,
        "left" => TextAlignment::Left,
        "center" => TextAlignment::Center,
        "right" => TextAlignment::Right,
        "justified" => TextAlignment::Justified,
        _ => panic!("invalid text alignment: {}", s),
    })
}

fn parse_shaping_attr(e: &BytesStart, name: &[u8]) -> Option<Shaping> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "auto" => Shaping::Auto,
        "basic" => Shaping::Basic,
        "advanced" => Shaping::Advanced,
        _ => panic!("invalid shaping: {}", s),
    })
}

fn parse_wrapping_attr(e: &BytesStart, name: &[u8]) -> Option<Wrapping> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "none" => Wrapping::None,
        "word" => Wrapping::Word,
        "glyph" => Wrapping::Glyph,
        "word-or-glyph" => Wrapping::WordOrGlyph,
        _ => panic!("invalid wrapping: {}", s),
    })
}

fn parse_text_attrs(e: &BytesStart) -> TextAttrs {
    TextAttrs {
        size: parse_f32_attr(e, b"size"),
        line_height: parse_line_height_attr(e, b"line-height"),
        width: parse_length_attr(e, b"width"),
        height: parse_length_attr(e, b"height"),
        align_x: parse_text_alignment_attr(e, b"align-x"),
        align_y: parse_vertical_attr(e, b"align-y"),
        color: parse_color_attr(e, b"color"),
    }
}

fn parse_border_radius(e: &BytesStart) -> BorderRadius {
    if let Some(all) = parse_f32_attr(e, b"border-radius") {
        return BorderRadius {
            top_left: Some(all),
            top_right: Some(all),
            bottom_right: Some(all),
            bottom_left: Some(all),
        };
    }
    BorderRadius {
        top_left: parse_f32_attr(e, b"border-radius-top-left"),
        top_right: parse_f32_attr(e, b"border-radius-top-right"),
        bottom_right: parse_f32_attr(e, b"border-radius-bottom-right"),
        bottom_left: parse_f32_attr(e, b"border-radius-bottom-left"),
    }
}

fn parse_container_style(e: &BytesStart) -> (String, ContainerStyle) {
    let id = parse_string_attr(e, b"id").expect("<container-style> requires an 'id' attribute");
    let style = ContainerStyle {
        text_color: parse_color_attr(e, b"text-color"),
        background_color: parse_color_attr(e, b"background-color"),
        border_color: parse_color_attr(e, b"border-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_radius: parse_border_radius(e),
        shadow_color: parse_color_attr(e, b"shadow-color"),
        shadow_offset_x: parse_f32_attr(e, b"shadow-offset-x"),
        shadow_offset_y: parse_f32_attr(e, b"shadow-offset-y"),
        shadow_blur_radius: parse_f32_attr(e, b"shadow-blur-radius"),
        snap: parse_bool_attr(e, b"snap"),
    };
    (id, style)
}

fn parse_text_style(e: &BytesStart) -> (String, TextStyle) {
    let id = parse_string_attr(e, b"id").expect("<text-style> requires an 'id' attribute");
    let style = TextStyle {
        color: parse_color_attr(e, b"color"),
    };
    (id, style)
}

fn parse_button_style_fields(e: &BytesStart) -> ButtonStyleFields {
    ButtonStyleFields {
        text_color: parse_color_attr(e, b"text-color"),
        background_color: parse_color_attr(e, b"background-color"),
        border_color: parse_color_attr(e, b"border-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_radius: parse_border_radius(e),
        shadow_color: parse_color_attr(e, b"shadow-color"),
        shadow_offset_x: parse_f32_attr(e, b"shadow-offset-x"),
        shadow_offset_y: parse_f32_attr(e, b"shadow-offset-y"),
        shadow_blur_radius: parse_f32_attr(e, b"shadow-blur-radius"),
        snap: parse_bool_attr(e, b"snap"),
    }
}

fn assign_button_status_fields(style: &mut ButtonStyle, tag: &[u8], fields: ButtonStyleFields) {
    match tag {
        b"active" => style.active = Some(fields),
        b"hovered" => style.hovered = Some(fields),
        b"pressed" => style.pressed = Some(fields),
        b"disabled" => style.disabled = Some(fields),
        other => panic!(
            "unexpected element in <button-style>: {}",
            String::from_utf8_lossy(other)
        ),
    }
}

fn parse_button_style(e: &BytesStart, reader: &mut Reader<&[u8]>) -> (String, ButtonStyle) {
    let id = parse_string_attr(e, b"id").expect("<button-style> requires an 'id' attribute");
    let base = parse_button_style_fields(e);
    let mut style = ButtonStyle {
        base,
        ..Default::default()
    };

    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => {
                let tag = e.name().as_ref().to_vec();
                let fields = parse_button_style_fields(&e);
                assign_button_status_fields(&mut style, &tag, fields);
                // consume closing tag
                loop {
                    match reader.read_event().expect("failed to read XML") {
                        Event::End(end) if end.name().as_ref() == tag.as_slice() => break,
                        Event::Text(_) | Event::Comment(_) => continue,
                        other => panic!(
                            "expected </{}>, found {:?}",
                            String::from_utf8_lossy(&tag),
                            other
                        ),
                    }
                }
            }
            Event::Empty(e) => {
                let tag = e.name().as_ref().to_vec();
                let fields = parse_button_style_fields(&e);
                assign_button_status_fields(&mut style, &tag, fields);
            }
            Event::End(e) if e.name().as_ref() == b"button-style" => break,
            Event::Text(_) | Event::Comment(_) => continue,
            other => panic!("unexpected event in <button-style>: {:?}", other),
        }
    }

    (id, style)
}

fn parse_checkbox_style(e: &BytesStart) -> (String, CheckboxStyle) {
    let id = parse_string_attr(e, b"id").expect("<checkbox-style> requires an 'id' attribute");
    let style = CheckboxStyle {
        background_color: parse_color_attr(e, b"background-color"),
        icon_color: parse_color_attr(e, b"icon-color"),
        border_color: parse_color_attr(e, b"border-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_radius: parse_border_radius(e),
        text_color: parse_color_attr(e, b"text-color"),
    };
    (id, style)
}

fn parse_text_input_style_fields(e: &BytesStart) -> TextInputStyleFields {
    TextInputStyleFields {
        background_color: parse_color_attr(e, b"background-color"),
        border_color: parse_color_attr(e, b"border-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_radius: parse_border_radius(e),
        icon_color: parse_color_attr(e, b"icon"),
        placeholder_color: parse_color_attr(e, b"placeholder"),
        value_color: parse_color_attr(e, b"value"),
        selection_color: parse_color_attr(e, b"selection"),
    }
}

fn assign_text_input_status_fields(style: &mut TextInputStyle, tag: &[u8], fields: TextInputStyleFields) {
    match tag {
        b"active" => style.active = Some(fields),
        b"hovered" => style.hovered = Some(fields),
        b"disabled" => style.disabled = Some(fields),
        other => panic!(
            "unexpected element in <text-input-style>: {}",
            String::from_utf8_lossy(other)
        ),
    }
}

fn parse_text_input_style(e: &BytesStart, reader: &mut Reader<&[u8]>) -> (String, TextInputStyle) {
    let id = parse_string_attr(e, b"id").expect("<text-input-style> requires an 'id' attribute");
    let base = parse_text_input_style_fields(e);
    let mut style = TextInputStyle {
        base,
        ..Default::default()
    };

    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => {
                let tag = e.name().as_ref().to_vec();
                let fields = parse_text_input_style_fields(&e);
                assign_text_input_status_fields(&mut style, &tag, fields);
                consume_closing_tag(reader, &tag);
            }
            Event::Empty(e) => {
                let tag = e.name().as_ref().to_vec();
                let fields = parse_text_input_style_fields(&e);
                assign_text_input_status_fields(&mut style, &tag, fields);
            }
            Event::End(e) if e.name().as_ref() == b"text-input-style" => break,
            Event::Text(_) | Event::Comment(_) => continue,
            other => panic!("unexpected event in <text-input-style>: {:?}", other),
        }
    }

    (id, style)
}

struct ParsedStyles {
    container: Vec<(String, ContainerStyle)>,
    text: Vec<(String, TextStyle)>,
    button: Vec<(String, ButtonStyle)>,
    checkbox: Vec<(String, CheckboxStyle)>,
    text_input: Vec<(String, TextInputStyle)>,
}

fn consume_closing_tag(reader: &mut Reader<&[u8]>, tag: &[u8]) {
    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::End(end) if end.name().as_ref() == tag => break,
            Event::Text(_) | Event::Comment(_) => continue,
            other => panic!(
                "expected </{}>, found {:?}",
                String::from_utf8_lossy(tag),
                other
            ),
        }
    }
}

fn parse_styles(reader: &mut Reader<&[u8]>) -> ParsedStyles {
    let mut container_styles = Vec::new();
    let mut text_styles = Vec::new();
    let mut button_styles = Vec::new();
    let mut checkbox_styles = Vec::new();
    let mut text_input_styles = Vec::new();

    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => {
                let tag = e.name().as_ref().to_vec();
                match tag.as_slice() {
                    b"container-style" => {
                        container_styles.push(parse_container_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"text-style" => {
                        text_styles.push(parse_text_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"button-style" => {
                        button_styles.push(parse_button_style(&e, reader));
                    }
                    b"checkbox-style" => {
                        checkbox_styles.push(parse_checkbox_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"text-input-style" => {
                        text_input_styles.push(parse_text_input_style(&e, reader));
                    }
                    other => panic!(
                        "unexpected element in <styles>: {}",
                        String::from_utf8_lossy(other)
                    ),
                }
            }
            Event::Empty(e) => match e.name().as_ref() {
                b"container-style" => container_styles.push(parse_container_style(&e)),
                b"text-style" => text_styles.push(parse_text_style(&e)),
                b"button-style" => {
                    let id = parse_string_attr(&e, b"id")
                        .expect("<button-style> requires an 'id' attribute");
                    let base = parse_button_style_fields(&e);
                    button_styles.push((id, ButtonStyle { base, ..Default::default() }));
                }
                b"checkbox-style" => checkbox_styles.push(parse_checkbox_style(&e)),
                b"text-input-style" => {
                    let id = parse_string_attr(&e, b"id")
                        .expect("<text-input-style> requires an 'id' attribute");
                    let base = parse_text_input_style_fields(&e);
                    text_input_styles.push((id, TextInputStyle { base, ..Default::default() }));
                }
                other => panic!(
                    "unexpected element in <styles>: {}",
                    String::from_utf8_lossy(other)
                ),
            },
            Event::End(e) if e.name().as_ref() == b"styles" => break,
            Event::Text(_) | Event::Comment(_) => continue,
            other => panic!("unexpected event in <styles>: {:?}", other),
        }
    }

    ParsedStyles {
        container: container_styles,
        text: text_styles,
        button: button_styles,
        checkbox: checkbox_styles,
        text_input: text_input_styles,
    }
}

pub fn parse(xml: &str) -> Layout {
    let mut reader = Reader::from_str(xml);
    // Find <layout>
    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) if e.name().as_ref() == b"layout" => break,
            Event::Decl(_) | Event::Text(_) | Event::Comment(_) => continue,
            other => panic!("expected <layout> root element, found {:?}", other),
        }
    }

    let mut styles = ParsedStyles {
        container: Vec::new(),
        text: Vec::new(),
        button: Vec::new(),
        checkbox: Vec::new(),
        text_input: Vec::new(),
    };
    let mut root = None;

    // Parse <styles> and <root> children (order-independent)
    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => match e.name().as_ref() {
                b"styles" => {
                    styles = parse_styles(&mut reader);
                }
                b"root" => {
                    root = Some(parse_node(&mut reader));
                    // consume </root>
                    loop {
                        match reader.read_event().expect("failed to read XML") {
                            Event::End(end) if end.name().as_ref() == b"root" => break,
                            Event::Text(_) | Event::Comment(_) => continue,
                            other => panic!("expected </root>, found {:?}", other),
                        }
                    }
                }
                other => panic!(
                    "expected <styles> or <root>, found <{}>",
                    String::from_utf8_lossy(other)
                ),
            },
            Event::Empty(_e) if _e.name().as_ref() == b"styles" => {
                // empty <styles/> — no styles defined
            }
            Event::End(e) if e.name().as_ref() == b"layout" => break,
            Event::Text(_) | Event::Comment(_) => continue,
            other => panic!("unexpected event in <layout>: {:?}", other),
        }
    }

    Layout {
        container_styles: styles.container,
        text_styles: styles.text,
        button_styles: styles.button,
        checkbox_styles: styles.checkbox,
        text_input_styles: styles.text_input,
        root: root.expect("<layout> must contain a <root> element"),
    }
}

fn parse_node(reader: &mut Reader<&[u8]>) -> Node {
    loop {
        // Unify Start and Empty events: both carry attributes via BytesStart.
        // `has_closing_tag` tells us whether to consume a closing tag afterward.
        let (e, has_closing_tag) = match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => (e.into_owned(), true),
            Event::Empty(e) => (e.into_owned(), false),
            Event::End(_) => {
                return Node::Text {
                    content: String::new(),
                    style: None,
                    attrs: TextAttrs::default(),
                }
            }
            Event::Text(_) | Event::Comment(_) | Event::Decl(_) => continue,
            Event::Eof => panic!("unexpected end of XML"),
            _ => continue,
        };

        let tag = e.name().as_ref().to_vec();
        return match tag.as_slice() {
            b"container" => {
                let id = parse_string_attr(&e, b"id");
                let style = parse_string_attr(&e, b"style");
                let padding = parse_padding(&e);

                let mut children = Vec::new();
                if has_closing_tag {
                    loop {
                        let child = parse_node(reader);
                        match child {
                            Node::Text { ref content, .. } if content.is_empty() => break,
                            _ => children.push(child),
                        }
                    }
                }
                Node::Container { id, style, padding, children }
            }
            b"text" => {
                let style = parse_string_attr(&e, b"style");
                let attrs = parse_text_attrs(&e);
                let mut content = String::new();
                if has_closing_tag {
                    loop {
                        match reader.read_event().expect("failed to read XML") {
                            Event::Text(e) => {
                                content.push_str(
                                    &e.unescape().expect("failed to unescape text"),
                                );
                            }
                            Event::End(e) if e.name().as_ref() == b"text" => break,
                            _ => {}
                        }
                    }
                }
                Node::Text { content, style, attrs }
            }
            b"row" => {
                let spacing = parse_f32_attr(&e, b"spacing");
                let padding = parse_padding(&e);
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let align_y = parse_vertical_attr(&e, b"align-y");
                let clip = parse_bool_attr(&e, b"clip");

                let mut children = Vec::new();
                if has_closing_tag {
                    loop {
                        let child = parse_node(reader);
                        match child {
                            Node::Text { ref content, .. } if content.is_empty() => break,
                            _ => children.push(child),
                        }
                    }
                }
                Node::Row { spacing, padding, width, height, align_y, clip, children }
            }
            b"column" => {
                let spacing = parse_f32_attr(&e, b"spacing");
                let padding = parse_padding(&e);
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let max_width = parse_f32_attr(&e, b"max-width");
                let align_x = parse_horizontal_attr(&e, b"align-x");
                let clip = parse_bool_attr(&e, b"clip");

                let mut children = Vec::new();
                if has_closing_tag {
                    loop {
                        let child = parse_node(reader);
                        match child {
                            Node::Text { ref content, .. } if content.is_empty() => break,
                            _ => children.push(child),
                        }
                    }
                }
                Node::Column { spacing, padding, width, height, max_width, align_x, clip, children }
            }
            b"button" => {
                let style = parse_string_attr(&e, b"style");
                let padding = parse_padding(&e);
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let clip = parse_bool_attr(&e, b"clip");
                let on_press = parse_string_attr(&e, b"on-press");
                let on_press_with = parse_string_attr(&e, b"on-press-with");
                let on_press_maybe = parse_string_attr(&e, b"on-press-maybe");

                let mut children = Vec::new();
                if has_closing_tag {
                    loop {
                        let child = parse_node(reader);
                        match child {
                            Node::Text { ref content, .. } if content.is_empty() => break,
                            _ => children.push(child),
                        }
                    }
                }
                Node::Button { style, padding, width, height, clip, on_press, on_press_with, on_press_maybe, children }
            }
            b"stack" => {
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let clip = parse_bool_attr(&e, b"clip");

                let mut children = Vec::new();
                if has_closing_tag {
                    loop {
                        let child = parse_node(reader);
                        match child {
                            Node::Text { ref content, .. } if content.is_empty() => break,
                            _ => children.push(child),
                        }
                    }
                }
                Node::Stack { width, height, clip, children }
            }
            b"space" => {
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                if has_closing_tag {
                    consume_closing_tag(reader, b"space");
                }
                Node::Space { width, height }
            }
            b"text-input" => {
                let placeholder = parse_string_attr(&e, b"placeholder")
                    .expect("<text-input> requires a 'placeholder' attribute");
                let value = parse_string_attr(&e, b"value")
                    .expect("<text-input> requires a 'value' attribute");
                let id = parse_string_attr(&e, b"id");
                let secure = parse_bool_attr(&e, b"secure");
                let on_input = parse_string_attr(&e, b"on-input");
                let on_submit = parse_string_attr(&e, b"on-submit");
                let on_submit_maybe = parse_string_attr(&e, b"on-submit-maybe");
                let on_paste = parse_string_attr(&e, b"on-paste");
                let width = parse_length_attr(&e, b"width");
                let padding = parse_padding(&e);
                let size = parse_f32_attr(&e, b"size");
                let line_height = parse_line_height_attr(&e, b"line-height");
                let align_x = parse_horizontal_attr(&e, b"align-x");
                let style = parse_string_attr(&e, b"style");
                if has_closing_tag {
                    consume_closing_tag(reader, b"text-input");
                }
                Node::TextInput {
                    placeholder, value, id, secure, on_input,
                    on_submit, on_submit_maybe, on_paste,
                    width, padding, size, line_height, align_x, style,
                }
            }
            b"checkbox" => {
                let is_checked = parse_string_attr(&e, b"is-checked")
                    .expect("<checkbox> requires an 'is-checked' attribute");
                let on_toggle = parse_string_attr(&e, b"on-toggle");
                let on_toggle_maybe = parse_string_attr(&e, b"on-toggle-maybe");
                let size = parse_f32_attr(&e, b"size");
                let width = parse_length_attr(&e, b"width");
                let spacing = parse_f32_attr(&e, b"spacing");
                let text_size = parse_f32_attr(&e, b"text-size");
                let text_line_height = parse_line_height_attr(&e, b"text-line-height");
                let text_shaping = parse_shaping_attr(&e, b"text-shaping");
                let text_wrapping = parse_wrapping_attr(&e, b"text-wrapping");
                let style = parse_string_attr(&e, b"style");

                let mut label = String::new();
                if has_closing_tag {
                    loop {
                        match reader.read_event().expect("failed to read XML") {
                            Event::Text(t) => {
                                label.push_str(
                                    &t.unescape().expect("failed to unescape text"),
                                );
                            }
                            Event::End(end) if end.name().as_ref() == b"checkbox" => break,
                            _ => {}
                        }
                    }
                }
                Node::Checkbox {
                    label, is_checked, on_toggle, on_toggle_maybe,
                    size, width, spacing, text_size, text_line_height,
                    text_shaping, text_wrapping, style,
                }
            }
            other => panic!(
                "unsupported tag: {}",
                String::from_utf8_lossy(other)
            ),
        };
    }
}
