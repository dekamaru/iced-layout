use iced_layout_core::{
    BorderRadius, Color, Horizontal, Length, LineHeight, Padding, Shaping, TextAlignment, TextAttrs,
    Vertical, Wrapping,
};
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn parse_f32_attr(e: &BytesStart, name: &[u8]) -> Option<f32> {
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

pub fn parse_string_attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

pub fn parse_length_attr(e: &BytesStart, name: &[u8]) -> Option<Length> {
    parse_string_attr(e, name).map(|s| parse_length(&s))
}

pub fn parse_length(s: &str) -> Length {
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

pub fn color_from_rgba_bytes(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
}

pub fn parse_hex_color(s: &str) -> Color {
    let hex = s
        .strip_prefix('#')
        .unwrap_or_else(|| panic!("color must start with #, got \"{s}\""));
    let chars: Vec<char> = hex.chars().collect();
    match chars.len() {
        3 => {
            let r = expand_nibble(parse_nibble(chars[0]));
            let g = expand_nibble(parse_nibble(chars[1]));
            let b = expand_nibble(parse_nibble(chars[2]));
            color_from_rgba_bytes(r, g, b, 255)
        }
        4 => {
            let r = expand_nibble(parse_nibble(chars[0]));
            let g = expand_nibble(parse_nibble(chars[1]));
            let b = expand_nibble(parse_nibble(chars[2]));
            let a = expand_nibble(parse_nibble(chars[3]));
            color_from_rgba_bytes(r, g, b, a)
        }
        6 => {
            let r = parse_byte(chars[0], chars[1]);
            let g = parse_byte(chars[2], chars[3]);
            let b = parse_byte(chars[4], chars[5]);
            color_from_rgba_bytes(r, g, b, 255)
        }
        8 => {
            let r = parse_byte(chars[0], chars[1]);
            let g = parse_byte(chars[2], chars[3]);
            let b = parse_byte(chars[4], chars[5]);
            let a = parse_byte(chars[6], chars[7]);
            color_from_rgba_bytes(r, g, b, a)
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

pub fn parse_color_attr(e: &BytesStart, name: &[u8]) -> Option<Color> {
    parse_string_attr(e, name).map(|s| parse_hex_color(&s))
}

pub fn parse_bool_attr(e: &BytesStart, name: &[u8]) -> Option<bool> {
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

pub fn parse_horizontal_attr(e: &BytesStart, name: &[u8]) -> Option<Horizontal> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "left" => Horizontal::Left,
        "center" => Horizontal::Center,
        "right" => Horizontal::Right,
        _ => panic!("invalid horizontal alignment: {}", s),
    })
}

pub fn parse_vertical_attr(e: &BytesStart, name: &[u8]) -> Option<Vertical> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "top" => Vertical::Top,
        "center" => Vertical::Center,
        "bottom" => Vertical::Bottom,
        _ => panic!("invalid vertical alignment: {}", s),
    })
}

pub fn parse_padding(e: &BytesStart) -> Padding {
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

pub fn parse_line_height_attr(e: &BytesStart, name: &[u8]) -> Option<LineHeight> {
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

pub fn parse_text_alignment_attr(e: &BytesStart, name: &[u8]) -> Option<TextAlignment> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "default" => TextAlignment::Default,
        "left" => TextAlignment::Left,
        "center" => TextAlignment::Center,
        "right" => TextAlignment::Right,
        "justified" => TextAlignment::Justified,
        _ => panic!("invalid text alignment: {}", s),
    })
}

pub fn parse_shaping_attr(e: &BytesStart, name: &[u8]) -> Option<Shaping> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "auto" => Shaping::Auto,
        "basic" => Shaping::Basic,
        "advanced" => Shaping::Advanced,
        _ => panic!("invalid shaping: {}", s),
    })
}

pub fn parse_wrapping_attr(e: &BytesStart, name: &[u8]) -> Option<Wrapping> {
    parse_string_attr(e, name).map(|s| match s.as_str() {
        "none" => Wrapping::None,
        "word" => Wrapping::Word,
        "glyph" => Wrapping::Glyph,
        "word-or-glyph" => Wrapping::WordOrGlyph,
        _ => panic!("invalid wrapping: {}", s),
    })
}

pub fn parse_text_attrs(e: &BytesStart) -> TextAttrs {
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

pub fn parse_border_radius(e: &BytesStart) -> BorderRadius {
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

pub fn consume_closing_tag(reader: &mut Reader<&[u8]>, tag: &[u8]) {
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
