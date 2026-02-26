use iced_layout_core::{
    BorderRadius, ButtonStyle, ButtonStyleFields, CheckboxIcon, CheckboxStyle, ContainerStyle,
    FloatStyle, FontDef, OverlayMenuStyle, PickListIcon, PickListStyle, PickListStyleFields,
    ProgressBarStyle, RadioStyle, RuleFillMode, RuleStyle, TextEditorStyle, TextEditorStyleFields,
    TextInputIcon, TextInputStyle, TextInputStyleFields, TextStyle, TogglerStyle,
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
    pub text_editor: Vec<(String, TextEditorStyle)>,
    pub overlay_menu: Vec<(String, OverlayMenuStyle)>,
    pub float: Vec<(String, FloatStyle)>,
    pub pick_list: Vec<(String, PickListStyle)>,
    pub progress_bar: Vec<(String, ProgressBarStyle)>,
    pub radio: Vec<(String, RadioStyle)>,
    pub rule: Vec<(String, RuleStyle)>,
    pub font: Vec<(String, FontDef)>,
    pub checkbox_icons: Vec<(String, CheckboxIcon)>,
    pub text_input_icons: Vec<(String, TextInputIcon)>,
    pub pick_list_icons: Vec<(String, PickListIcon)>,
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
            text_editor: Vec::new(),
            overlay_menu: Vec::new(),
            float: Vec::new(),
            pick_list: Vec::new(),
            progress_bar: Vec::new(),
            radio: Vec::new(),
            rule: Vec::new(),
            font: Vec::new(),
            checkbox_icons: Vec::new(),
            text_input_icons: Vec::new(),
            pick_list_icons: Vec::new(),
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

fn parse_text_editor_style_fields(e: &BytesStart) -> TextEditorStyleFields {
    TextEditorStyleFields {
        background_color: parse_color_attr(e, b"background-color"),
        border_color: parse_color_attr(e, b"border-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_radius: parse_border_radius(e),
        placeholder_color: parse_color_attr(e, b"placeholder"),
        value_color: parse_color_attr(e, b"value"),
        selection_color: parse_color_attr(e, b"selection"),
    }
}

fn assign_text_editor_status_fields(
    style: &mut TextEditorStyle,
    tag: &[u8],
    fields: TextEditorStyleFields,
) {
    match tag {
        b"active" => style.active = Some(fields),
        b"hovered" => style.hovered = Some(fields),
        b"disabled" => style.disabled = Some(fields),
        other => panic!(
            "unexpected element in <text-editor-style>: {}",
            String::from_utf8_lossy(other)
        ),
    }
}

fn parse_text_editor_style(
    e: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> (String, TextEditorStyle) {
    let id =
        parse_string_attr(e, b"id").expect("<text-editor-style> requires an 'id' attribute");
    let base = parse_text_editor_style_fields(e);
    let mut style = TextEditorStyle {
        base,
        ..Default::default()
    };

    parse_stateful_children(reader, b"text-editor-style", &mut style, |s, tag, e| {
        assign_text_editor_status_fields(s, tag, parse_text_editor_style_fields(e))
    });

    (id, style)
}

fn parse_text_editor_style_empty(e: &BytesStart) -> (String, TextEditorStyle) {
    let id =
        parse_string_attr(e, b"id").expect("<text-editor-style> requires an 'id' attribute");
    let base = parse_text_editor_style_fields(e);
    (id, TextEditorStyle { base, ..Default::default() })
}

fn parse_overlay_menu_style(e: &BytesStart) -> (String, OverlayMenuStyle) {
    let id =
        parse_string_attr(e, b"id").expect("<overlay-menu-style> requires an 'id' attribute");
    let style = OverlayMenuStyle {
        background_color: parse_color_attr(e, b"background-color"),
        border_color: parse_color_attr(e, b"border-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_radius: parse_border_radius(e),
        text_color: parse_color_attr(e, b"text-color"),
        selected_text_color: parse_color_attr(e, b"selected-text-color"),
        selected_background_color: parse_color_attr(e, b"selected-background-color"),
        shadow_color: parse_color_attr(e, b"shadow-color"),
        shadow_offset_x: parse_f32_attr(e, b"shadow-offset-x"),
        shadow_offset_y: parse_f32_attr(e, b"shadow-offset-y"),
        shadow_blur_radius: parse_f32_attr(e, b"shadow-blur-radius"),
    };
    (id, style)
}

fn parse_float_style(e: &BytesStart) -> (String, FloatStyle) {
    let id = parse_string_attr(e, b"id").expect("<float-style> requires an 'id' attribute");
    let style = FloatStyle {
        shadow_color: parse_color_attr(e, b"shadow-color"),
        shadow_offset_x: parse_f32_attr(e, b"shadow-offset-x"),
        shadow_offset_y: parse_f32_attr(e, b"shadow-offset-y"),
        shadow_blur_radius: parse_f32_attr(e, b"shadow-blur-radius"),
        shadow_border_radius: parse_shadow_border_radius(e),
    };
    (id, style)
}

fn parse_progress_bar_style(e: &BytesStart) -> (String, ProgressBarStyle) {
    let id =
        parse_string_attr(e, b"id").expect("<progress-bar-style> requires an 'id' attribute");
    let style = ProgressBarStyle {
        background_color: parse_color_attr(e, b"background-color"),
        bar_color: parse_color_attr(e, b"bar-color"),
        border_color: parse_color_attr(e, b"border-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_radius: parse_border_radius(e),
    };
    (id, style)
}

fn parse_radio_style(e: &BytesStart) -> (String, RadioStyle) {
    let id = parse_string_attr(e, b"id").expect("<radio-style> requires an 'id' attribute");
    let style = RadioStyle {
        background_color: parse_color_attr(e, b"background-color"),
        dot_color: parse_color_attr(e, b"dot-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_color: parse_color_attr(e, b"border-color"),
        text_color: parse_color_attr(e, b"text-color"),
    };
    (id, style)
}

fn parse_rule_style(e: &BytesStart) -> (String, RuleStyle) {
    let id = parse_string_attr(e, b"id").expect("<rule-style> requires an 'id' attribute");

    // Radius: supports uniform `radius` or per-corner variants
    let radius = if let Some(all) = parse_f32_attr(e, b"radius") {
        BorderRadius {
            top_left: Some(all),
            top_right: Some(all),
            bottom_right: Some(all),
            bottom_left: Some(all),
        }
    } else {
        BorderRadius {
            top_left: parse_f32_attr(e, b"radius-top-left"),
            top_right: parse_f32_attr(e, b"radius-top-right"),
            bottom_right: parse_f32_attr(e, b"radius-bottom-right"),
            bottom_left: parse_f32_attr(e, b"radius-bottom-left"),
        }
    };

    // FillMode: at most one of these groups should be set
    let fill_mode = if let Some(pct) = parse_f32_attr(e, b"fill-mode-percent") {
        RuleFillMode::Percent(pct)
    } else if let Some(pad) = parse_u16_attr(e, b"fill-mode-padded") {
        RuleFillMode::Padded(pad)
    } else if let Some(p1) = parse_u16_attr(e, b"fill-mode-asymmetric-padding-value-1") {
        let p2 = parse_u16_attr(e, b"fill-mode-asymmetric-padding-value-2")
            .expect("<rule-style fill-mode-asymmetric-padding-value-1> requires 'fill-mode-asymmetric-padding-value-2'");
        RuleFillMode::AsymmetricPadding(p1, p2)
    } else {
        RuleFillMode::Full
    };

    let style = RuleStyle {
        color: parse_color_attr(e, b"color"),
        radius,
        fill_mode,
        snap: parse_bool_attr(e, b"snap"),
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

fn parse_checkbox_icon(e: &BytesStart) -> (String, CheckboxIcon) {
    let id = parse_string_attr(e, b"id").expect("<checkbox-icon> requires an 'id' attribute");
    let font = parse_string_attr(e, b"font").expect("<checkbox-icon> requires a 'font' attribute");
    let code_point = parse_code_point_attr(e, b"code-point")
        .expect("<checkbox-icon> requires a 'code-point' attribute");
    let icon = CheckboxIcon {
        font,
        code_point,
        size: parse_f32_attr(e, b"size"),
        line_height: parse_line_height_attr(e, b"line-height"),
        shaping: parse_shaping_attr(e, b"shaping"),
    };
    (id, icon)
}

fn parse_text_input_icon(e: &BytesStart) -> (String, TextInputIcon) {
    let id =
        parse_string_attr(e, b"id").expect("<text-input-icon> requires an 'id' attribute");
    let font =
        parse_string_attr(e, b"font").expect("<text-input-icon> requires a 'font' attribute");
    let code_point = parse_code_point_attr(e, b"code-point")
        .expect("<text-input-icon> requires a 'code-point' attribute");
    let icon = TextInputIcon {
        font,
        code_point,
        size: parse_f32_attr(e, b"size"),
        spacing: parse_f32_attr(e, b"spacing"),
        side: parse_text_input_side_attr(e, b"side"),
    };
    (id, icon)
}

fn parse_pick_list_icon(e: &BytesStart) -> (String, PickListIcon) {
    let id =
        parse_string_attr(e, b"id").expect("<pick-list-icon> requires an 'id' attribute");
    let font =
        parse_string_attr(e, b"font").expect("<pick-list-icon> requires a 'font' attribute");
    let code_point = parse_code_point_attr(e, b"code-point")
        .expect("<pick-list-icon> requires a 'code-point' attribute");
    let icon = PickListIcon {
        font,
        code_point,
        size: parse_f32_attr(e, b"size"),
        line_height: parse_line_height_attr(e, b"line-height"),
        shaping: parse_shaping_attr(e, b"shaping"),
    };
    (id, icon)
}

fn parse_pick_list_style_fields(e: &BytesStart) -> PickListStyleFields {
    PickListStyleFields {
        text_color: parse_color_attr(e, b"text-color"),
        placeholder_color: parse_color_attr(e, b"placeholder-color"),
        handle_color: parse_color_attr(e, b"handle-color"),
        background_color: parse_color_attr(e, b"background-color"),
        border_color: parse_color_attr(e, b"border-color"),
        border_width: parse_f32_attr(e, b"border-width"),
        border_radius: parse_border_radius(e),
    }
}

fn assign_pick_list_status_fields(
    style: &mut PickListStyle,
    tag: &[u8],
    fields: PickListStyleFields,
) {
    match tag {
        b"active" => style.active = Some(fields),
        b"hovered" => style.hovered = Some(fields),
        other => panic!(
            "unexpected element in <pick-list-style>: {}",
            String::from_utf8_lossy(other)
        ),
    }
}

fn parse_pick_list_style(e: &BytesStart, reader: &mut Reader<&[u8]>) -> (String, PickListStyle) {
    let id =
        parse_string_attr(e, b"id").expect("<pick-list-style> requires an 'id' attribute");
    let base = parse_pick_list_style_fields(e);
    let mut style = PickListStyle {
        base,
        ..Default::default()
    };
    parse_stateful_children(reader, b"pick-list-style", &mut style, |s, tag, e| {
        assign_pick_list_status_fields(s, tag, parse_pick_list_style_fields(e))
    });
    (id, style)
}

fn parse_pick_list_style_empty(e: &BytesStart) -> (String, PickListStyle) {
    let id =
        parse_string_attr(e, b"id").expect("<pick-list-style> requires an 'id' attribute");
    let base = parse_pick_list_style_fields(e);
    (id, PickListStyle { base, ..Default::default() })
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
                    b"text-editor-style" => {
                        styles.text_editor.push(parse_text_editor_style(&e, reader));
                    }
                    b"overlay-menu-style" => {
                        styles.overlay_menu.push(parse_overlay_menu_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"float-style" => {
                        styles.float.push(parse_float_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"font" => {
                        styles.font.push(parse_font_def(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"pick-list-style" => {
                        styles.pick_list.push(parse_pick_list_style(&e, reader));
                    }
                    b"progress-bar-style" => {
                        styles.progress_bar.push(parse_progress_bar_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"radio-style" => {
                        styles.radio.push(parse_radio_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"rule-style" => {
                        styles.rule.push(parse_rule_style(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"checkbox-icon" => {
                        styles.checkbox_icons.push(parse_checkbox_icon(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"text-input-icon" => {
                        styles.text_input_icons.push(parse_text_input_icon(&e));
                        consume_closing_tag(reader, &tag);
                    }
                    b"pick-list-icon" => {
                        styles.pick_list_icons.push(parse_pick_list_icon(&e));
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
                b"text-editor-style" => styles.text_editor.push(parse_text_editor_style_empty(&e)),
                b"overlay-menu-style" => styles.overlay_menu.push(parse_overlay_menu_style(&e)),
                b"float-style" => styles.float.push(parse_float_style(&e)),
                b"pick-list-style" => styles.pick_list.push(parse_pick_list_style_empty(&e)),
                b"progress-bar-style" => styles.progress_bar.push(parse_progress_bar_style(&e)),
                b"radio-style" => styles.radio.push(parse_radio_style(&e)),
                b"rule-style" => styles.rule.push(parse_rule_style(&e)),
                b"font" => styles.font.push(parse_font_def(&e)),
                b"checkbox-icon" => styles.checkbox_icons.push(parse_checkbox_icon(&e)),
                b"text-input-icon" => styles.text_input_icons.push(parse_text_input_icon(&e)),
                b"pick-list-icon" => styles.pick_list_icons.push(parse_pick_list_icon(&e)),
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
