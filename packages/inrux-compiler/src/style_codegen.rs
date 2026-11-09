//! Translate the shared style representation into Rust construction code.
use inrux_style::{Color, Rule, Size, State, Style};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Emit an sRGB value without parsing colors at runtime.
fn color_tokens(c: Color) -> TokenStream {
    let (r, g, b, a) = (c.0, c.1, c.2, c.3);
    if a == 255 {
        quote!(::inrux::Color(#r,#g,#b))
    } else {
        quote!(::inrux::Color::rgba(#r,#g,#b,#a))
    }
}
/// Preserve the typed size variant in generated Rust.
fn size_tokens(size: Size) -> TokenStream {
    match size {
        Size::Px(n) => quote!(::inrux::Size::Px(#n)),
        Size::Percent(n) => quote!(::inrux::Size::Percent(#n)),
        Size::Fill => quote!(::inrux::Size::Fill),
        Size::Content => quote!(::inrux::Size::Content),
    }
}
/// Emit only explicitly set style properties.
fn style_tokens(style: &Style) -> TokenStream {
    let mut result = quote!(::inrux::Style::new());
    if let Some((thumb, track)) = style.scrollbar_colors {
        let thumb = color_tokens(thumb);
        let track = color_tokens(track);
        result = quote!(#result.scrollbar_colors(#thumb,#track));
    }
    if let Some(family) = &style.font_family {
        result = quote!(#result.font_family(#family));
    }
    for (name, value) in [
        ("gap", style.gap),
        ("row_gap", style.row_gap),
        ("column_gap", style.column_gap),
        ("padding", style.padding),
        ("font_size", style.font_size),
        ("rounded", style.rounded),
        ("border_width", style.border_width),
        ("flex_grow", style.flex_grow),
        ("aspect_ratio", style.aspect_ratio),
    ] {
        if let Some(value) = value {
            let method = format_ident!("{name}");
            result = quote!(#result.#method(#value));
        }
    }
    for (name, value) in [
        ("background", style.background),
        ("accent_color", style.accent_color),
        ("color", style.color),
        ("border_color", style.border_color),
    ] {
        if let Some(value) = value {
            let method = format_ident!("{name}");
            let value = color_tokens(value);
            result = quote!(#result.#method(#value));
        }
    }
    for (name, value) in [
        ("top", style.top),
        ("right", style.right),
        ("bottom", style.bottom),
        ("left", style.left),
        ("width", style.width),
        ("height", style.height),
        ("min_width", style.min_width),
        ("max_width", style.max_width),
        ("min_height", style.min_height),
        ("max_height", style.max_height),
    ] {
        if let Some(value) = value {
            let method = format_ident!("{name}");
            let value = size_tokens(value);
            result = quote!(#result.#method(#value));
        }
    }
    if let Some(value) = style.margin_inline_auto {
        result = quote!(#result.margin_inline_auto(#value));
    }
    for (name, value) in [
        ("margin_top", style.margin_top),
        ("margin_right", style.margin_right),
        ("margin_bottom", style.margin_bottom),
        ("margin_left", style.margin_left),
    ] {
        if let Some(value) = value {
            let method = format_ident!("{name}");
            result = quote!(#result.#method(#value));
        }
    }
    if let Some(value) = style.grid_columns {
        result = quote!(#result.grid_columns(#value));
    }
    if let Some(value) = style.white_space {
        let variant = format_ident!("{value:?}");
        result = quote!(#result.white_space(::inrux::WhiteSpace::#variant));
    }
    if let Some(value) = style.position {
        let variant = format_ident!("{value:?}");
        result = quote!(#result.position(::inrux::Position::#variant));
    }
    if let Some(value) = style.display {
        let variant = format_ident!("{value:?}");
        result = quote!(#result.display(::inrux::Display::#variant));
    }
    if let Some(value) = style.flex_direction {
        let variant = format_ident!("{value:?}");
        result = quote!(#result.flex_direction(::inrux::FlexDirection::#variant));
    }
    if let Some(value) = style.wrap {
        result = quote!(#result.wrap(#value));
    }
    for (name, value) in [
        ("overflow_x", style.overflow_x),
        ("overflow_y", style.overflow_y),
    ] {
        if let Some(value) = value {
            let method = format_ident!("{name}");
            let variant = format_ident!("{value:?}");
            result = quote!(#result.#method(::inrux::Overflow::#variant));
        }
    }
    if let Some(value) = style.align_items {
        let variant = format_ident!("{value:?}");
        result = quote!(#result.align_items(::inrux::Align::#variant));
    }
    if let Some(value) = style.justify_content {
        let variant = format_ident!("{value:?}");
        result = quote!(#result.justify_content(::inrux::Justify::#variant));
    }
    result
}
/// Preserve selector scope and interaction state in a compiled rule.
pub(crate) fn rule_tokens(rule: &Rule) -> TokenStream {
    let kind = option_string(&rule.selector.kind);
    let id = option_string(&rule.selector.id);
    let classes = &rule.selector.classes;
    let part = match rule.selector.part {
        Some(part) => {
            let part = format_ident!("{part:?}");
            quote!(Some(::inrux::Part::#part))
        }
        None => quote!(None),
    };
    let bounds = [
        rule.media.min_width,
        rule.media.max_width,
        rule.media.min_height,
        rule.media.max_height,
    ]
    .map(|value| match value {
        Some(n) => quote!(Some(#n)),
        None => quote!(None),
    });
    let [min_width, max_width, min_height, max_height] = bounds;
    let state = format_ident!(
        "{}",
        match rule.state {
            State::Base => "Base",
            State::Hover => "Hover",
            State::Focus => "Focus",
            State::Disabled => "Disabled",
            State::Checked => "Checked",
            State::Unchecked => "Unchecked",
        }
    );
    let style = style_tokens(&rule.style);
    quote!(::inrux::Rule {
        selector: ::inrux::Selector { kind: #kind, id: #id, classes: vec![#(#classes.into()),*], part: #part },
        media: ::inrux::Media { min_width: #min_width, max_width: #max_width, min_height: #min_height, max_height: #max_height },
        state: ::inrux::StyleState::#state, style: #style
    })
}

/// Emit optional owned selector metadata without runtime parsing.
fn option_string(value: &Option<String>) -> TokenStream {
    match value {
        Some(value) => quote!(Some(#value.into())),
        None => quote!(None),
    }
}
