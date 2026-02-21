# iced-layout

Write your [iced](https://iced.rs) UI layouts in XML instead of deeply nested Rust code. The `layout!()` macro turns XML into widget trees at compile time.

## Documentation

Full documentation is available at **[dekamaru.github.io/iced-layout](https://dekamaru.github.io/iced-layout/)**.

## Installation

```toml
[dependencies]
iced = "0.14"
iced-layout = "0.0.1"
```

## Usage

**page/my-layout.xml:**

```xml
<?xml version="1.0" encoding="utf-8" ?>
<layout xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/dekamaru/iced-layout/refs/heads/main/schemas/0.0.1.xsd">
    <root>
        <column spacing="10" padding="20">
            <text size="24">Hello, ${user_name}!</text>
            <button on-press="Message::Increment">
                <text>Count: ${counter}</text>
            </button>
        </column>
    </root>
</layout>
```

**src/main.rs:**

```rust
fn render(&self) -> iced::Element<'_, Message> {
    layout!("page/my-layout.xml")
}
```

## Disclaimers

> **Early stage:** This project is in active development. APIs may change without notice.

> **AI assistance:** This project was developed with AI assistance.

## License

MIT
