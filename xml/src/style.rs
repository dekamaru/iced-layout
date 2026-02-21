use iced_layout_core::{
    ButtonStyle, ButtonStyleFields, CheckboxStyle, ContainerStyle, FontDef, TextInputStyle,
    TextInputStyleFields, TextStyle, TogglerStyle,
};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::attr::*;

pub struct ParsedStyles {
    pub container: Vec<(String, ContainerStyle)>,
    pub text: Vec<(String, TextStyle)>,
    pub button: Vec<(String, ButtonStyle)>,
    pub checkbox: Vec<(String, CheckboxStyle)>,
    pub text_input: Vec<(String, TextInputStyle)>,
    pub toggler: Vec<(String, TogglerStyle)>,
    pub font: Vec<(String, FontDef)>,
}

impl Default for ParsedStyles {
    fn default() -> Self {
        Self {
            container: Vec::new(),
            text: Vec::new(),
            button: Vec::new(),
            checkbox: Vec::new(),
            text_input: Vec::new(),
            toggler: Vec::new(),
            font: Vec::new(),
        }
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

fn assign_text_input_status_fields(
    style: &mut TextInputStyle,
    tag: &[u8],
    fields: TextInputStyleFields,
) {
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

/// Parses stateful style sub-elements (active/hovered/pressed/disabled) from the reader.
fn parse_stateful_children<S>(
    reader: &mut Reader<&[u8]>,
    parent_tag: &[u8],
    style: &mut S,
    mut assign: impl FnMut(&mut S, &[u8], &BytesStart),
) {
    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => {
                let tag = e.name().as_ref().to_vec();
                assign(style, &tag, &e);
                consume_closing_tag(reader, &tag);
            }
            Event::Empty(e) => {
                let tag = e.name().as_ref().to_vec();
                assign(style, &tag, &e);
            }
            Event::End(e) if e.name().as_ref() == parent_tag => break,
            Event::Text(_) | Event::Comment(_) => continue,
            other => panic!(
                "unexpected event in <{}>: {:?}",
                String::from_utf8_lossy(parent_tag),
                other
            ),
        }
    }
}

fn parse_button_style(e: &BytesStart, reader: &mut Reader<&[u8]>) -> (String, ButtonStyle) {
    let id = parse_string_attr(e, b"id").expect("<button-style> requires an 'id' attribute");
    let base = parse_button_style_fields(e);
    let mut style = ButtonStyle {
        base,
        ..Default::default()
    };

    parse_stateful_children(reader, b"button-style", &mut style, |s, tag, e| {
        assign_button_status_fields(s, tag, parse_button_style_fields(e))
    });

    (id, style)
}

fn parse_button_style_empty(e: &BytesStart) -> (String, ButtonStyle) {
    let id = parse_string_attr(e, b"id").expect("<button-style> requires an 'id' attribute");
    let base = parse_button_style_fields(e);
    (id, ButtonStyle { base, ..Default::default() })
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

fn parse_text_input_style(
    e: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> (String, TextInputStyle) {
    let id =
        parse_string_attr(e, b"id").expect("<text-input-style> requires an 'id' attribute");
    let base = parse_text_input_style_fields(e);
    let mut style = TextInputStyle {
        base,
        ..Default::default()
    };

    parse_stateful_children(reader, b"text-input-style", &mut style, |s, tag, e| {
        assign_text_input_status_fields(s, tag, parse_text_input_style_fields(e))
    });

    (id, style)
}

fn parse_toggler_style(e: &BytesStart) -> (String, TogglerStyle) {
    let id = parse_string_attr(e, b"id").expect("<toggler-style> requires an 'id' attribute");
    let style = TogglerStyle {
        background_color: parse_color_attr(e, b"background-color"),
        background_border_width: parse_f32_attr(e, b"background-border-width"),
        background_border_color: parse_color_attr(e, b"background-border-color"),
        foreground_color: parse_color_attr(e, b"foreground-color"),
        foreground_border_width: parse_f32_attr(e, b"foreground-border-width"),
        foreground_border_color: parse_color_attr(e, b"foreground-border-color"),
        text_color: parse_color_attr(e, b"text-color"),
        border_radius: parse_border_radius(e),
        padding_ratio: parse_f32_attr(e, b"padding-ratio"),
    };
    (id, style)
}

fn parse_font_def(e: &BytesStart) -> (String, FontDef) {
    let id = parse_string_attr(e, b"id").expect("<font> requires an 'id' attribute");
    let def = FontDef {
        family: parse_string_attr(e, b"family"),
        weight: parse_font_weight_attr(e, b"weight"),
        stretch: parse_font_stretch_attr(e, b"stretch"),
        style: parse_font_style_attr(e, b"style"),
    };
    (id, def)
}

fn parse_text_input_style_empty(e: &BytesStart) -> (String, TextInputStyle) {
    let id =
        parse_string_attr(e, b"id").expect("<text-input-style> requires an 'id' attribute");
    let base = parse_text_input_style_fields(e);
    (id, TextInputStyle { base, ..Default::default() })
}

pub fn parse_styles(reader: &mut Reader<&[u8]>) -> ParsedStyles {
    let mut styles = ParsedStyles::default();

    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => {
                let tag = e.name().as_ref().to_vec();
                match tag.as_slice() {
                    b"container-style" => {
                        styles.container.push(parse_container_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"text-style" => {
                        styles.text.push(parse_text_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"button-style" => {
                        styles.button.push(parse_button_style(&e, reader));
                    }
                    b"checkbox-style" => {
                        styles.checkbox.push(parse_checkbox_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"text-input-style" => {
                        styles.text_input.push(parse_text_input_style(&e, reader));
                    }
                    b"toggler-style" => {
                        styles.toggler.push(parse_toggler_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"font" => {
                        styles.font.push(parse_font_def(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    other => panic!(
                        "unexpected element in <styles>: {}",
                        String::from_utf8_lossy(other)
                    ),
                }
            }
            Event::Empty(e) => match e.name().as_ref() {
                b"container-style" => styles.container.push(parse_container_style(&e)),
                b"text-style" => styles.text.push(parse_text_style(&e)),
                b"button-style" => styles.button.push(parse_button_style_empty(&e)),
                b"checkbox-style" => styles.checkbox.push(parse_checkbox_style(&e)),
                b"text-input-style" => styles.text_input.push(parse_text_input_style_empty(&e)),
                b"toggler-style" => styles.toggler.push(parse_toggler_style(&e)),
                b"font" => styles.font.push(parse_font_def(&e)),
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

    styles
}
