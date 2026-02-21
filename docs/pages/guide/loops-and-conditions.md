# Loops and Conditions

## Conditional rendering with `<if>`

The `<if>` element conditionally renders a widget based on a boolean expression from your state.

### With both branches

When both a `<true>` and `<false>` child are provided, the result is a single widget that can be placed anywhere — including inside `<container>` or `<button>`.

```xml
<if condition="is_logged_in">
    <true>
        <text>Welcome back!</text>
    </true>
    <false>
        <text>Please log in.</text>
    </false>
</if>
```

Expands to:

```rust
{
    let __if_result: iced::Element<'_, _> = if self.is_logged_in {
        (iced::widget::text("Welcome back!")).into()
    } else {
        (iced::widget::text("Please log in.")).into()
    };
    __if_result
}
```

### Without a false branch

Omitting `<false>` produces an *optional* element. It can only appear as a child of a multi-child container (`<column>`, `<row>`, `<stack>`). Using it inside `<container>` or `<button>` is a compile error.

```xml
<column>
    <text>Always visible</text>
    <if condition="show_hint">
        <true>
            <text>Hint text</text>
        </true>
    </if>
</column>
```

### Negating a condition

Prefix the condition with `!` to invert it:

```xml
<if condition="!is_loading">
    <true>
        <text>Done</text>
    </true>
</if>
```

### Field paths in conditions

Like all state-bearing attributes, the condition is resolved against `self`. Dot-separated paths and method calls work:

```xml
<if condition="form.has_errors()">…</if>
<if condition="!user.is_active">…</if>
```

## Loops with `<foreach>`

The `<foreach>` element iterates over a collection and renders one widget per item. It can only appear as a child of a multi-child container (`<column>`, `<row>`, `<stack>`).

```xml
<column>
    <foreach iterable="items">
        <text>${item}</text>
    </foreach>
</column>
```

Expands to:

```rust
iced::widget::Column::new()
    .extend(self.items.iter().map(|item| {
        (iced::widget::text(&item)).into()
    }))
```

### The `item` variable

Inside `<foreach>`, `item` refers to the current element of the iterator. It is a local binding — **not** a field on `self` — so you write `item` (or `${item}`) without any prefix.

```xml
<foreach iterable="messages">
    <row>
        <text>${item}</text>
    </row>
</foreach>
```

The `item` local also takes precedence over any `self.item` field you might have, should both exist.

### Dot-separated iterables

The `iterable` attribute follows the same field-path rules as other state attributes:

```xml
<foreach iterable="chat.messages">
    <text>${item}</text>
</foreach>
```

```rust
self.chat.messages.iter().map(|item| { … })
```
