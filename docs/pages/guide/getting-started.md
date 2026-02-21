# Getting Started

## Installation

Add `iced-layout` to your `Cargo.toml`:

```toml
[dependencies]
iced-layout = "0.0.1"
```

### Versioning

iced-layout versions are tied to the iced version they target:

| iced-layout | iced     |
|-------------|----------|
| `0.0.*`     | `0.14.0` |

Choose the `iced-layout` version that matches your iced version.

## Basic Usage

Define your layout in an XML file (e.g. `src/my_layout.xml`):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<layout xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/dekamaru/iced-layout/main/schemas/0.0.1.xsd">
    <root>
        <column>
            <text size="24">Hello, iced-layout!</text>
            <button on-press="Message::Clicked">
                <text>Click me</text>
            </button>
        </column>
    </root>
</layout>
```

Modern IDEs will provide schema validation and autocompletion for layout files out of the box.

Then use the `layout!` macro in your Rust code:

```rust
use iced_layout::layout;

fn view(&self) -> iced::Element<Message> {
    layout!("src/my_layout.xml")
}
```

The macro expands your XML into iced widget calls at compile time.

## Working with the Macro

Because `layout!` is a proc macro that reads files at compile time, Cargo won't automatically know to re-compile when your XML changes. You need to tell it explicitly.

Keep your layout files in a dedicated folder (e.g. `layouts/`) and add a `build.rs` at the root of your crate:

```rust
fn main() {
    println!("cargo::rerun-if-changed=layouts/");
}
```

This instructs Cargo to re-run the build script — and thus re-compile your crate — whenever any file inside `layouts/` changes. Without this, edits to XML files won't trigger a rebuild until something else forces one.

Reference layouts relative to your crate root:

```rust
layout!("layouts/main.xml")
```
