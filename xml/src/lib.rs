use iced_layout_core::{Color, Length, Node, Padding, TextAttrs};
use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::events::BytesStart;

fn parse_f32_attr(e: &BytesStart, name: &[u8]) -> Option<f32> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name)
        .map(|a| {
            String::from_utf8_lossy(&a.value)
                .parse::<f32>()
                .unwrap_or_else(|err| panic!("invalid f32 for {:?}: {}", String::from_utf8_lossy(name), err))
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
            let v: u16 = inner.parse().unwrap_or_else(|e| panic!("invalid fill-portion value: {e}"));
            Length::FillPortion(v)
        }
        _ if s.starts_with("fixed(") && s.ends_with(')') => {
            let inner = &s["fixed(".len()..s.len() - 1];
            let v: f32 = inner.parse().unwrap_or_else(|e| panic!("invalid fixed value: {e}"));
            Length::Fixed(v)
        }
        _ => {
            let v: f32 = s.parse().unwrap_or_else(|e| panic!("invalid length \"{s}\": {e}"));
            Length::Fixed(v)
        }
    }
}

fn parse_hex_color(s: &str) -> Color {
    let hex = s.strip_prefix('#').unwrap_or_else(|| panic!("color must start with #, got \"{s}\""));
    let chars: Vec<char> = hex.chars().collect();
    match chars.len() {
        // #rgb
        3 => {
            let r = expand_nibble(parse_nibble(chars[0]));
            let g = expand_nibble(parse_nibble(chars[1]));
            let b = expand_nibble(parse_nibble(chars[2]));
            Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
        }
        // #rgba
        4 => {
            let r = expand_nibble(parse_nibble(chars[0]));
            let g = expand_nibble(parse_nibble(chars[1]));
            let b = expand_nibble(parse_nibble(chars[2]));
            let a = expand_nibble(parse_nibble(chars[3]));
            Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 }
        }
        // #rrggbb
        6 => {
            let r = parse_byte(chars[0], chars[1]);
            let g = parse_byte(chars[2], chars[3]);
            let b = parse_byte(chars[4], chars[5]);
            Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
        }
        // #rrggbbaa
        8 => {
            let r = parse_byte(chars[0], chars[1]);
            let g = parse_byte(chars[2], chars[3]);
            let b = parse_byte(chars[4], chars[5]);
            let a = parse_byte(chars[6], chars[7]);
            Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 }
        }
        _ => panic!("invalid color format \"{s}\", expected #rgb, #rgba, #rrggbb, or #rrggbbaa"),
    }
}

fn parse_nibble(c: char) -> u8 {
    c.to_digit(16).unwrap_or_else(|| panic!("invalid hex digit '{c}'")) as u8
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

fn parse_text_attrs(e: &BytesStart) -> TextAttrs {
    TextAttrs {
        size: parse_f32_attr(e, b"size"),
        width: parse_length_attr(e, b"width"),
        height: parse_length_attr(e, b"height"),
        color: parse_color_attr(e, b"color"),
    }
}

pub fn parse(xml: &str) -> Node {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) if e.name().as_ref() == b"layout" => break,
            Event::Decl(_) | Event::Text(_) | Event::Comment(_) => continue,
            other => panic!("expected <layout> root element, found {:?}", other),
        }
    }
    let root = parse_node(&mut reader);
    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::End(e) if e.name().as_ref() == b"layout" => break,
            Event::Text(_) | Event::Comment(_) => continue,
            other => panic!("expected </layout>, found {:?}", other),
        }
    }
    root
}

fn parse_node(reader: &mut Reader<&[u8]>) -> Node {
    loop {
        match reader.read_event().expect("failed to read XML") {
            Event::Start(e) => match e.name().as_ref() {
                b"container" => {
                    let id = parse_string_attr(&e, b"id");
                    let padding = parse_padding(&e);

                    let mut children = Vec::new();
                    loop {
                        let child = parse_node(reader);
                        match child {
                            Node::Text { ref content, .. } if content.is_empty() => break,
                            _ => children.push(child),
                        }
                    }
                    return Node::Container { id, padding, children };
                }
                b"text" => {
                    let attrs = parse_text_attrs(&e);
                    let mut content = String::new();
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
                    return Node::Text { content, attrs };
                }
                other => panic!(
                    "unsupported tag: {}",
                    String::from_utf8_lossy(other)
                ),
            },
            Event::End(_) => return Node::Text { content: String::new(), attrs: TextAttrs::default() },
            Event::Text(_) | Event::Comment(_) | Event::Decl(_) => continue,
            Event::Eof => panic!("unexpected end of XML"),
            _ => continue,
        }
    }
}
