# XML Format

All layouts are defined in XML files. The root element must be `<layout>`, which wraps a mandatory `<root>` element and an optional `<styles>` element.

## Structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<layout xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/dekamaru/iced-layout/main/schemas/0.0.1.xsd">
    <root>
        <!-- your widget tree here -->
    </root>
    <styles>
        <!-- optional style definitions -->
    </styles>
</layout>
```

## Schema

See the [Schema Reference](/schema/container) for a full list of available elements and their attributes.

## Styles

See the [Styles Reference](/styles/container-style) for a full list of available style elements and their attributes.

Style elements (`<container-style>`, `<button-style>`, etc.) defined inside `<styles>` are referenced from widgets via the `style` attribute:

```xml
<layout xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/dekamaru/iced-layout/main/schemas/0.0.1.xsd">
    <root>
        <container style="my-box">
            <text>Styled</text>
        </container>
    </root>
    <styles>
        <container-style id="my-box" background-color="#1e1e2e" border-radius="8" />
    </styles>
</layout>
```
