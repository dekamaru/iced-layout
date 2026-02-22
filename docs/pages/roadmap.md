# Roadmap

The core of iced-layout is complete — state access, event handlers, conditions, loops, styling, and custom widgets are all production-ready, with a few exceptions such as background gradients. The remaining work is primarily widget coverage: not every iced widget has a dedicated XML element yet.

That said, missing widgets are not a blocker. You can use the [`<widget>`](/guide/custom-widgets) element to call any method on `self` and drop the result into the layout tree, so any iced widget — including third-party ones — can be integrated today without waiting for built-in support.

## Core features

| Feature                             | Status     |
|-------------------------------------|------------|
| State access                        | ✅ Done     |
| Callbacks (buttons, inputs)         | ✅ Done     |
| If conditions                       | ✅ Done     |
| Loops (`<foreach>`)                 | ✅ Done     |
| Styling                             | ✅ Done     |
| Custom defined widgets (`<widget>`) | ✅ Done     |
| Support icon attributes             | 🔲 Planned |
| Background gradients                | 🔲 Planned |
| Components                          | 🔲 Planned |
| Checkbox `checked` in styles        | 🔲 Planned |
| Text editor highlight support       | 🔲 Planned |

## Widget coverage

| Widget         | Status         |
|----------------|----------------|
| Button         | ✅ Done         |
| Checkbox       | ✅ Done         |
| Column         | ✅ Done         |
| Container      | ✅ Done         |
| Row            | ✅ Done         |
| Space          | ✅ Done         |
| Stack          | ✅ Done         |
| Text           | ✅ Done         |
| TextInput      | ✅ Done         |
| Tooltip        | ✅ Done         |
| VerticalSlider | ✅ Done (0.0.2) |
| Sensor         | ✅ Done (0.0.2) |
| TextEditor     | ✅ Done (0.0.2) |
| Toggler        | ✅ Done (0.0.2) |
| ComboBox       | ✅ Done (0.0.2) |
| Float          | ✅ Done (0.0.2) |
| Grid           | 🔲 Planned     |
| MouseArea      | 🔲 Planned     |
| PaneGrid       | 🔲 Planned     |
| PickList       | 🔲 Planned     |
| Pin            | 🔲 Planned     |
| ProgressBar    | 🔲 Planned     |
| Radio          | 🔲 Planned     |
| Responsive     | 🔲 Planned     |
| Rule           | 🔲 Planned     |
| Scrollable     | 🔲 Planned     |
| Slider         | 🔲 Planned     |

## Feature-gated widgets

These widgets require optional iced feature flags to be enabled.

| Widget   | Feature flag | Status     |
|----------|--------------|------------|
| Canvas   | `canvas`     | 🔲 Planned |
| Image    | `image`      | 🔲 Planned |
| Markdown | `markdown`   | 🔲 Planned |
| QRCode   | `qr_code`    | 🔲 Planned |
| Shader   | `wgpu`       | 🔲 Planned |
| Svg      | `svg`        | 🔲 Planned |
