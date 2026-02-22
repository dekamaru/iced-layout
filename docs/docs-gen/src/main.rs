use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Discovers widget and style element names from the XSD by reading the allowed
/// children of `<root>` and `<styles>`, preserving their declaration order.
fn discover_element_lists(content: &str) -> (Vec<String>, Vec<String>) {
    let mut reader = Reader::from_str(content);
    let mut depth: i32 = 0;
    let mut current_element: Option<String> = None;
    let mut in_root_choice = false;
    let mut in_styles_choice = false;
    let mut widgets: Vec<String> = Vec::new();
    let mut styles: Vec<String> = Vec::new();

    loop {
        match reader.read_event().expect("XSD parse error") {
            Event::Start(e) => {
                depth += 1;
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if local == "element" && depth == 2 {
                    current_element = get_attr(&e, b"name");
                } else if local == "choice" && depth == 4 {
                    match current_element.as_deref() {
                        Some("root") => in_root_choice = true,
                        Some("styles") => in_styles_choice = true,
                        _ => {}
                    }
                }
            }
            Event::Empty(e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if local == "element" && depth == 4 {
                    if let Some(ref_name) = get_attr(&e, b"ref") {
                        if in_root_choice && !widgets.contains(&ref_name) {
                            widgets.push(ref_name);
                        } else if in_styles_choice && !styles.contains(&ref_name) {
                            styles.push(ref_name);
                        }
                    }
                }
            }
            Event::End(e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if local == "choice" && depth == 4 {
                    in_root_choice = false;
                    in_styles_choice = false;
                } else if local == "element" && depth == 2 {
                    current_element = None;
                }
                depth -= 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }

    (widgets, styles)
}

fn fallback_description(name: &str) -> &'static str {
    match name {
        "if" => "Conditionally renders one of two child widgets based on a boolean condition.",
        "foreach" => "Iterates over a collection and renders a child widget for each item.",
        "widget" => "Renders a custom widget by calling a user-defined method.",
        _ => "",
    }
}

fn display_type(raw: &str) -> &str {
    match raw {
        "xs:string" => "string",
        "xs:float" => "float",
        "xs:boolean" => "boolean",
        other => other,
    }
}

fn type_cell(raw: &str, type_map: &HashMap<String, Vec<String>>) -> String {
    let display = display_type(raw);
    let has_values = type_map.get(raw).map(|v| !v.is_empty()).unwrap_or(false);
    if has_values {
        let anchor = display.to_lowercase();
        format!("[`{display}`](/types#{anchor})")
    } else {
        format!("`{display}`")
    }
}

fn attrs_table(md: &mut String, attrs: &[AttrDef], type_map: &HashMap<String, Vec<String>>) {
    if attrs.is_empty() {
        md.push_str("This element has no attributes.\n\n");
        return;
    }
    md.push_str("| Attribute | Type | Required |\n");
    md.push_str("|-----------|------|----------|\n");
    for attr in attrs {
        let req = if attr.required { "Yes" } else { "No" };
        md.push_str(&format!(
            "| `{}` | {} | {} |\n",
            attr.name,
            type_cell(&attr.typ, type_map),
            req
        ));
    }
    md.push('\n');
}

fn generate_types_page(type_map: &HashMap<String, Vec<String>>, used_types: &HashSet<String>) -> String {
    let mut md = String::from("# Types\n\n");

    let mut types: Vec<(&str, &str)> = type_map
        .iter()
        .filter(|(key, vals)| !vals.is_empty() && used_types.contains(*key))
        .map(|(key, _)| (key.as_str(), display_type(key.as_str())))
        .collect();
    types.sort_by_key(|(_, display)| *display);

    for (key, display) in types {
        let values = &type_map[key];
        md.push_str(&format!("## {display}\n\n"));
        md.push_str("| Value |\n|-------|\n");
        for v in values {
            md.push_str(&format!("| `{v}` |\n"));
        }
        md.push('\n');
    }

    md
}

/// Returns a sensible placeholder value for a required attribute in examples.
fn attr_example_value(name: &str, typ: &str) -> &'static str {
    match name {
        "is-checked" => "is_checked",
        "is-toggled" => "is_toggled",
        "on-toggle" => "Message::Toggled",
        "on-press" => "Message::Pressed",
        "on-press-with" => "Message::PressedWith",
        "on-change" => "Message::Changed",
        "on-action" => "Message::Action",
        "on-input" => "Message::Input",
        "on-submit" => "Message::Submit",
        "on-paste" => "Message::Paste",
        "on-show" => "Message::Shown",
        "on-resize" => "Message::Resized",
        "on-hide" => "Message::Hidden",
        "placeholder" => "Enter text...",
        "value" => "value",
        "content" => "content",
        "range-start" => "0",
        "range-end" => "100",
        "position" => "top",
        "condition" => "show",
        "iterable" => "items",
        "method" => "my_widget",
        _ => match typ {
            "xs:float" => "10",
            "xs:boolean" => "true",
            "Length" => "fill",
            "Horizontal" => "center",
            "Vertical" => "center",
            _ => "value",
        },
    }
}

/// Converts an XSD regex pattern to a human-readable example string.
fn simplify_pattern(pattern: &str) -> String {
    pattern
        .replace(r"\(", "(")
        .replace(r"\)", ")")
        .replace(".+", "N")
}

/// Parses all xs:simpleType definitions from the XSD and returns a map of
/// type name → list of possible values (for use in the "Values" column).
///
/// Depth accounting (depth incremented before processing Start events):
///   xs:schema Start       → depth 1
///   xs:simpleType Start   → depth 2
///   xs:restriction Start  → depth 3
///   xs:enumeration Empty  → depth 3  (sibling of restriction children)
///   xs:pattern Empty      → depth 3
///   xs:union Empty        → depth 2  (direct child of simpleType)
fn parse_type_map(content: &str) -> HashMap<String, Vec<String>> {
    let mut reader = Reader::from_str(content);
    let mut type_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut union_pending: HashMap<String, Vec<String>> = HashMap::new();
    let mut current: Option<String> = None;
    let mut depth: i32 = 0;

    loop {
        match reader.read_event().expect("XSD parse error") {
            Event::Start(e) => {
                depth += 1;
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if local == "simpleType" && depth == 2 {
                    if let Some(name) = get_attr(&e, b"name") {
                        type_map.entry(name.clone()).or_default();
                        current = Some(name);
                    }
                }
            }
            Event::Empty(e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if local == "enumeration" && depth == 3 {
                    if let Some(ref t) = current {
                        if let Some(val) = get_attr(&e, b"value") {
                            type_map.entry(t.clone()).or_default().push(val);
                        }
                    }
                } else if local == "pattern" && depth == 3 {
                    if let Some(ref t) = current {
                        if let Some(pat) = get_attr(&e, b"value") {
                            type_map
                                .entry(t.clone())
                                .or_default()
                                .push(simplify_pattern(&pat));
                        }
                    }
                } else if local == "union" && depth == 2 {
                    if let Some(ref t) = current {
                        if let Some(members_str) = get_attr(&e, b"memberTypes") {
                            let members: Vec<String> = members_str
                                .split_whitespace()
                                .map(|s| s.to_string())
                                .collect();
                            union_pending.insert(t.clone(), members);
                        }
                    }
                }
            }
            Event::End(e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if local == "simpleType" && depth == 2 {
                    current = None;
                }
                depth -= 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }

    // Resolve union types by expanding their member types.
    for (name, members) in &union_pending {
        let mut values: Vec<String> = Vec::new();
        for member in members {
            match member.as_str() {
                "xs:float" | "xs:integer" => values.push("number".to_string()),
                other => {
                    if let Some(member_vals) = type_map.get(other) {
                        values.extend(member_vals.clone());
                    }
                }
            }
        }
        type_map.insert(name.clone(), values);
    }

    type_map
}

struct AttrDef {
    name: String,
    typ: String,
    required: bool,
}

struct Widget {
    name: String,
    description: Option<String>,
    attrs: Vec<AttrDef>,
    has_content: bool,
    has_children: bool,
}

struct StateVariant {
    name: String,
    attrs: Vec<AttrDef>,
}

struct Style {
    name: String,
    attrs: Vec<AttrDef>,
    variants: Vec<StateVariant>,
}

fn get_attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

fn parse_xsd(path: &PathBuf) -> (Vec<Widget>, Vec<Style>, HashMap<String, Vec<String>>) {
    let content = fs::read_to_string(path).expect("Failed to read XSD file");
    let type_map = parse_type_map(&content);
    let (widget_names, style_names) = discover_element_lists(&content);

    let mut reader = Reader::from_str(&content);
    let mut widgets: HashMap<String, Widget> = HashMap::new();
    let mut current_widget: Option<String> = None;
    let mut depth: i32 = 0;
    let mut reading_doc = false;

    loop {
        match reader.read_event().expect("XSD parse error") {
            Event::Start(e) => {
                depth += 1;
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();

                if local == "element" && depth == 2 {
                    if let Some(name) = get_attr(&e, b"name") {
                        if widget_names.contains(&name) {
                            widgets.entry(name.clone()).or_insert_with(|| Widget {
                                name: name.clone(),
                                description: None,
                                attrs: Vec::new(),
                                has_content: false,
                                has_children: false,
                            });
                            current_widget = Some(name);
                        }
                    }
                } else if local == "documentation" && current_widget.is_some() {
                    reading_doc = true;
                } else if local == "complexType" && depth == 3 && current_widget.is_some() {
                    if let Some(ref w) = current_widget {
                        if get_attr(&e, b"mixed").as_deref() == Some("true") {
                            if let Some(widget) = widgets.get_mut(w) {
                                widget.has_content = true;
                            }
                        }
                    }
                } else if (local == "choice" || local == "all") && depth == 4 {
                    if let Some(ref w) = current_widget {
                        if let Some(widget) = widgets.get_mut(w) {
                            widget.has_children = true;
                        }
                    }
                }
            }
            Event::Text(e) => {
                if reading_doc {
                    if let Some(ref w) = current_widget {
                        let text = e.unescape().unwrap_or_default().trim().to_string();
                        if !text.is_empty() {
                            if let Some(widget) = widgets.get_mut(w) {
                                widget.description = Some(text);
                            }
                        }
                    }
                }
            }
            Event::Empty(e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();

                if local == "attribute" && depth == 3 {
                    if let Some(ref w) = current_widget {
                        let attr_name = get_attr(&e, b"name");
                        let typ = get_attr(&e, b"type")
                            .unwrap_or_else(|| "xs:string".to_string());
                        let use_ = get_attr(&e, b"use")
                            .unwrap_or_else(|| "optional".to_string());

                        if let Some(n) = attr_name {
                            if let Some(widget) = widgets.get_mut(w) {
                                widget.attrs.push(AttrDef {
                                    name: n,
                                    typ,
                                    required: use_ == "required",
                                });
                            }
                        }
                    }
                }
            }
            Event::End(e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if local == "documentation" {
                    reading_doc = false;
                } else if e.name().as_ref() == b"xs:element" && depth == 2 {
                    current_widget = None;
                }
                depth -= 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }

    let widgets = widget_names
        .iter()
        .filter_map(|name| widgets.remove(name.as_str()))
        .collect();

    // --- Parse styles ---
    // Style elements at depth 2, top-level attrs at depth 3.
    // State variant child elements (active/hovered/etc.) at depth 5 (inside xs:choice at depth 4).
    // State variant attrs at depth 6.
    let mut reader = Reader::from_str(&content);
    let mut styles: HashMap<String, Style> = HashMap::new();
    let mut current_style: Option<String> = None;
    let mut current_variant: Option<usize> = None;
    let mut depth: i32 = 0;

    loop {
        match reader.read_event().expect("XSD parse error") {
            Event::Start(e) => {
                depth += 1;
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();

                if local == "element" && depth == 2 {
                    if let Some(name) = get_attr(&e, b"name") {
                        if style_names.contains(&name) {
                            styles.entry(name.clone()).or_insert_with(|| Style {
                                name: name.clone(),
                                attrs: Vec::new(),
                                variants: Vec::new(),
                            });
                            current_style = Some(name);
                            current_variant = None;
                        }
                    }
                } else if local == "element" && depth == 5 && current_style.is_some() {
                    if let Some(variant_name) = get_attr(&e, b"name") {
                        if let Some(ref s) = current_style {
                            if let Some(style) = styles.get_mut(s) {
                                style.variants.push(StateVariant {
                                    name: variant_name,
                                    attrs: Vec::new(),
                                });
                                current_variant = Some(style.variants.len() - 1);
                            }
                        }
                    }
                }
            }
            Event::Empty(e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();

                if local == "attribute" && current_style.is_some() {
                    let attr_name = get_attr(&e, b"name");
                    let typ = get_attr(&e, b"type").unwrap_or_else(|| "xs:string".to_string());
                    let use_ = get_attr(&e, b"use").unwrap_or_else(|| "optional".to_string());

                    if let Some(n) = attr_name {
                        let attr = AttrDef { name: n, typ, required: use_ == "required" };
                        if let Some(ref s) = current_style {
                            if let Some(style) = styles.get_mut(s) {
                                if depth == 6 {
                                    if let Some(idx) = current_variant {
                                        if let Some(variant) = style.variants.get_mut(idx) {
                                            variant.attrs.push(attr);
                                        }
                                    }
                                } else if depth == 3 {
                                    style.attrs.push(attr);
                                }
                            }
                        }
                    }
                }
            }
            Event::End(e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if local == "element" && depth == 2 {
                    current_style = None;
                    current_variant = None;
                } else if local == "element" && depth == 5 {
                    current_variant = None;
                }
                depth -= 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }

    let styles = style_names
        .iter()
        .filter_map(|name| styles.remove(name.as_str()))
        .collect();

    (widgets, styles, type_map)
}

fn generate_example(widget: &Widget) -> String {
    let name = &widget.name;

    let req_attrs: String = widget
        .attrs
        .iter()
        .filter(|a| a.required)
        .map(|a| format!(" {}=\"{}\"", a.name, attr_example_value(&a.name, &a.typ)))
        .collect::<Vec<_>>()
        .join("");

    if widget.has_content {
        format!("<{name}{req_attrs}>Hello, world!</{name}>")
    } else if name == "if" {
        format!(
            "<{name}{req_attrs}>\n    <true>\n        <text>Yes</text>\n    </true>\n    <false>\n        <text>No</text>\n    </false>\n</{name}>"
        )
    } else if name == "tooltip" {
        format!(
            "<{name}{req_attrs}>\n    <button on-press=\"Message::Clicked\">\n        <text>Hover me</text>\n    </button>\n    <text>Tooltip text</text>\n</{name}>"
        )
    } else if widget.has_children {
        format!("<{name}{req_attrs}>\n    <text>...</text>\n</{name}>")
    } else {
        format!("<{name}{req_attrs} />")
    }
}

fn generate_widget_markdown(widget: &Widget, type_map: &HashMap<String, Vec<String>>) -> String {
    let title = &widget.name;
    let desc = widget
        .description
        .as_deref()
        .unwrap_or_else(|| fallback_description(title));
    let mut md = format!("# `<{title}>`\n\n{desc}\n\n");

    let example = generate_example(widget);
    md.push_str("## Example\n\n```xml\n");
    md.push_str(&example);
    md.push_str("\n```\n\n");

    if widget.has_content {
        md.push_str("The text content of this element is used as the displayed string.\n\n");
    }

    md.push_str("## Attributes\n\n");
    attrs_table(&mut md, &widget.attrs, type_map);

    md
}

fn generate_style_example(style: &Style) -> String {
    let name = &style.name;
    if style.variants.is_empty() {
        format!("<{name} id=\"my-style\" />")
    } else {
        let children: String = style
            .variants
            .iter()
            .map(|v| format!("    <{} />\n", v.name))
            .collect();
        format!("<{name} id=\"my-style\">\n{children}</{name}>")
    }
}

fn generate_style_markdown(style: &Style, type_map: &HashMap<String, Vec<String>>) -> String {
    let name = &style.name;
    let mut md = format!("# `<{name}>`\n\n");

    md.push_str("## Example\n\n```xml\n");
    md.push_str(&generate_style_example(style));
    md.push_str("\n```\n\n");

    md.push_str("## Attributes\n\n");
    attrs_table(&mut md, &style.attrs, type_map);

    if !style.variants.is_empty() {
        md.push_str("## States\n\n");
        for variant in &style.variants {
            md.push_str(&format!("### `<{}>`\n\n", variant.name));
            attrs_table(&mut md, &variant.attrs, type_map);
        }
    }

    md
}

fn main() {
    let docs_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("docs-gen has no parent directory")
        .to_path_buf();

    let schema_path = docs_dir
        .parent()
        .expect("docs/ has no parent directory")
        .join("schemas/0.0.2.xsd");

    let widgets_dir = docs_dir.join("pages/schema");
    let styles_dir = docs_dir.join("pages/styles");
    let sidebar_path = docs_dir.join(".vitepress/generated-sidebar.json");

    fs::create_dir_all(&widgets_dir).expect("Failed to create pages/schema/");
    fs::create_dir_all(&styles_dir).expect("Failed to create pages/styles/");

    let (widgets, styles, type_map) = parse_xsd(&schema_path);

    let mut widget_sidebar: Vec<serde_json::Value> = Vec::new();
    let mut style_sidebar: Vec<serde_json::Value> = Vec::new();

    for widget in &widgets {
        let md = generate_widget_markdown(widget, &type_map);
        let file_path = widgets_dir.join(format!("{}.md", widget.name));
        fs::write(&file_path, &md).expect("Failed to write widget markdown");
        println!("Wrote {}", file_path.display());

        widget_sidebar.push(serde_json::json!({
            "text": widget.name,
            "link": format!("/schema/{}", widget.name),
        }));
    }

    for style in &styles {
        let md = generate_style_markdown(style, &type_map);
        let file_path = styles_dir.join(format!("{}.md", style.name));
        fs::write(&file_path, &md).expect("Failed to write style markdown");
        println!("Wrote {}", file_path.display());

        style_sidebar.push(serde_json::json!({
            "text": style.name,
            "link": format!("/styles/{}", style.name),
        }));
    }

    let sidebar_json = serde_json::to_string_pretty(&serde_json::json!({
        "widgets": widget_sidebar,
        "styles": style_sidebar,
    }))
    .expect("Failed to serialize sidebar JSON");
    fs::write(&sidebar_path, sidebar_json).expect("Failed to write generated-sidebar.json");
    println!("Wrote {}", sidebar_path.display());

    let used_types: HashSet<String> = widgets
        .iter()
        .flat_map(|w| w.attrs.iter().map(|a| a.typ.clone()))
        .chain(styles.iter().flat_map(|s| {
            s.attrs
                .iter()
                .chain(s.variants.iter().flat_map(|v| v.attrs.iter()))
                .map(|a| a.typ.clone())
        }))
        .collect();

    let types_md = generate_types_page(&type_map, &used_types);
    let types_path = docs_dir.join("pages/types.md");
    fs::write(&types_path, types_md).expect("Failed to write types.md");
    println!("Wrote {}", types_path.display());
}
