# State Access

The `layout!` macro expands XML into Rust code that runs inside a `&self` method (typically `view`). Attribute values and text content that reference application state are automatically resolved against `self`.

## Field and Method Attributes

Attributes that accept state — such as `value` on `<text-input>` or `is-checked` on `<checkbox>` — are written as bare field or method paths. The macro prefixes them with `self.` automatically.

```xml
<text-input placeholder="Search…" value="query" />
```

Expands to:

```rust
iced::widget::text_input("Search…", &self.query)
```

Dot-separated paths and method calls are also supported:

```xml
<text-input placeholder="Name" value="user.name" />
<checkbox is-checked="form.is_valid()" on-toggle="toggle_valid" />
```

Expand to:

```rust
iced::widget::text_input("Name", &self.user.name)
iced::widget::checkbox(self.form.is_valid())
```

## Event Handler Attributes

Attributes that accept a handler — such as `on-submit`, `on-press`, `on-input`, `on-toggle` — accept two forms.

### Message variant (contains `::`)

If the value contains `::`, it is emitted as-is:

```xml
<button on-press="Message::Clicked">…</button>
```

```rust
iced::widget::button(…).on_press(Message::Clicked)
```

### Method name (no `::`)

Otherwise the value is treated as a method on `self`. The exact wrapping depends on the handler:

| Attribute | Generated code |
|-----------|---------------|
| `on-press`, `on-submit`, `on-press-maybe`, `on-submit-maybe` | `self.method()` |
| `on-press-with` | `\|\| self.method()` |
| `on-input`, `on-paste` | `\|s\| self.method(s)` |
| `on-toggle` | `\|checked\| self.method(checked)` |
| `on-change` (vertical-slider) | `\|v\| self.method(v)` |

Examples:

```xml
<text-input value="query" on-input="update_query" on-submit="submit" />
```

```rust
iced::widget::text_input("", &self.query)
    .on_input(|s| self.update_query(s))
    .on_submit(self.submit())
```

```xml
<checkbox is-checked="checked" on-toggle="toggle" />
```

```rust
iced::widget::checkbox(self.checked)
    .on_toggle(|checked| self.toggle(checked))
```

## Text Interpolation

Inside `<text>` content (and other text-bearing attributes such as `label` on `<checkbox>`), you can embed state values using `${…}` syntax. Like attribute paths, the variable is resolved against `self`.

### Single variable

When the entire content is a single `${…}` expression, the macro passes a reference directly — no allocation:

```xml
<text>${username}</text>
```

```rust
iced::widget::text(&self.username)
```

### Mixed content

When the content contains both literal text and variables, the macro generates a `format!` call:

```xml
<text>Hello, ${username}! You have ${message_count} messages.</text>
```

```rust
iced::widget::text(format!("Hello, {}! You have {} messages.", self.username, self.message_count))
```
