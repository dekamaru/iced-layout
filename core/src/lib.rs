#[derive(Default)]
pub struct Padding {
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Clone, Copy)]
pub enum Length {
    Fill,
    FillPortion(u16),
    Shrink,
    Fixed(f32),
}

#[derive(Clone, Copy)]
pub enum Horizontal {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
pub enum Vertical {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy)]
pub enum LineHeight {
    Relative(f32),
    Absolute(f32),
}

#[derive(Clone, Copy)]
pub enum TextAlignment {
    Default,
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Clone, Copy)]
pub enum Shaping {
    Auto,
    Basic,
    Advanced,
}

#[derive(Clone, Copy)]
pub enum Wrapping {
    None,
    Word,
    Glyph,
    WordOrGlyph,
}

#[derive(Default)]
pub struct TextAttrs {
    pub size: Option<f32>,
    pub line_height: Option<LineHeight>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub align_x: Option<TextAlignment>,
    pub align_y: Option<Vertical>,
    pub color: Option<Color>,
}

#[derive(Default)]
pub struct BorderRadius {
    pub top_left: Option<f32>,
    pub top_right: Option<f32>,
    pub bottom_right: Option<f32>,
    pub bottom_left: Option<f32>,
}

#[derive(Default)]
pub struct ContainerStyle {
    pub text_color: Option<Color>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_radius: BorderRadius,
    pub shadow_color: Option<Color>,
    pub shadow_offset_x: Option<f32>,
    pub shadow_offset_y: Option<f32>,
    pub shadow_blur_radius: Option<f32>,
    pub snap: Option<bool>,
}

#[derive(Default)]
pub struct TextStyle {
    pub color: Option<Color>,
}

#[derive(Default)]
pub struct ButtonStyleFields {
    pub text_color: Option<Color>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_radius: BorderRadius,
    pub shadow_color: Option<Color>,
    pub shadow_offset_x: Option<f32>,
    pub shadow_offset_y: Option<f32>,
    pub shadow_blur_radius: Option<f32>,
    pub snap: Option<bool>,
}

#[derive(Default)]
pub struct ButtonStyle {
    pub base: ButtonStyleFields,
    pub active: Option<ButtonStyleFields>,
    pub hovered: Option<ButtonStyleFields>,
    pub pressed: Option<ButtonStyleFields>,
    pub disabled: Option<ButtonStyleFields>,
}

#[derive(Default)]
pub struct CheckboxStyle {
    pub background_color: Option<Color>,
    pub icon_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_radius: BorderRadius,
    pub text_color: Option<Color>,
}

#[derive(Default)]
pub struct TextInputStyleFields {
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_radius: BorderRadius,
    pub icon_color: Option<Color>,
    pub placeholder_color: Option<Color>,
    pub value_color: Option<Color>,
    pub selection_color: Option<Color>,
}

#[derive(Default)]
pub struct TextInputStyle {
    pub base: TextInputStyleFields,
    pub active: Option<TextInputStyleFields>,
    pub hovered: Option<TextInputStyleFields>,
    pub disabled: Option<TextInputStyleFields>,
}

pub struct Layout {
    pub container_styles: Vec<(String, ContainerStyle)>,
    pub text_styles: Vec<(String, TextStyle)>,
    pub button_styles: Vec<(String, ButtonStyle)>,
    pub checkbox_styles: Vec<(String, CheckboxStyle)>,
    pub text_input_styles: Vec<(String, TextInputStyle)>,
    pub root: Node,
}

pub enum Node {
    Container {
        id: Option<String>,
        style: Option<String>,
        padding: Padding,
        children: Vec<Node>,
    },
    Text {
        content: String,
        style: Option<String>,
        attrs: TextAttrs,
    },
    Row {
        spacing: Option<f32>,
        padding: Padding,
        width: Option<Length>,
        height: Option<Length>,
        align_y: Option<Vertical>,
        clip: Option<bool>,
        children: Vec<Node>,
    },
    Column {
        spacing: Option<f32>,
        padding: Padding,
        width: Option<Length>,
        height: Option<Length>,
        max_width: Option<f32>,
        align_x: Option<Horizontal>,
        clip: Option<bool>,
        children: Vec<Node>,
    },
    Button {
        style: Option<String>,
        padding: Padding,
        width: Option<Length>,
        height: Option<Length>,
        clip: Option<bool>,
        on_press: Option<String>,
        on_press_with: Option<String>,
        on_press_maybe: Option<String>,
        children: Vec<Node>,
    },
    Stack {
        width: Option<Length>,
        height: Option<Length>,
        clip: Option<bool>,
        children: Vec<Node>,
    },
    Space {
        width: Option<Length>,
        height: Option<Length>,
    },
    TextInput {
        placeholder: String,
        value: String,
        id: Option<String>,
        secure: Option<bool>,
        on_input: Option<String>,
        on_submit: Option<String>,
        on_submit_maybe: Option<String>,
        on_paste: Option<String>,
        width: Option<Length>,
        padding: Padding,
        size: Option<f32>,
        line_height: Option<LineHeight>,
        align_x: Option<Horizontal>,
        style: Option<String>,
    },
    Checkbox {
        label: String,
        is_checked: String,
        on_toggle: Option<String>,
        on_toggle_maybe: Option<String>,
        size: Option<f32>,
        width: Option<Length>,
        spacing: Option<f32>,
        text_size: Option<f32>,
        text_line_height: Option<LineHeight>,
        text_shaping: Option<Shaping>,
        text_wrapping: Option<Wrapping>,
        style: Option<String>,
    },
    If {
        condition: String,
        true_branch: Box<Node>,
        false_branch: Option<Box<Node>>,
    },
    ForEach {
        iterable: String,
        body: Box<Node>,
    },
}
