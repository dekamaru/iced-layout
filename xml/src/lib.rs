mod attr;
mod node;
mod style;

use iced_layout_core::Layout;
use quick_xml::events::Event;
use quick_xml::Reader;

use node::parse_node;
use style::{ParsedStyles, parse_styles};

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

    let mut styles = ParsedStyles::default();
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
