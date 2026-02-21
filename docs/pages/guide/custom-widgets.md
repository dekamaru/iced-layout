# Custom Widgets

The `<widget>` element lets you call any method on `self` and embed the result in the layout tree. Use it to integrate custom iced widgets, third-party components, or complex widget-building logic that doesn't map cleanly to the built-in elements.

## How it works

```xml
<widget method="my_widget" />
```

Expands to:

```rust
self.my_widget()
```

The method must return something that implements `Into<iced::Element<'_, Message>>`.

## Passing a child element

A child element placed inside `<widget>` is passed as the first argument, converted to `iced::Element` via `.into()`:

```xml
<widget method="card">
    <column>
        <text>Title</text>
        <text>Body</text>
    </column>
</widget>
```

```rust
self.card((iced::widget::column![
    iced::widget::text("Title"),
    iced::widget::text("Body"),
]).into())
```

## Passing additional arguments

Use `arg-0` through `arg-9` to pass extra arguments. They are resolved against `self` (same rules as other state attributes — dot-separated paths and method calls are supported) and appended after the child in index order.

```xml
<widget method="labeled_input" arg-0="label_text" arg-1="input_value" />
```

```rust
self.labeled_input(self.label_text, self.input_value)
```

When a child is also present it comes first, followed by `arg-0`, `arg-1`, etc.:

```xml
<widget method="card" arg-0="card_style" arg-1="is_selected">
    <text>Content</text>
</widget>
```

```rust
self.card(
    (iced::widget::text("Content")).into(),
    self.card_style,
    self.is_selected,
)
```

## Example: wrapping a third-party widget

Suppose you use a custom `Badge` widget from an external crate. Define a helper on your state:

```rust
impl MyApp {
    fn badge(&self, content: iced::Element<Message>, count: u32) -> iced::Element<Message> {
        Badge::new(content, count).into()
    }
}
```

Then reference it in your layout:

```xml
<widget method="badge" arg-0="notification_count">
    <text>Inbox</text>
</widget>
```

```rust
self.badge(
    (iced::widget::text("Inbox")).into(),
    self.notification_count,
)
```

## Constraints

- `<widget>` accepts **at most one** direct child element.
- Arguments are positional — gaps in the index sequence (e.g. `arg-0` and `arg-2` without `arg-1`) are ignored; only the values present are collected, sorted by index.
- The method name must be a valid Rust identifier.
