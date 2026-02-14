# iced-layout

Write your [iced](https://iced.rs) UI layouts in XML instead of deeply nested Rust code. The `layout!()` macro turns XML into widget trees at compile time — what you write in XML is what iced sees in code. Comes with a schema for IDE autocompletion and validation out of the box.

## Quick Start

**Cargo.toml:**

```toml
[dependencies]
iced = "0.14"
iced-layout = "0.0.1"
```

**page/my-layout.xml:**

```xml
<?xml version="1.0" encoding="utf-8" ?>
<layout xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="path/to/schema/0.14.0.xsd">
    <styles>
        <container-style id="card" background-color="#f0f0f0" border-radius="8" />
    </styles>
    <root>
        <column spacing="10" padding="20">
            <container style="card" padding="16">
                <text size="24">Hello, ${user_name}!</text>
            </container>
            <button on-press="Message::Increment">
                <text>Count: ${counter}</text>
            </button>
        </column>
    </root>
</layout>
```

**src/main.rs:**

```rust
use iced::Task;
use iced_layout::layout;

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::render).run()
}

struct App {
    user_name: String,
    counter: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        (Self { user_name: "World".into(), counter: 0 }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.counter += 1,
        }
        Task::none()
    }

    fn render(&self) -> iced::Element<'_, Message> {
        layout!("page/my-layout.xml")
    }
}
```

The `layout!()` macro reads the XML at compile time and expands it into iced widget calls. The path is relative to `CARGO_MANIFEST_DIR`.

## Rebuilding on XML Changes

Because `layout!()` is a proc macro, Cargo only sees the `.rs` file that calls it as a dependency — not the XML file it reads. This means editing an XML layout file alone won't trigger a rebuild.

To fix this, add a `build.rs` that tells Cargo to watch your layout directory:

```rust
// build.rs
fn main() {
    println!("cargo::rerun-if-changed=page/");
}
```

This ensures that any change inside `page/` (or whichever directory holds your XML files) triggers a recompilation, so the `layout!()` macro re-reads the updated XML.

Without this, you'd need to touch a `.rs` file or run `cargo clean` to see XML changes take effect.

## XML Structure

Every layout file has this shape:

```xml
<layout>
    <styles>
        <!-- optional style definitions -->
    </styles>
    <root>
        <!-- exactly one root widget -->
    </root>
</layout>
```

Add the `xsi:noNamespaceSchemaLocation` attribute on `<layout>` pointing to `schema/0.14.0.xsd` for IDE autocomplete and validation.

## Widgets

### `<container>`

Wraps exactly one child element with optional padding and styling.

```xml
<container id="main" padding="20" width="fill" style="card">
    <text>Content</text>
</container>
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `id` | string | Element identifier |
| `style` | string | Reference to a `<container-style>` |
| `padding` | f32 | Padding on all sides |
| `padding-top`, `padding-right`, `padding-bottom`, `padding-left` | f32 | Per-side padding |
| `width`, `height` | Length | Sizing |
| `max-width` | f32 | Maximum width |
| `align-x` | Horizontal | Horizontal alignment |
| `align-y` | Vertical | Vertical alignment |
| `clip` | bool | Clip overflowing content |

### `<text>`

Displays text with optional interpolation.

```xml
<text size="18" color="#333">Counter: ${counter}</text>
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `style` | string | Reference to a `<text-style>` |
| `size` | f32 | Font size in pixels |
| `line-height` | LineHeight | Line height |
| `width`, `height` | Length | Sizing |
| `align-x` | TextAlignment | Text alignment |
| `align-y` | Vertical | Vertical alignment |
| `color` | Color | Text color (overrides style) |

### `<row>`

Lays out children horizontally.

```xml
<row spacing="10" align-y="center">
    <text>Left</text>
    <text>Right</text>
</row>
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `spacing` | f32 | Space between children |
| `padding` | f32 | Padding on all sides |
| `padding-top`, `padding-right`, `padding-bottom`, `padding-left` | f32 | Per-side padding |
| `width`, `height` | Length | Sizing |
| `align-y` | Vertical | Vertical alignment of children |
| `clip` | bool | Clip overflowing content |

### `<column>`

Lays out children vertically.

```xml
<column spacing="10" align-x="center" max-width="600">
    <text>Top</text>
    <text>Bottom</text>
</column>
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `spacing` | f32 | Space between children |
| `padding` | f32 | Padding on all sides |
| `padding-top`, `padding-right`, `padding-bottom`, `padding-left` | f32 | Per-side padding |
| `width`, `height` | Length | Sizing |
| `max-width` | f32 | Maximum width |
| `align-x` | Horizontal | Horizontal alignment of children |
| `clip` | bool | Clip overflowing content |

### `<button>`

Interactive button with at most one child element.

```xml
<button on-press="Message::Click" style="primary" padding="10">
    <text>Click me</text>
</button>
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `style` | string | Reference to a `<button-style>` |
| `padding` | f32 | Padding on all sides |
| `padding-top`, `padding-right`, `padding-bottom`, `padding-left` | f32 | Per-side padding |
| `width`, `height` | Length | Sizing |
| `clip` | bool | Clip overflowing content |
| `on-press` | Handler | Triggered on click |
| `on-press-with` | Handler | Triggered on click (wrapped in closure) |
| `on-press-maybe` | Handler | Conditionally enabled (`Option<Message>`) |

### `<stack>`

Stacks children on top of each other (z-axis).

```xml
<stack width="fill" height="fill">
    <text>Background</text>
    <text>Foreground</text>
</stack>
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `width`, `height` | Length | Sizing |
| `clip` | bool | Clip overflowing content |

### `<space>`

Empty spacing element.

```xml
<space width="20" height="10" />
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `width`, `height` | Length | Sizing |

### `<text-input>`

Single-line text input field. Self-closing element.

```xml
<text-input
    placeholder="Enter your name..."
    value="name_field"
    on-input="handle_input"
    on-submit="Message::Submit"
    width="fill"
/>
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `placeholder` | string | **Required.** Placeholder text |
| `value` | string | **Required.** Field path on `self` (e.g. `name_field`) |
| `id` | string | Element identifier |
| `secure` | bool | Password mode (hides characters) |
| `style` | string | Reference to a `<text-input-style>` |
| `width` | Length | Sizing |
| `padding` | f32 | Padding on all sides |
| `padding-top`, `padding-right`, `padding-bottom`, `padding-left` | f32 | Per-side padding |
| `size` | f32 | Font size |
| `line-height` | LineHeight | Line height |
| `align-x` | Horizontal | Text alignment |
| `on-input` | Handler | Called with input string on each keystroke |
| `on-submit` | Handler | Called when Enter is pressed |
| `on-submit-maybe` | Handler | Conditionally called on Enter (`Option<Message>`) |
| `on-paste` | Handler | Called with pasted string |

### `<checkbox>`

Checkbox with an optional label. The label is the element's text content.

```xml
<checkbox is-checked="enabled" on-toggle="toggle_enabled" size="20">
    Enable feature
</checkbox>
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `is-checked` | string | **Required.** Bool field path on `self` |
| `style` | string | Reference to a `<checkbox-style>` |
| `size` | f32 | Checkbox size |
| `width` | Length | Sizing |
| `spacing` | f32 | Space between checkbox and label |
| `text-size` | f32 | Label font size |
| `text-line-height` | LineHeight | Label line height |
| `text-shaping` | Shaping | Label text shaping |
| `text-wrapping` | Wrapping | Label text wrapping |
| `on-toggle` | Handler | Called with `bool` on toggle |
| `on-toggle-maybe` | Handler | Conditionally enabled (`Option<Message>`) |

## Attribute Types

### Length

| Value | Description |
|-------|-------------|
| `fill` | Fill available space |
| `shrink` | Shrink to fit content |
| `fill-portion(N)` | Fill proportional share (N is an integer) |
| `200` | Fixed pixel value (any number) |

### Color

Hex format: `#rgb`, `#rgba`, `#rrggbb`, or `#rrggbbaa`.

```xml
<text color="#333">Dark</text>
<text color="#ff000080">Semi-transparent red</text>
```

### Horizontal

`left` | `center` | `right`

### Vertical

`top` | `center` | `bottom`

### TextAlignment

`default` | `left` | `center` | `right` | `justified`

### LineHeight

- `relative(1.5)` — multiplier of font size
- `absolute(20)` — fixed pixel height

### Shaping

`auto` | `basic` | `advanced`

### Wrapping

`none` | `word` | `glyph` | `word-or-glyph`

## Text Interpolation

Use `${field_path}` inside `<text>` or checkbox labels to reference fields on `self`:

```xml
<text>Hello, ${user.name}!</text>
<text>${counter}</text>
```

- A single `${var}` compiles to a reference: `&self.var`
- Mixed content like `Hello, ${name}!` compiles to `format!("Hello, {}!", self.name)`

## Event Handlers

Handler attributes accept two forms:

**Message expression** (contains `::`) — passed directly:

```xml
<button on-press="Message::Click" />
<text-input on-input="Message::InputChanged" />
```

**Method name** (no `::`) — calls `self.method(...)`:

```xml
<button on-press="handle_click" />         <!-- self.handle_click() -->
<button on-press-with="make_message" />    <!-- || self.make_message() -->
<button on-press-maybe="maybe_click" />    <!-- self.maybe_click() → Option<Message> -->
<text-input on-input="handle_input" />     <!-- |s| self.handle_input(s) -->
<checkbox on-toggle="handle_toggle" />     <!-- |checked| self.handle_toggle(checked) -->
```

Expected method signatures for the method-name form:

| Handler | Signature |
|---------|-----------|
| `on-press` | `fn(&self) -> Message` |
| `on-press-with` | `fn(&self) -> Message` |
| `on-press-maybe` | `fn(&self) -> Option<Message>` |
| `on-submit` | `fn(&self) -> Message` |
| `on-submit-maybe` | `fn(&self) -> Option<Message>` |
| `on-input` | `fn(&self, String) -> Message` |
| `on-paste` | `fn(&self, String) -> Message` |
| `on-toggle` | `fn(&self, bool) -> Message` |
| `on-toggle-maybe` | `fn(&self) -> Option<Message>` |

## Styles

Define styles in the `<styles>` block and reference them by `id` from widgets.

### `<container-style>`

```xml
<container-style id="card"
    background-color="#ffffff"
    text-color="#333333"
    border-color="#cccccc"
    border-width="1"
    border-radius="8"
    shadow-color="#00000033"
    shadow-offset-x="0"
    shadow-offset-y="2"
    shadow-blur-radius="4"
    snap="false"
/>
```

Individual border radius: `border-radius-top-left`, `border-radius-top-right`, `border-radius-bottom-right`, `border-radius-bottom-left`.

### `<text-style>`

```xml
<text-style id="heading" color="#111111" />
```

### `<button-style>`

Supports per-state overrides. State elements inherit base attributes and override only what's specified.

```xml
<button-style id="primary" background-color="#0066ff" text-color="#ffffff" border-radius="4">
    <hovered background-color="#0055dd" />
    <pressed background-color="#004499" />
    <disabled background-color="#cccccc" text-color="#888888" />
</button-style>
```

States: `<active>`, `<hovered>`, `<pressed>`, `<disabled>`. Each accepts the same attributes as the base.

Base and state attributes: `background-color`, `text-color`, `border-color`, `border-width`, `border-radius`, `border-radius-*`, `shadow-color`, `shadow-offset-x`, `shadow-offset-y`, `shadow-blur-radius`, `snap`.

### `<checkbox-style>`

```xml
<checkbox-style id="custom"
    background-color="#ffffff"
    icon-color="#0066ff"
    border-color="#cccccc"
    border-width="1"
    border-radius="4"
    text-color="#333333"
/>
```

### `<text-input-style>`

Supports per-state overrides like button styles.

```xml
<text-input-style id="search"
    background-color="#f5f5f5"
    border-color="#dddddd"
    border-width="1"
    border-radius="4"
    icon="#888888"
    placeholder="#aaaaaa"
    value="#333333"
    selection="#0066ff4d"
>
    <active border-color="#0066ff" />
    <hovered border-color="#0055dd" />
    <disabled background-color="#eeeeee" />
</text-input-style>
```

States: `<active>`, `<hovered>`, `<disabled>`. Each accepts the same attributes as the base.

## XSD Schema

The `schema/0.14.0.xsd` file provides XML validation and IDE autocomplete. Point to it from your layout files:

```xml
<layout xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="../../schema/0.14.0.xsd">
```