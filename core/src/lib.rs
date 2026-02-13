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

#[derive(Default)]
pub struct TextAttrs {
    pub size: Option<f32>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub color: Option<Color>,
}

pub enum Node {
    Container {
        id: Option<String>,
        padding: Padding,
        children: Vec<Node>,
    },
    Text {
        content: String,
        attrs: TextAttrs,
    },
}
