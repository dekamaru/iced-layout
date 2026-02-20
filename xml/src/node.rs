use iced_layout_core::Node;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::attr::*;

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
                if has_closing_tag {
                    consume_closing_tag(reader, b"text-input");
                }
                Node::TextInput {
                    placeholder, value, id, secure, on_input,
                    on_submit, on_submit_maybe, on_paste,
                    width, padding, size, line_height, align_x, style, font,
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
                    text_shaping, text_wrapping, style, font,
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
                if has_closing_tag {
                    consume_closing_tag(reader, b"vertical-slider");
                }
                Node::VerticalSlider { range_start, range_end, value, on_change, default, on_release, width, height, step, shift_step }
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
