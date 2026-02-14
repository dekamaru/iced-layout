use iced_layout_core::{
    BorderRadius, Color, Horizontal, Length, LineHeight, Padding, Shaping, TextAlignment, Vertical,
    Wrapping,
};
use quote::quote;

pub fn generate_padding(padding: &Padding) -> Option<proc_macro2::TokenStream> {
    if padding.top.is_none()
        && padding.right.is_none()
        && padding.bottom.is_none()
        && padding.left.is_none()
    {
        return None;
    }

    let top = padding.top.unwrap_or(0.0);
    let right = padding.right.unwrap_or(0.0);
    let bottom = padding.bottom.unwrap_or(0.0);
    let left = padding.left.unwrap_or(0.0);
    Some(quote! {
        iced::Padding { top: #top, right: #right, bottom: #bottom, left: #left }
    })
}

pub fn generate_length(len: &Length) -> proc_macro2::TokenStream {
    match len {
        Length::Fill => quote! { iced::Length::Fill },
        Length::FillPortion(v) => quote! { iced::Length::FillPortion(#v) },
        Length::Shrink => quote! { iced::Length::Shrink },
        Length::Fixed(v) => quote! { iced::Length::Fixed(#v) },
    }
}

pub fn generate_color(c: &Color) -> proc_macro2::TokenStream {
    let r = c.r;
    let g = c.g;
    let b = c.b;
    let a = c.a;
    quote! { iced::Color { r: #r, g: #g, b: #b, a: #a } }
}

pub fn generate_option_color(c: &Option<Color>) -> proc_macro2::TokenStream {
    match c {
        Some(c) => {
            let c = generate_color(c);
            quote! { Some(#c) }
        }
        None => quote! { None },
    }
}

pub fn generate_color_or(
    c: &Option<Color>,
    fallback: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match c {
        Some(c) => generate_color(c),
        None => fallback,
    }
}

pub fn generate_horizontal(h: &Horizontal) -> proc_macro2::TokenStream {
    match h {
        Horizontal::Left => quote! { iced::alignment::Horizontal::Left },
        Horizontal::Center => quote! { iced::alignment::Horizontal::Center },
        Horizontal::Right => quote! { iced::alignment::Horizontal::Right },
    }
}

pub fn generate_vertical(v: &Vertical) -> proc_macro2::TokenStream {
    match v {
        Vertical::Top => quote! { iced::alignment::Vertical::Top },
        Vertical::Center => quote! { iced::alignment::Vertical::Center },
        Vertical::Bottom => quote! { iced::alignment::Vertical::Bottom },
    }
}

pub fn generate_line_height(lh: &LineHeight) -> proc_macro2::TokenStream {
    match lh {
        LineHeight::Relative(v) => quote! { iced::widget::text::LineHeight::Relative(#v) },
        LineHeight::Absolute(v) => {
            quote! { iced::widget::text::LineHeight::Absolute(iced::Pixels(#v)) }
        }
    }
}

pub fn generate_text_alignment(a: &TextAlignment) -> proc_macro2::TokenStream {
    match a {
        TextAlignment::Default => quote! { iced::widget::text::Alignment::Default },
        TextAlignment::Left => quote! { iced::widget::text::Alignment::Left },
        TextAlignment::Center => quote! { iced::widget::text::Alignment::Center },
        TextAlignment::Right => quote! { iced::widget::text::Alignment::Right },
        TextAlignment::Justified => quote! { iced::widget::text::Alignment::Justified },
    }
}

pub fn generate_shaping(s: &Shaping) -> proc_macro2::TokenStream {
    match s {
        Shaping::Auto => quote! { iced::widget::text::Shaping::Auto },
        Shaping::Basic => quote! { iced::widget::text::Shaping::Basic },
        Shaping::Advanced => quote! { iced::widget::text::Shaping::Advanced },
    }
}

pub fn generate_wrapping(w: &Wrapping) -> proc_macro2::TokenStream {
    match w {
        Wrapping::None => quote! { iced::widget::text::Wrapping::None },
        Wrapping::Word => quote! { iced::widget::text::Wrapping::Word },
        Wrapping::Glyph => quote! { iced::widget::text::Wrapping::Glyph },
        Wrapping::WordOrGlyph => quote! { iced::widget::text::Wrapping::WordOrGlyph },
    }
}

pub fn generate_border_radius(br: &BorderRadius) -> proc_macro2::TokenStream {
    let tl = br.top_left.unwrap_or(0.0);
    let tr = br.top_right.unwrap_or(0.0);
    let brr = br.bottom_right.unwrap_or(0.0);
    let bl = br.bottom_left.unwrap_or(0.0);
    quote! { iced::border::Radius { top_left: #tl, top_right: #tr, bottom_right: #brr, bottom_left: #bl } }
}

pub fn generate_border(
    color: &Option<Color>,
    width: Option<f32>,
    radius: &BorderRadius,
) -> proc_macro2::TokenStream {
    let border_color = generate_color_or(color, quote! { iced::Color::TRANSPARENT });
    let border_width = width.unwrap_or(0.0);
    let border_radius = generate_border_radius(radius);
    quote! {
        iced::Border {
            color: #border_color,
            width: #border_width,
            radius: #border_radius,
        }
    }
}

pub fn generate_shadow(
    color: &Option<Color>,
    ox: Option<f32>,
    oy: Option<f32>,
    blur: Option<f32>,
) -> proc_macro2::TokenStream {
    let shadow_color = generate_color_or(color, quote! { iced::Color::TRANSPARENT });
    let shadow_ox = ox.unwrap_or(0.0);
    let shadow_oy = oy.unwrap_or(0.0);
    let shadow_blur = blur.unwrap_or(0.0);
    quote! {
        iced::Shadow {
            color: #shadow_color,
            offset: iced::Vector::new(#shadow_ox, #shadow_oy),
            blur_radius: #shadow_blur,
        }
    }
}
