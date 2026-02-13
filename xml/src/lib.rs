use iced_layout_core::Node;
use quick_xml::Reader;
use quick_xml::events::Event;

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
                    let id = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"id")
                        .map(|a| String::from_utf8_lossy(&a.value).into_owned());

                    let mut children = Vec::new();
                    loop {
                        let child = parse_node(reader);
                        match child {
                            Node::Text(ref s) if s.is_empty() => break,
                            _ => children.push(child),
                        }
                    }
                    return Node::Container { id, children };
                }
                b"text" => {
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
                    return Node::Text(content);
                }
                other => panic!(
                    "unsupported tag: {}",
                    String::from_utf8_lossy(other)
                ),
            },
            Event::End(_) => return Node::Text(String::new()),
            Event::Text(_) | Event::Comment(_) | Event::Decl(_) => continue,
            Event::Eof => panic!("unexpected end of XML"),
            _ => continue,
        }
    }
}
