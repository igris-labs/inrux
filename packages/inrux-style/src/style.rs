//! Typed style construction and property-level validation.
use crate::values::{length, size};
use crate::{Align, Color, Display, FlexDirection, Justify, Overflow, Size, StyleError};
/// Unset properties preserve the preceding layer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    /// Inherited wrapping policy for text; code blocks use Pre.
    pub white_space: Option<crate::WhiteSpace>,
    /// Flow participation and parent-relative inset placement.
    pub position: Option<crate::Position>,
    /// Top inset in logical pixels or a parent-relative percentage.
    pub top: Option<Size>,
    /// Right inset in logical pixels or a parent-relative percentage.
    pub right: Option<Size>,
    /// Bottom inset in logical pixels or a parent-relative percentage.
    pub bottom: Option<Size>,
    /// Left inset in logical pixels or a parent-relative percentage.
    pub left: Option<Size>,
    /// Horizontal overflow; unset resolves to visible before axis normalization.
    pub overflow_x: Option<Overflow>,
    /// Vertical overflow; unset resolves to visible before axis normalization.
    pub overflow_y: Option<Overflow>,
    /// Container layout algorithm; the primitive determines the default.
    pub display: Option<Display>,
    /// Flexible main axis; the primitive determines the default.
    pub flex_direction: Option<FlexDirection>,
    /// Uniform grid column count, from 1 through MAX_GRID_COLUMNS.
    pub grid_columns: Option<u16>,
    /// Preferred width divided by height; must be finite and positive.
    pub aspect_ratio: Option<f32>,
    /// Spacing between children in logical pixels.
    pub gap: Option<f32>,
    /// Vertical spacing between rows, overriding gap on this axis.
    pub row_gap: Option<f32>,
    /// Horizontal spacing between columns, overriding gap on this axis.
    pub column_gap: Option<f32>,
    /// Inherited accent for selection marks, switches and range fills.
    pub accent_color: Option<Color>,
    /// Uniform interior spacing in logical pixels.
    pub padding: Option<f32>,
    /// Horizontal sizing policy.
    pub width: Option<Size>,
    /// Vertical sizing policy.
    pub height: Option<Size>,
    /// Minimum width constraint.
    pub min_width: Option<Size>,
    /// Maximum width constraint.
    pub max_width: Option<Size>,
    /// Minimum height constraint.
    pub min_height: Option<Size>,
    /// Maximum height constraint.
    pub max_height: Option<Size>,
    /// Cross-axis flex alignment.
    pub align_items: Option<Align>,
    /// Main-axis flex distribution.
    pub justify_content: Option<Justify>,
    /// Whether children can wrap to additional flex lines.
    pub wrap: Option<bool>,
    /// Unitless share of remaining main-axis space.
    pub flex_grow: Option<f32>,
    /// Automatic physical horizontal margins; false resolves to zero margins.
    pub margin_inline_auto: Option<bool>,
    /// Signed physical top margin in logical pixels.
    pub margin_top: Option<f32>,
    /// Signed physical right margin in logical pixels.
    pub margin_right: Option<f32>,
    /// Signed physical bottom margin in logical pixels.
    pub margin_bottom: Option<f32>,
    /// Signed physical left margin in logical pixels.
    pub margin_left: Option<f32>,
    /// Inherited scrollbar thumb and track colors, respectively.
    pub scrollbar_colors: Option<(Color, Color)>,
    /// Background fill.
    pub background: Option<Color>,
    /// Inherited text color.
    pub color: Option<Color>,
    /// Inherited proportional font size in logical pixels.
    pub font_size: Option<f32>,
    /// Inherited font family, registered through application window options.
    pub font_family: Option<String>,
    /// Corner radius in logical pixels.
    pub rounded: Option<f32>,
    /// Painted border color; does not reserve layout space.
    pub border_color: Option<Color>,
    /// Painted border thickness in logical pixels.
    pub border_width: Option<f32>,
}
macro_rules! setters {
    ($($field:ident: $ty:ty),* $(,)?) => { $(
        /// Set this property, preserving all other properties.
        pub fn $field(mut self, value: $ty) -> Self { self.$field = Some(value); self }
    )* };
}
impl Style {
    /// Whether a conditional layer can change paint without invalidating layout or inherited metrics.
    pub fn is_paint_only(&self) -> bool {
        let mut paint = Self::new();
        paint.background = self.background;
        paint.color = self.color;
        paint.rounded = self.rounded;
        paint.border_color = self.border_color;
        paint.border_width = self.border_width;
        paint == *self
    }

    /// Set both physical overflow axes.
    pub fn overflow(mut self, value: Overflow) -> Self {
        self.overflow_x = Some(value);
        self.overflow_y = Some(value);
        self
    }
    /// Resolve the CSS rule that visible paired with non-visible overflow computes to auto.
    pub fn overflow_axes(&self) -> (Overflow, Overflow) {
        let (mut x, mut y) = (
            self.overflow_x.unwrap_or_default(),
            self.overflow_y.unwrap_or_default(),
        );
        if x == Overflow::Visible && y != Overflow::Visible {
            x = Overflow::Auto;
        }
        if y == Overflow::Visible && x != Overflow::Visible {
            y = Overflow::Auto;
        }
        (x, y)
    }
    /// Start a style with no overrides.
    pub fn new() -> Self {
        Self::default()
    }
    setters!(row_gap: f32, column_gap: f32, accent_color: Color, padding: f32, width: Size, height: Size, background: Color, color: Color, font_size: f32, rounded: f32, border_color: Color, border_width: f32);
    setters!(min_width: Size, max_width: Size, min_height: Size, max_height: Size, align_items: Align, justify_content: Justify, wrap: bool, flex_grow: f32);
    setters!(display: Display, flex_direction: FlexDirection, grid_columns: u16, aspect_ratio: f32);
    setters!(overflow_x: Overflow, overflow_y: Overflow);
    setters!(margin_top:f32,margin_right:f32,margin_bottom:f32,margin_left:f32);
    /// Set all physical margins, resetting horizontal auto margins.
    pub fn margin(mut self, value: f32) -> Self {
        self.margin_top = Some(value);
        self.margin_right = Some(value);
        self.margin_bottom = Some(value);
        self.margin_left = Some(value);
        self.margin_inline_auto = Some(false);
        self
    }
    /// Set both horizontal margins to auto/zero, resetting preceding physical horizontal values.
    pub fn margin_inline_auto(mut self, value: bool) -> Self {
        self.margin_inline_auto = Some(value);
        self.margin_left = None;
        self.margin_right = None;
        self
    }
    /// Choose a registered font family.
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }
    /// Set the inherited thumb and track palette without changing scrollbar geometry.
    pub fn scrollbar_colors(mut self, thumb: Color, track: Color) -> Self {
        self.scrollbar_colors = Some((thumb, track));
        self
    }
    /// Resolve the authored palette or the framework's existing default colors.
    pub fn resolved_scrollbar_colors(&self) -> (Color, Color) {
        self.scrollbar_colors
            .unwrap_or((Color(126, 148, 137), Color(232, 237, 234)))
    }
    /// Set both gap axes; later row_gap/column_gap setters can override one axis.
    pub fn gap(mut self, value: f32) -> Self {
        self.gap = Some(value);
        self.row_gap = Some(value);
        self.column_gap = Some(value);
        self
    }
    /// Set inherited native text wrapping without changing the stored source text.
    pub fn white_space(mut self, mode: crate::WhiteSpace) -> Self {
        self.white_space = Some(mode);
        self
    }
    /// Choose flow participation; fixed document rails use an absolute sibling outside the scroller.
    pub fn position(mut self, position: crate::Position) -> Self {
        self.position = Some(position);
        self
    }
    setters!(top: Size, right: Size, bottom: Size, left: Size);
    /// Overlay only explicitly set properties.
    pub fn overlay(&mut self, other: &Self) {
        if other.font_family.is_some() {
            self.font_family.clone_from(&other.font_family);
        }
        if other.margin_inline_auto.is_some() {
            self.margin_left = None;
            self.margin_right = None;
        }
        if let Some(gap) = other.gap {
            self.row_gap = Some(gap);
            self.column_gap = Some(gap);
        }
        macro_rules! merge { ($($field:ident),*) => { $(if other.$field.is_some() { self.$field = other.$field; })* }; }
        merge!(
            white_space,
            position,
            top,
            right,
            bottom,
            left,
            overflow_x,
            overflow_y,
            display,
            flex_direction,
            grid_columns,
            aspect_ratio,
            gap,
            row_gap,
            column_gap,
            accent_color,
            padding,
            width,
            height,
            scrollbar_colors,
            background,
            color,
            font_size,
            rounded,
            border_color,
            border_width,
            min_width,
            max_width,
            min_height,
            max_height,
            align_items,
            justify_content,
            wrap,
            flex_grow,
            margin_inline_auto,
            margin_top,
            margin_right,
            margin_bottom,
            margin_left
        );
    }
    /// Compile a supported declaration into the same typed style as the builder API.
    pub fn declaration(&mut self, property: &str, value: &str) -> Result<(), StyleError> {
        match property {
            "overflow" | "overflow-x" | "overflow-y" => {
                let value = match value {
                    "visible" => Overflow::Visible,
                    "hidden" => Overflow::Hidden,
                    "auto" => Overflow::Auto,
                    "scroll" => Overflow::Scroll,
                    _ => {
                        return Err(StyleError(
                            "overflow supports visible, hidden, auto or scroll".into(),
                        ));
                    }
                };
                if property != "overflow-y" {
                    self.overflow_x = Some(value);
                }
                if property != "overflow-x" {
                    self.overflow_y = Some(value);
                }
            }
            "white-space" => {
                self.white_space = Some(match value {
                    "normal" => crate::WhiteSpace::Normal,
                    "pre" => crate::WhiteSpace::Pre,
                    _ => return Err(StyleError("white-space supports normal or pre".into())),
                });
            }
            "position" => {
                self.position = Some(match value {
                    "relative" => crate::Position::Relative,
                    "absolute" => crate::Position::Absolute,
                    _ => return Err(StyleError("position supports relative or absolute".into())),
                });
            }
            "top" => self.top = Some(size(value)?),
            "right" => self.right = Some(size(value)?),
            "bottom" => self.bottom = Some(size(value)?),
            "left" => self.left = Some(size(value)?),
            "display" => {
                self.display = Some(match value {
                    "flex" => Display::Flex,
                    "grid" => Display::Grid,
                    _ => return Err(StyleError("display supports flex or grid".into())),
                })
            }
            "flex-direction" => {
                self.flex_direction = Some(match value {
                    "row" => FlexDirection::Row,
                    "column" => FlexDirection::Column,
                    _ => return Err(StyleError("flex-direction supports row or column".into())),
                })
            }
            "grid-template-columns" => {
                self.grid_columns = Some(crate::values::grid_columns(value)?)
            }
            "aspect-ratio" => self.aspect_ratio = Some(crate::values::aspect_ratio(value)?),
            "gap" => {
                let values = value.split_whitespace().collect::<Vec<_>>();
                if values.is_empty() || values.len() > 2 {
                    return Err(StyleError(
                        "gap expects one or two nonnegative lengths".into(),
                    ));
                }
                let row = length(values[0])?;
                let column = length(values.get(1).copied().unwrap_or(values[0]))?;
                self.gap = Some(row);
                self.row_gap = Some(row);
                self.column_gap = Some(column);
            }
            "row-gap" => self.row_gap = Some(length(value)?),
            "column-gap" => self.column_gap = Some(length(value)?),
            "accent-color" => self.accent_color = Some(Color::parse(value)?),
            "padding" => self.padding = Some(length(value)?),
            "width" => self.width = Some(size(value)?),
            "height" => self.height = Some(size(value)?),
            "min-width" => self.min_width = Some(size(value)?),
            "max-width" => self.max_width = Some(size(value)?),
            "min-height" => self.min_height = Some(size(value)?),
            "max-height" => self.max_height = Some(size(value)?),
            "align-items" => {
                self.align_items = Some(match value {
                    "start" | "flex-start" => Align::Start,
                    "end" | "flex-end" => Align::End,
                    "center" => Align::Center,
                    "stretch" => Align::Stretch,
                    _ => return Err(StyleError(format!("unsupported align-items `{value}`"))),
                })
            }
            "justify-content" => {
                self.justify_content = Some(match value {
                    "start" | "flex-start" => Justify::Start,
                    "end" | "flex-end" => Justify::End,
                    "center" => Justify::Center,
                    "space-between" => Justify::SpaceBetween,
                    "space-around" => Justify::SpaceAround,
                    "space-evenly" => Justify::SpaceEvenly,
                    _ => return Err(StyleError(format!("unsupported justify-content `{value}`"))),
                })
            }
            "flex-wrap" => {
                self.wrap = Some(match value {
                    "wrap" => true,
                    "nowrap" => false,
                    _ => return Err(StyleError(format!("unsupported flex-wrap `{value}`"))),
                })
            }
            "flex-grow" => {
                let number: f32 = value
                    .parse()
                    .map_err(|_| StyleError("flex-grow requires a unitless number".into()))?;
                if !number.is_finite() || number < 0.0 {
                    return Err(StyleError(
                        "flex-grow must be finite and nonnegative".into(),
                    ));
                }
                self.flex_grow = Some(number);
            }
            "margin" => {
                let [top, right, bottom, left] = crate::values::margin_edges(value)?;
                self.margin_top = Some(top);
                self.margin_right = Some(right);
                self.margin_bottom = Some(bottom);
                self.margin_left = Some(left);
                self.margin_inline_auto = Some(false);
            }
            "margin-top" => self.margin_top = Some(crate::values::signed_length(value)?),
            "margin-right" => self.margin_right = Some(crate::values::signed_length(value)?),
            "margin-bottom" => self.margin_bottom = Some(crate::values::signed_length(value)?),
            "margin-left" => self.margin_left = Some(crate::values::signed_length(value)?),
            "margin-inline" => {
                self.margin_left = None;
                self.margin_right = None;
                self.margin_inline_auto = Some(match value {
                    "auto" => true,
                    "0" | "0px" => false,
                    _ => return Err(StyleError("margin-inline supports auto or 0".into())),
                })
            }
            "background" | "background-color" => self.background = Some(Color::parse(value)?),
            "color" => self.color = Some(Color::parse(value)?),
            "font-family" => {
                let family = value.trim().trim_matches(['\'', '"']);
                if family.is_empty() || family.contains(',') {
                    return Err(StyleError(
                        "font-family requires one registered family".into(),
                    ));
                }
                self.font_family = Some(family.into());
            }
            "font-size" => self.font_size = Some(length(value)?),
            "border-radius" => self.rounded = Some(length(value)?),
            "scrollbar-color" => {
                let values = crate::syntax::words(value)?;
                if values.len() != 2 {
                    return Err(StyleError(
                        "scrollbar-color requires thumb and track literal CSS colors".into(),
                    ));
                }
                self.scrollbar_colors = Some((Color::parse(values[0])?, Color::parse(values[1])?));
            }
            "border-color" => self.border_color = Some(Color::parse(value)?),
            "border-width" => self.border_width = Some(length(value)?),
            _ => return Err(StyleError(format!("unsupported property `{property}`"))),
        }
        Ok(())
    }
}
