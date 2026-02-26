use iced_layout_core::{Node, PickListHandle};
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::attr::*;

fn parse_text_content(reader: &mut Reader<&[u8]>, closing_tag: &[u8]) -> String {
    let mut content = String::new();
    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Text(t) => {
                content.push_str(&t.unescape().expect("failed to unescape text"));
            }
            Event::End(end) if end.name().as_ref() == closing_tag => break,
            _ => {}
        }
    }
    content
}

fn parse_children(reader: &mut Reader<&[u8]>) -> Vec<Node> {
    let mut children = Vec::new();
    loop {
        let child = parse_node(reader);
        match child {
            Node::Text { ref content, .. } if content.is_empty() => break,
            _ => children.push(child),
        }
    }
    children
}

pub fn parse_node(reader: &mut Reader<&[u8]>) -> Node {
    loop {
        let (e, has_closing_tag) = match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => (e.into_owned(), true),
            Event::Empty(e) => (e.into_owned(), false),
            Event::End(_) => {
                return Node::Text {
                    content: String::new(),
                    style: None,
                    attrs: Default::default(),
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
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
                Node::Container { id, style, padding, children }
            }
            b"text" => {
                let style = parse_string_attr(&e, b"style");
                let attrs = parse_text_attrs(&e);
                let content = if has_closing_tag {
                    parse_text_content(reader, b"text")
                } else {
                    String::new()
                };
                Node::Text { content, style, attrs }
            }
            b"row" => {
                let spacing = parse_f32_attr(&e, b"spacing");
                let padding = parse_padding(&e);
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let align_y = parse_vertical_attr(&e, b"align-y");
                let clip = parse_bool_attr(&e, b"clip");
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
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
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
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
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
                Node::Button { style, padding, width, height, clip, on_press, on_press_with, on_press_maybe, children }
            }
            b"stack" => {
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let clip = parse_bool_attr(&e, b"clip");
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
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
                let font = parse_string_attr(&e, b"font");
                let icon = parse_string_attr(&e, b"icon");
                if has_closing_tag {
                    consume_closing_tag(reader, b"text-input");
                }
                Node::TextInput {
                    placeholder, value, id, secure, on_input,
                    on_submit, on_submit_maybe, on_paste,
                    width, padding, size, line_height, align_x, style, font, icon,
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
                let font = parse_string_attr(&e, b"font");
                let icon = parse_string_attr(&e, b"icon");

                let label = if has_closing_tag {
                    parse_text_content(reader, b"checkbox")
                } else {
                    String::new()
                };
                Node::Checkbox {
                    label, is_checked, on_toggle, on_toggle_maybe,
                    size, width, spacing, text_size, text_line_height,
                    text_shaping, text_wrapping, style, font, icon,
                }
            }
            b"foreach" => {
                let iterable = parse_string_attr(&e, b"iterable")
                    .expect("<foreach> requires an 'iterable' attribute");
                assert!(has_closing_tag, "<foreach> must have a closing tag");
                let child = parse_node(reader);
                assert!(
                    !matches!(child, Node::Text { ref content, .. } if content.is_empty()),
                    "<foreach> must contain exactly 1 child element"
                );
                consume_closing_tag(reader, b"foreach");
                Node::ForEach { iterable, body: Box::new(child) }
            }
            b"if" => {
                let condition = parse_string_attr(&e, b"condition")
                    .expect("<if> requires a 'condition' attribute");
                assert!(has_closing_tag, "<if> must have a closing tag");

                let mut true_branch: Option<Node> = None;
                let mut false_branch: Option<Node> = None;

                loop {
                    match reader.read_event().expect("failed to read XML") {
                        Event::Start(inner) => {
                            let inner_tag = inner.name().as_ref().to_vec();
                            match inner_tag.as_slice() {
                                b"true" => {
                                    assert!(true_branch.is_none(), "<if> must have exactly one <true> element");
                                    let child = parse_node(reader);
                                    assert!(
                                        !matches!(child, Node::Text { ref content, .. } if content.is_empty()),
                                        "<true> must contain exactly 1 child element"
                                    );
                                    consume_closing_tag(reader, b"true");
                                    true_branch = Some(child);
                                }
                                b"false" => {
                                    assert!(false_branch.is_none(), "<if> must have at most one <false> element");
                                    let child = parse_node(reader);
                                    assert!(
                                        !matches!(child, Node::Text { ref content, .. } if content.is_empty()),
                                        "<false> must contain exactly 1 child element"
                                    );
                                    consume_closing_tag(reader, b"false");
                                    false_branch = Some(child);
                                }
                                other => panic!(
                                    "<if> may only contain <true> and <false>, found <{}>",
                                    String::from_utf8_lossy(other)
                                ),
                            }
                        }
                        Event::End(end) if end.name().as_ref() == b"if" => break,
                        Event::Text(_) | Event::Comment(_) => continue,
                        _ => continue,
                    }
                }

                let true_branch = true_branch.expect("<if> must contain a <true> element");
                Node::If {
                    condition,
                    true_branch: Box::new(true_branch),
                    false_branch: false_branch.map(Box::new),
                }
            }
            b"vertical-slider" => {
                let range_start = parse_f32_attr(&e, b"range-start")
                    .expect("<vertical-slider> requires a 'range-start' attribute");
                let range_end = parse_f32_attr(&e, b"range-end")
                    .expect("<vertical-slider> requires a 'range-end' attribute");
                let value = parse_string_attr(&e, b"value")
                    .expect("<vertical-slider> requires a 'value' attribute");
                let on_change = parse_string_attr(&e, b"on-change")
                    .expect("<vertical-slider> requires an 'on-change' attribute");
                let default = parse_f32_attr(&e, b"default");
                let on_release = parse_string_attr(&e, b"on-release");
                let width = parse_f32_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let step = parse_string_attr(&e, b"step");
                let shift_step = parse_f32_attr(&e, b"shift-step");
                let style = parse_string_attr(&e, b"style");
                if has_closing_tag {
                    consume_closing_tag(reader, b"vertical-slider");
                }
                Node::VerticalSlider { range_start, range_end, value, on_change, default, on_release, width, height, step, shift_step, style }
            }
            b"slider" => {
                let range_start = parse_f32_attr(&e, b"range-start")
                    .expect("<slider> requires a 'range-start' attribute");
                let range_end = parse_f32_attr(&e, b"range-end")
                    .expect("<slider> requires a 'range-end' attribute");
                let value = parse_string_attr(&e, b"value")
                    .expect("<slider> requires a 'value' attribute");
                let on_change = parse_string_attr(&e, b"on-change")
                    .expect("<slider> requires an 'on-change' attribute");
                let default = parse_f32_attr(&e, b"default");
                let on_release = parse_string_attr(&e, b"on-release");
                let width = parse_length_attr(&e, b"width");
                let height = parse_f32_attr(&e, b"height");
                let step = parse_f32_attr(&e, b"step");
                let shift_step = parse_f32_attr(&e, b"shift-step");
                let style = parse_string_attr(&e, b"style");
                if has_closing_tag {
                    consume_closing_tag(reader, b"slider");
                }
                Node::Slider { range_start, range_end, value, on_change, default, on_release, width, height, step, shift_step, style }
            }
            b"tooltip" => {
                let position = parse_tooltip_position_attr(&e, b"position")
                    .expect("<tooltip> requires a 'position' attribute");
                let gap = parse_f32_attr(&e, b"gap");
                let padding = parse_f32_attr(&e, b"padding");
                let delay = parse_delay_attr(&e, b"delay");
                let snap_within_viewport = parse_bool_attr(&e, b"snap-within-viewport");
                let style = parse_string_attr(&e, b"style");
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
                assert_eq!(children.len(), 2, "<tooltip> must have exactly 2 children (content, tooltip)");
                Node::Tooltip { position, gap, padding, delay, snap_within_viewport, style, children }
            }
            b"toggler" => {
                let is_toggled = parse_string_attr(&e, b"is-toggled")
                    .expect("<toggler> requires an 'is-toggled' attribute");
                let label = parse_string_attr(&e, b"label");
                let on_toggle = parse_string_attr(&e, b"on-toggle");
                let on_toggle_maybe = parse_string_attr(&e, b"on-toggle-maybe");
                let size = parse_f32_attr(&e, b"size");
                let width = parse_length_attr(&e, b"width");
                let text_size = parse_f32_attr(&e, b"text-size");
                let text_line_height = parse_line_height_attr(&e, b"text-line-height");
                let text_alignment = parse_text_alignment_attr(&e, b"text-alignment");
                let text_shaping = parse_shaping_attr(&e, b"text-shaping");
                let text_wrapping = parse_wrapping_attr(&e, b"text-wrapping");
                let spacing = parse_f32_attr(&e, b"spacing");
                let font = parse_string_attr(&e, b"font");
                let style = parse_string_attr(&e, b"style");
                if has_closing_tag {
                    consume_closing_tag(reader, b"toggler");
                }
                Node::Toggler {
                    is_toggled, label, on_toggle, on_toggle_maybe,
                    size, width, text_size, text_line_height,
                    text_alignment, text_shaping, text_wrapping,
                    spacing, font, style,
                }
            }
            b"combo-box" => {
                let state = parse_string_attr(&e, b"state")
                    .expect("<combo-box> requires a 'state' attribute");
                let placeholder = parse_string_attr(&e, b"placeholder")
                    .expect("<combo-box> requires a 'placeholder' attribute");
                let selection = parse_string_attr(&e, b"selection")
                    .expect("<combo-box> requires a 'selection' attribute");
                let on_selected = parse_string_attr(&e, b"on-selected")
                    .expect("<combo-box> requires an 'on-selected' attribute");
                let on_input = parse_string_attr(&e, b"on-input");
                let on_option_hovered = parse_string_attr(&e, b"on-option-hovered");
                let on_open = parse_string_attr(&e, b"on-open");
                let on_close = parse_string_attr(&e, b"on-close");
                let padding = parse_padding(&e);
                let font = parse_string_attr(&e, b"font");
                let size = parse_f32_attr(&e, b"size");
                let line_height = parse_line_height_attr(&e, b"line-height");
                let width = parse_length_attr(&e, b"width");
                let menu_height = parse_length_attr(&e, b"menu-height");
                let text_shaping = parse_shaping_attr(&e, b"text-shaping");
                let input_style = parse_string_attr(&e, b"input-style");
                let menu_style = parse_string_attr(&e, b"menu-style");
                let icon = parse_string_attr(&e, b"icon");
                if has_closing_tag {
                    consume_closing_tag(reader, b"combo-box");
                }
                Node::ComboBox {
                    state, placeholder, selection, on_selected,
                    on_input, on_option_hovered, on_open, on_close,
                    padding, font, size, line_height, width, menu_height,
                    text_shaping, input_style, menu_style, icon,
                }
            }
            b"text-editor" => {
                let content = parse_string_attr(&e, b"content")
                    .expect("<text-editor> requires a 'content' attribute");
                let id = parse_string_attr(&e, b"id");
                let placeholder = parse_string_attr(&e, b"placeholder");
                let width = parse_f32_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let min_height = parse_f32_attr(&e, b"min-height");
                let max_height = parse_f32_attr(&e, b"max-height");
                let on_action = parse_string_attr(&e, b"on-action");
                let font = parse_string_attr(&e, b"font");
                let size = parse_f32_attr(&e, b"size");
                let line_height = parse_line_height_attr(&e, b"line-height");
                let padding = parse_padding(&e);
                let wrapping = parse_wrapping_attr(&e, b"wrapping");
                let key_binding = parse_string_attr(&e, b"key-binding");
                let style = parse_string_attr(&e, b"style");
                if has_closing_tag {
                    consume_closing_tag(reader, b"text-editor");
                }
                Node::TextEditor {
                    content, id, placeholder, width, height,
                    min_height, max_height, on_action, font, size,
                    line_height, padding, wrapping, key_binding, style,
                }
            }
            b"sensor" => {
                let on_show = parse_string_attr(&e, b"on-show");
                let on_resize = parse_string_attr(&e, b"on-resize");
                let on_hide = parse_string_attr(&e, b"on-hide");
                let anticipate = parse_f32_attr(&e, b"anticipate");
                let delay = parse_delay_attr(&e, b"delay");
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
                Node::Sensor { on_show, on_resize, on_hide, anticipate, delay, children }
            }
            b"float" => {
                let scale = parse_f32_attr(&e, b"scale");
                let translate = parse_string_attr(&e, b"translate");
                let style = parse_string_attr(&e, b"style");
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
                Node::Float { scale, translate, style, children }
            }
            b"mouse-area" => {
                let on_press = parse_string_attr(&e, b"on-press");
                let on_release = parse_string_attr(&e, b"on-release");
                let on_double_click = parse_string_attr(&e, b"on-double-click");
                let on_right_press = parse_string_attr(&e, b"on-right-press");
                let on_right_release = parse_string_attr(&e, b"on-right-release");
                let on_middle_press = parse_string_attr(&e, b"on-middle-press");
                let on_middle_release = parse_string_attr(&e, b"on-middle-release");
                let on_scroll = parse_string_attr(&e, b"on-scroll");
                let on_enter = parse_string_attr(&e, b"on-enter");
                let on_move = parse_string_attr(&e, b"on-move");
                let on_exit = parse_string_attr(&e, b"on-exit");
                let interaction = parse_interaction_attr(&e, b"interaction");
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
                Node::MouseArea {
                    on_press, on_release, on_double_click,
                    on_right_press, on_right_release,
                    on_middle_press, on_middle_release,
                    on_scroll, on_enter, on_move, on_exit,
                    interaction, children,
                }
            }
            b"pick-list" => {
                let options = parse_string_attr(&e, b"options")
                    .expect("<pick-list> requires an 'options' attribute");
                let selected = parse_string_attr(&e, b"selected")
                    .expect("<pick-list> requires a 'selected' attribute");
                let on_select = parse_string_attr(&e, b"on-select")
                    .expect("<pick-list> requires an 'on-select' attribute");
                let placeholder = parse_string_attr(&e, b"placeholder");
                let width = parse_length_attr(&e, b"width");
                let menu_height = parse_length_attr(&e, b"menu-height");
                let padding = parse_padding(&e);
                let text_size = parse_f32_attr(&e, b"text-size");
                let text_line_height = parse_line_height_attr(&e, b"text-line-height");
                let text_shaping = parse_shaping_attr(&e, b"text-shaping");
                let font = parse_string_attr(&e, b"font");
                let on_open = parse_string_attr(&e, b"on-open");
                let on_close = parse_string_attr(&e, b"on-close");
                let style = parse_string_attr(&e, b"style");
                let menu_style = parse_string_attr(&e, b"menu-style");

                let handle_type = parse_string_attr(&e, b"handle");
                let handle_arrow_size = parse_f32_attr(&e, b"handle-arrow-size");
                let handle_static = parse_string_attr(&e, b"handle-static-value");
                let handle_dynamic_closed = parse_string_attr(&e, b"handle-dynamic-closed");
                let handle_dynamic_open = parse_string_attr(&e, b"handle-dynamic-open");
                let handle = match handle_type.as_deref() {
                    None => None,
                    Some("arrow") => Some(PickListHandle::Arrow { size: handle_arrow_size }),
                    Some("none") => Some(PickListHandle::None),
                    Some("static") => Some(PickListHandle::Static {
                        icon: handle_static
                            .expect("<pick-list handle=\"static\"> requires 'handle-static-value'"),
                    }),
                    Some("dynamic") => Some(PickListHandle::Dynamic {
                        closed: handle_dynamic_closed
                            .expect("<pick-list handle=\"dynamic\"> requires 'handle-dynamic-closed'"),
                        open: handle_dynamic_open
                            .expect("<pick-list handle=\"dynamic\"> requires 'handle-dynamic-open'"),
                    }),
                    Some(other) => panic!("invalid pick-list handle type: {}", other),
                };

                if has_closing_tag {
                    consume_closing_tag(reader, b"pick-list");
                }
                Node::PickList {
                    options, selected, on_select, placeholder, width, menu_height,
                    padding, text_size, text_line_height, text_shaping, font,
                    handle, on_open, on_close, style, menu_style,
                }
            }
            b"radio" => {
                let value = parse_string_attr(&e, b"value")
                    .expect("<radio> requires a 'value' attribute");
                let selected = parse_string_attr(&e, b"selected")
                    .expect("<radio> requires a 'selected' attribute");
                let on_select = parse_string_attr(&e, b"on-select")
                    .expect("<radio> requires an 'on-select' attribute");
                let size = parse_f32_attr(&e, b"size");
                let width = parse_length_attr(&e, b"width");
                let spacing = parse_f32_attr(&e, b"spacing");
                let text_size = parse_f32_attr(&e, b"text-size");
                let text_line_height = parse_line_height_attr(&e, b"text-line-height");
                let text_shaping = parse_shaping_attr(&e, b"text-shaping");
                let text_wrapping = parse_wrapping_attr(&e, b"text-wrapping");
                let font = parse_string_attr(&e, b"font");
                let style = parse_string_attr(&e, b"style");

                let label = if has_closing_tag {
                    parse_text_content(reader, b"radio")
                } else {
                    String::new()
                };
                Node::Radio {
                    label, value, selected, on_select,
                    size, width, spacing, text_size, text_line_height,
                    text_shaping, text_wrapping, font, style,
                }
            }
            b"responsive" => {
                let view = parse_string_attr(&e, b"view")
                    .expect("<responsive> requires a 'view' attribute");
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                if has_closing_tag {
                    consume_closing_tag(reader, b"responsive");
                }
                Node::Responsive { view, width, height }
            }
            b"rule-horizontal" => {
                let thickness = parse_f32_attr(&e, b"height")
                    .expect("<rule-horizontal> requires a 'height' attribute");
                let style = parse_string_attr(&e, b"style");
                if has_closing_tag {
                    consume_closing_tag(reader, b"rule-horizontal");
                }
                Node::RuleHorizontal { thickness, style }
            }
            b"rule-vertical" => {
                let thickness = parse_f32_attr(&e, b"width")
                    .expect("<rule-vertical> requires a 'width' attribute");
                let style = parse_string_attr(&e, b"style");
                if has_closing_tag {
                    consume_closing_tag(reader, b"rule-vertical");
                }
                Node::RuleVertical { thickness, style }
            }
            b"progress-bar" => {
                let range_start = parse_f32_attr(&e, b"range-start")
                    .expect("<progress-bar> requires a 'range-start' attribute");
                let range_end = parse_f32_attr(&e, b"range-end")
                    .expect("<progress-bar> requires a 'range-end' attribute");
                let value = parse_string_attr(&e, b"value")
                    .expect("<progress-bar> requires a 'value' attribute");
                let length = parse_length_attr(&e, b"length");
                let girth = parse_length_attr(&e, b"girth");
                let style = parse_string_attr(&e, b"style");
                if has_closing_tag {
                    consume_closing_tag(reader, b"progress-bar");
                }
                Node::ProgressBar { range_start, range_end, value, length, girth, style }
            }
            b"pin" => {
                let width = parse_length_attr(&e, b"width");
                let height = parse_length_attr(&e, b"height");
                let x = parse_f32_attr(&e, b"x");
                let y = parse_f32_attr(&e, b"y");
                let children = if has_closing_tag { parse_children(reader) } else { Vec::new() };
                Node::Pin { width, height, x, y, children }
            }
            b"widget" => {
                let method = parse_string_attr(&e, b"method")
                    .expect("<widget> requires a 'method' attribute");
                let mut indexed_args: Vec<(u8, String)> = Vec::new();
                for i in 0..10u8 {
                    let attr_name = format!("arg-{}", i);
                    if let Some(val) = parse_string_attr(&e, attr_name.as_bytes()) {
                        indexed_args.push((i, val));
                    }
                }
                indexed_args.sort_by_key(|(i, _)| *i);
                let args: Vec<String> = indexed_args.into_iter().map(|(_, v)| v).collect();
                let child = if has_closing_tag {
                    let c = parse_node(reader);
                    if matches!(c, Node::Text { ref content, .. } if content.is_empty()) {
                        None
                    } else {
                        consume_closing_tag(reader, b"widget");
                        Some(Box::new(c))
                    }
                } else {
                    None
                };
                Node::Widget { method, args, child }
            }
            other => panic!(
                "unsupported tag: {}",
                String::from_utf8_lossy(other)
            ),
        };
    }
}
