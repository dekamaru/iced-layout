use iced_layout_core::{
    BorderRadius, ButtonStyle, ButtonStyleFields, CheckboxStyle, ContainerStyle, OverlayMenuStyle,
    TextEditorStyle, TextEditorStyleFields, TextInputStyle, TextInputStyleFields, TogglerStyle,
};
use quote::quote;

use crate::types::{
    generate_border, generate_color, generate_color_or, generate_option_color, generate_shadow,
};

pub fn generate_container_style(s: &ContainerStyle) -> proc_macro2::TokenStream {
    let text_color = generate_option_color(&s.text_color);

    let background = match &s.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { Some(iced::Background::Color(#c)) }
        }
        None => quote! { None },
    };

    let border = generate_border(&s.border_color, s.border_width, &s.border_radius);
    let shadow = generate_shadow(
        &s.shadow_color,
        s.shadow_offset_x,
        s.shadow_offset_y,
        s.shadow_blur_radius,
    );
    let snap = s.snap.unwrap_or(false);

    quote! {
        |_theme| iced::widget::container::Style {
            text_color: #text_color,
            background: #background,
            border: #border,
            shadow: #shadow,
            snap: #snap,
        }
    }
}

pub fn generate_checkbox_style_closure(s: &CheckboxStyle) -> proc_macro2::TokenStream {
    let background = match &s.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { iced::Background::Color(#c) }
        }
        None => quote! { iced::Background::Color(iced::Color::TRANSPARENT) },
    };

    let icon_color = generate_color_or(&s.icon_color, quote! { iced::Color::BLACK });
    let border = generate_border(&s.border_color, s.border_width, &s.border_radius);
    let text_color = generate_option_color(&s.text_color);

    quote! {
        |_theme, _status| iced::widget::checkbox::Style {
            background: #background,
            icon_color: #icon_color,
            border: #border,
            text_color: #text_color,
        }
    }
}

pub fn generate_button_fields_tokens(fields: &ButtonStyleFields) -> proc_macro2::TokenStream {
    let background = match &fields.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { Some(iced::Background::Color(#c)) }
        }
        None => quote! { None },
    };

    let text_color = generate_color_or(&fields.text_color, quote! { iced::Color::BLACK });
    let border = generate_border(&fields.border_color, fields.border_width, &fields.border_radius);
    let shadow = generate_shadow(
        &fields.shadow_color,
        fields.shadow_offset_x,
        fields.shadow_offset_y,
        fields.shadow_blur_radius,
    );
    let snap = fields.snap.unwrap_or(false);

    quote! {
        iced::widget::button::Style {
            background: #background,
            text_color: #text_color,
            border: #border,
            shadow: #shadow,
            snap: #snap,
        }
    }
}

pub fn merge_button_fields(
    base: &ButtonStyleFields,
    overlay: &ButtonStyleFields,
) -> ButtonStyleFields {
    ButtonStyleFields {
        text_color: overlay.text_color.or(base.text_color),
        background_color: overlay.background_color.or(base.background_color),
        border_color: overlay.border_color.or(base.border_color),
        border_width: overlay.border_width.or(base.border_width),
        border_radius: BorderRadius {
            top_left: overlay.border_radius.top_left.or(base.border_radius.top_left),
            top_right: overlay.border_radius.top_right.or(base.border_radius.top_right),
            bottom_right: overlay
                .border_radius
                .bottom_right
                .or(base.border_radius.bottom_right),
            bottom_left: overlay
                .border_radius
                .bottom_left
                .or(base.border_radius.bottom_left),
        },
        shadow_color: overlay.shadow_color.or(base.shadow_color),
        shadow_offset_x: overlay.shadow_offset_x.or(base.shadow_offset_x),
        shadow_offset_y: overlay.shadow_offset_y.or(base.shadow_offset_y),
        shadow_blur_radius: overlay.shadow_blur_radius.or(base.shadow_blur_radius),
        snap: overlay.snap.or(base.snap),
    }
}

pub fn generate_button_style_closure(bs: &ButtonStyle) -> proc_macro2::TokenStream {
    let base_style = generate_button_fields_tokens(&bs.base);

    let status_overrides: Vec<_> = [
        (&bs.active, quote! { iced::widget::button::Status::Active }),
        (
            &bs.hovered,
            quote! { iced::widget::button::Status::Hovered },
        ),
        (
            &bs.pressed,
            quote! { iced::widget::button::Status::Pressed },
        ),
        (
            &bs.disabled,
            quote! { iced::widget::button::Status::Disabled },
        ),
    ]
    .into_iter()
    .filter_map(|(opt, status_token)| {
        opt.as_ref().map(|fields| {
            let merged = merge_button_fields(&bs.base, fields);
            let style = generate_button_fields_tokens(&merged);
            quote! { #status_token => #style }
        })
    })
    .collect();

    if status_overrides.is_empty() {
        quote! {
            |_theme, _status| #base_style
        }
    } else {
        quote! {
            |_theme, status| match status {
                #(#status_overrides,)*
                _ => #base_style
            }
        }
    }
}

pub fn generate_text_input_fields_tokens(
    fields: &TextInputStyleFields,
) -> proc_macro2::TokenStream {
    let background = match &fields.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { iced::Background::Color(#c) }
        }
        None => quote! { iced::Background::Color(iced::Color::TRANSPARENT) },
    };

    let border = generate_border(&fields.border_color, fields.border_width, &fields.border_radius);
    let icon_color = generate_color_or(&fields.icon_color, quote! { iced::Color::BLACK });
    let placeholder_color = generate_color_or(
        &fields.placeholder_color,
        quote! { iced::Color { r: 0.4, g: 0.4, b: 0.4, a: 1.0 } },
    );
    let value_color = generate_color_or(&fields.value_color, quote! { iced::Color::BLACK });
    let selection_color = generate_color_or(
        &fields.selection_color,
        quote! { iced::Color { r: 0.0, g: 0.0, b: 1.0, a: 0.3 } },
    );

    quote! {
        iced::widget::text_input::Style {
            background: #background,
            border: #border,
            icon: #icon_color,
            placeholder: #placeholder_color,
            value: #value_color,
            selection: #selection_color,
        }
    }
}

pub fn merge_text_input_fields(
    base: &TextInputStyleFields,
    overlay: &TextInputStyleFields,
) -> TextInputStyleFields {
    TextInputStyleFields {
        background_color: overlay.background_color.or(base.background_color),
        border_color: overlay.border_color.or(base.border_color),
        border_width: overlay.border_width.or(base.border_width),
        border_radius: BorderRadius {
            top_left: overlay.border_radius.top_left.or(base.border_radius.top_left),
            top_right: overlay.border_radius.top_right.or(base.border_radius.top_right),
            bottom_right: overlay
                .border_radius
                .bottom_right
                .or(base.border_radius.bottom_right),
            bottom_left: overlay
                .border_radius
                .bottom_left
                .or(base.border_radius.bottom_left),
        },
        icon_color: overlay.icon_color.or(base.icon_color),
        placeholder_color: overlay.placeholder_color.or(base.placeholder_color),
        value_color: overlay.value_color.or(base.value_color),
        selection_color: overlay.selection_color.or(base.selection_color),
    }
}

pub fn generate_toggler_style_closure(s: &TogglerStyle) -> proc_macro2::TokenStream {
    let background = match &s.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { iced::Background::Color(#c) }
        }
        None => quote! { iced::Background::Color(iced::Color::TRANSPARENT) },
    };
    let background_border_width = s.background_border_width.unwrap_or(0.0);
    let background_border_color =
        generate_color_or(&s.background_border_color, quote! { iced::Color::TRANSPARENT });

    let foreground = match &s.foreground_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { iced::Background::Color(#c) }
        }
        None => quote! { iced::Background::Color(iced::Color::TRANSPARENT) },
    };
    let foreground_border_width = s.foreground_border_width.unwrap_or(0.0);
    let foreground_border_color =
        generate_color_or(&s.foreground_border_color, quote! { iced::Color::TRANSPARENT });

    let text_color = generate_option_color(&s.text_color);

    let border_radius = {
        let br = &s.border_radius;
        if br.top_left.is_none()
            && br.top_right.is_none()
            && br.bottom_right.is_none()
            && br.bottom_left.is_none()
        {
            quote! { None }
        } else {
            let tl = br.top_left.unwrap_or(0.0);
            let tr = br.top_right.unwrap_or(0.0);
            let brr = br.bottom_right.unwrap_or(0.0);
            let bl = br.bottom_left.unwrap_or(0.0);
            quote! {
                Some(iced::border::Radius { top_left: #tl, top_right: #tr, bottom_right: #brr, bottom_left: #bl })
            }
        }
    };

    let padding_ratio = s.padding_ratio.unwrap_or(0.05);

    quote! {
        |_theme, _status| iced::widget::toggler::Style {
            background: #background,
            background_border_width: #background_border_width,
            background_border_color: #background_border_color,
            foreground: #foreground,
            foreground_border_width: #foreground_border_width,
            foreground_border_color: #foreground_border_color,
            text_color: #text_color,
            border_radius: #border_radius,
            padding_ratio: #padding_ratio,
        }
    }
}

pub fn generate_overlay_menu_style_closure(s: &OverlayMenuStyle) -> proc_macro2::TokenStream {
    let background = match &s.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { iced::Background::Color(#c) }
        }
        None => quote! { iced::Background::Color(iced::Color::TRANSPARENT) },
    };

    let border = generate_border(&s.border_color, s.border_width, &s.border_radius);
    let text_color = generate_color_or(&s.text_color, quote! { iced::Color::BLACK });
    let selected_text_color =
        generate_color_or(&s.selected_text_color, quote! { iced::Color::WHITE });
    let selected_background = match &s.selected_background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { iced::Background::Color(#c) }
        }
        None => quote! { iced::Background::Color(iced::Color::TRANSPARENT) },
    };
    let shadow = generate_shadow(
        &s.shadow_color,
        s.shadow_offset_x,
        s.shadow_offset_y,
        s.shadow_blur_radius,
    );

    quote! {
        |_theme| iced::overlay::menu::Style {
            background: #background,
            border: #border,
            text_color: #text_color,
            selected_text_color: #selected_text_color,
            selected_background: #selected_background,
            shadow: #shadow,
        }
    }
}

pub fn generate_text_editor_fields_tokens(
    fields: &TextEditorStyleFields,
) -> proc_macro2::TokenStream {
    let background = match &fields.background_color {
        Some(c) => {
            let c = generate_color(c);
            quote! { iced::Background::Color(#c) }
        }
        None => quote! { iced::Background::Color(iced::Color::TRANSPARENT) },
    };

    let border = generate_border(&fields.border_color, fields.border_width, &fields.border_radius);
    let placeholder_color = generate_color_or(
        &fields.placeholder_color,
        quote! { iced::Color { r: 0.4, g: 0.4, b: 0.4, a: 1.0 } },
    );
    let value_color = generate_color_or(&fields.value_color, quote! { iced::Color::BLACK });
    let selection_color = generate_color_or(
        &fields.selection_color,
        quote! { iced::Color { r: 0.0, g: 0.0, b: 1.0, a: 0.3 } },
    );

    quote! {
        iced::widget::text_editor::Style {
            background: #background,
            border: #border,
            placeholder: #placeholder_color,
            value: #value_color,
            selection: #selection_color,
        }
    }
}

pub fn merge_text_editor_fields(
    base: &TextEditorStyleFields,
    overlay: &TextEditorStyleFields,
) -> TextEditorStyleFields {
    TextEditorStyleFields {
        background_color: overlay.background_color.or(base.background_color),
        border_color: overlay.border_color.or(base.border_color),
        border_width: overlay.border_width.or(base.border_width),
        border_radius: BorderRadius {
            top_left: overlay.border_radius.top_left.or(base.border_radius.top_left),
            top_right: overlay.border_radius.top_right.or(base.border_radius.top_right),
            bottom_right: overlay
                .border_radius
                .bottom_right
                .or(base.border_radius.bottom_right),
            bottom_left: overlay
                .border_radius
                .bottom_left
                .or(base.border_radius.bottom_left),
        },
        placeholder_color: overlay.placeholder_color.or(base.placeholder_color),
        value_color: overlay.value_color.or(base.value_color),
        selection_color: overlay.selection_color.or(base.selection_color),
    }
}

pub fn generate_text_editor_style_closure(tes: &TextEditorStyle) -> proc_macro2::TokenStream {
    let base_style = generate_text_editor_fields_tokens(&tes.base);

    let status_overrides: Vec<_> = [
        (
            &tes.active,
            quote! { iced::widget::text_editor::Status::Active },
        ),
        (
            &tes.hovered,
            quote! { iced::widget::text_editor::Status::Hovered },
        ),
        (
            &tes.disabled,
            quote! { iced::widget::text_editor::Status::Disabled },
        ),
    ]
    .into_iter()
    .filter_map(|(opt, status_token)| {
        opt.as_ref().map(|fields| {
            let merged = merge_text_editor_fields(&tes.base, fields);
            let style = generate_text_editor_fields_tokens(&merged);
            quote! { #status_token => #style }
        })
    })
    .collect();

    if status_overrides.is_empty() {
        quote! {
            |_theme, _status| #base_style
        }
    } else {
        quote! {
            |_theme, status| match status {
                #(#status_overrides,)*
                _ => #base_style
            }
        }
    }
}

pub fn generate_text_input_style_closure(tis: &TextInputStyle) -> proc_macro2::TokenStream {
    let base_style = generate_text_input_fields_tokens(&tis.base);

    let status_overrides: Vec<_> = [
        (
            &tis.active,
            quote! { iced::widget::text_input::Status::Active },
        ),
        (
            &tis.hovered,
            quote! { iced::widget::text_input::Status::Hovered },
        ),
        (
            &tis.disabled,
            quote! { iced::widget::text_input::Status::Disabled },
        ),
    ]
    .into_iter()
    .filter_map(|(opt, status_token)| {
        opt.as_ref().map(|fields| {
            let merged = merge_text_input_fields(&tis.base, fields);
            let style = generate_text_input_fields_tokens(&merged);
            quote! { #status_token => #style }
        })
    })
    .collect();

    if status_overrides.is_empty() {
        quote! {
            |_theme, _status| #base_style
        }
    } else {
        quote! {
            |_theme, status| match status {
                #(#status_overrides,)*
                _ => #base_style
            }
        }
    }
}
