//! Taffy geometry and intrinsic content measurement.

use inrux_components::{Node, NodeKind as Kind};

use inrux_style::{Align, Display, FlexDirection, Justify, Size};

use taffy::prelude as tf;
/// Translate logical sizing into the shared Taffy layout representation.
fn dimension(size: Option<Size>) -> tf::Dimension {
    match size {
        Some(Size::Px(n)) => tf::length(n),
        Some(Size::Percent(n)) => tf::percent(n / 100.0),
        Some(Size::Fill) => tf::percent(1.0_f32),
        _ => tf::auto(),
    }
}
pub(super) struct Measure {
    pub(super) text: String,
    pub(super) font_size: f32,
    pub(super) font_family: Option<String>,
    pub(super) control: bool,
    pub(super) control_extra: f32,
    pub(super) input: bool,
    pub(super) preformatted: bool,
}
pub(super) struct LayoutNode {
    pub(super) id: tf::NodeId,
    pub(super) children: Vec<LayoutNode>,
}
/// Build geometry descriptors without borrowing mutable component state.
pub(super) fn build_layout(
    node: &Node,
    tree: &mut tf::TaffyTree<Measure>,
) -> Result<LayoutNode, taffy::TaffyError> {
    build(node, tree, false)
}
/// Scrollable parents measure natural children rather than shrinking their content to the viewport.
fn build(
    node: &Node,
    tree: &mut tf::TaffyTree<Measure>,
    scroll_child: bool,
) -> Result<LayoutNode, taffy::TaffyError> {
    let style = node.style();
    let toggle = matches!(node.kind, Kind::Checkbox { .. })
        .then(|| crate::control_parts::toggle(node, &style, false, false));
    let axes = style.overflow_axes();
    let overflow = |value| match value {
        inrux_style::Overflow::Visible => taffy::style::Overflow::Visible,
        inrux_style::Overflow::Hidden => taffy::style::Overflow::Hidden,
        _ => taffy::style::Overflow::Scroll,
    };
    let container = matches!(node.kind, Kind::Column | Kind::Row);
    let default_height = match node.kind {
        Kind::Input {
            multiline: true, ..
        } => Some(Size::Px(120.0)),
        Kind::Button(..) | Kind::Input { .. } | Kind::Checkbox { .. } | Kind::Select { .. } => {
            Some(Size::Px(40.0))
        }
        Kind::Heartbeat => Some(Size::Px(66.0)),
        _ => None,
    };
    let inset = |size| match size {
        Some(Size::Px(n)) => tf::length(n),
        Some(Size::Percent(n)) => tf::percent(n / 100.0),
        Some(Size::Fill) => tf::percent(1.0_f32),
        _ => tf::auto(),
    };
    let taffy_style = tf::Style {
        position: if style.position == Some(inrux_style::Position::Absolute) {
            tf::Position::Absolute
        } else {
            tf::Position::Relative
        },
        inset: tf::Rect {
            top: inset(style.top),
            right: inset(style.right),
            bottom: inset(style.bottom),
            left: inset(style.left),
        },
        overflow: taffy::geometry::Point {
            // Native editors own their internal text viewport; long text must not
            // contribute overflow to the containing application ScrollArea.
            x: if matches!(node.kind, Kind::Input { .. }) {
                taffy::style::Overflow::Hidden
            } else {
                overflow(axes.0)
            },
            y: if matches!(node.kind, Kind::Input { .. }) {
                taffy::style::Overflow::Hidden
            } else {
                overflow(axes.1)
            },
        },
        display: if matches!(node.kind, Kind::Empty | Kind::Dialog { .. }) {
            tf::Display::None
        } else if style.display == Some(Display::Grid) {
            tf::Display::Grid
        } else {
            tf::Display::Flex
        },
        flex_direction: match style
            .flex_direction
            .unwrap_or(if matches!(node.kind, Kind::Row) {
                FlexDirection::Row
            } else {
                FlexDirection::Column
            }) {
            FlexDirection::Row => tf::FlexDirection::Row,
            FlexDirection::Column => tf::FlexDirection::Column,
        },
        grid_template_columns: if style.display == Some(Display::Grid) {
            vec![
                tf::minmax(tf::length(0.0_f32), tf::fr(1.0_f32));
                usize::from(style.grid_columns.unwrap_or(2))
            ]
        } else {
            vec![]
        },
        aspect_ratio: style.aspect_ratio,
        size: tf::Size {
            width: dimension(
                style
                    .width
                    .or(if container { Some(Size::Fill) } else { None }),
            ),
            height: dimension(style.height.or(default_height)),
        },
        min_size: tf::Size {
            width: style
                .min_width
                .map_or(tf::length(0.0_f32), |size| dimension(Some(size))),
            height: dimension(style.min_height),
        },
        max_size: tf::Size {
            width: dimension(style.max_width),
            height: dimension(style.max_height),
        },
        align_items: style.align_items.map(|value| match value {
            Align::Start => tf::AlignItems::FlexStart,
            Align::Center => tf::AlignItems::Center,
            Align::End => tf::AlignItems::FlexEnd,
            Align::Stretch => tf::AlignItems::Stretch,
        }),
        justify_content: style.justify_content.map(|value| match value {
            Justify::Start => tf::JustifyContent::FlexStart,
            Justify::Center => tf::JustifyContent::Center,
            Justify::End => tf::JustifyContent::FlexEnd,
            Justify::SpaceBetween => tf::JustifyContent::SpaceBetween,
            Justify::SpaceAround => tf::JustifyContent::SpaceAround,
            Justify::SpaceEvenly => tf::JustifyContent::SpaceEvenly,
        }),
        flex_wrap: if style.wrap.unwrap_or(false) {
            tf::FlexWrap::Wrap
        } else {
            tf::FlexWrap::NoWrap
        },
        flex_grow: style.flex_grow.unwrap_or(0.0),
        margin: tf::Rect {
            left: style.margin_left.map_or_else(
                || {
                    if style.margin_inline_auto.unwrap_or(false) {
                        tf::auto()
                    } else {
                        tf::length(0.0_f32)
                    }
                },
                tf::length,
            ),
            right: style.margin_right.map_or_else(
                || {
                    if style.margin_inline_auto.unwrap_or(false) {
                        tf::auto()
                    } else {
                        tf::length(0.0_f32)
                    }
                },
                tf::length,
            ),
            top: tf::length(style.margin_top.unwrap_or(0.0)),
            bottom: tf::length(style.margin_bottom.unwrap_or(0.0)),
        },
        flex_shrink: if container && !scroll_child { 1.0 } else { 0.0 },
        gap: tf::Size {
            width: tf::length(style.column_gap.or(style.gap).unwrap_or(0.0)),
            height: tf::length(style.row_gap.or(style.gap).unwrap_or(0.0)),
        },
        padding: tf::Rect {
            left: tf::length(style.padding.unwrap_or(0.0)),
            right: tf::length(style.padding.unwrap_or(0.0)),
            top: tf::length(style.padding.unwrap_or(0.0)),
            bottom: tf::length(style.padding.unwrap_or(0.0)),
        },
        ..Default::default()
    };
    let children = node
        .children
        .iter()
        .map(|child| build(child, tree, axes.0.scrollable() || axes.1.scrollable()))
        .collect::<Result<Vec<_>, _>>()?;
    let id = if container {
        let mut native_children = children.iter().map(|child| child.id).collect::<Vec<_>>();
        if native_children.is_empty() && style.aspect_ratio.is_some() {
            // Taffy 0.9 classifies empty containers as leaves; its leaf ratio path can override
            // explicit/max heights. A zero-size private child selects the container algorithm.
            // It has no framework identity, paint, event target or published geometry entry.
            native_children.push(tree.new_leaf(tf::Style {
                size: tf::Size {
                    width: tf::length(0.0_f32),
                    height: tf::length(0.0_f32),
                },
                ..Default::default()
            })?);
        }
        tree.new_with_children(taffy_style, &native_children)?
    } else if matches!(node.kind, Kind::Range(_)) {
        // Range indicators have explicit geometry and no intrinsic text to shape.
        tree.new_leaf(taffy_style)?
    } else {
        let text = match &node.kind {
            Kind::Text(text) | Kind::Button(text, _) => text.clone(),
            Kind::Input { value, .. } | Kind::Select { value, .. } => value.clone(),
            Kind::Checkbox { label, .. } => label.clone(),
            _ => String::new(),
        };
        tree.new_leaf_with_context(
            taffy_style,
            Measure {
                text,
                font_size: toggle
                    .as_ref()
                    .map_or(&style, |v| &v.label)
                    .font_size
                    .unwrap_or(15.0),
                font_family: toggle
                    .as_ref()
                    .map_or(&style, |v| &v.label)
                    .font_family
                    .clone(),
                control_extra: toggle.as_ref().map_or(32.0, |v| v.width + v.gap + 10.0),
                control: matches!(
                    node.kind,
                    Kind::Button(..)
                        | Kind::Input { .. }
                        | Kind::Checkbox { .. }
                        | Kind::Select { .. }
                ),
                input: matches!(node.kind, Kind::Input { .. }),
                preformatted: matches!(node.kind, Kind::Text(_))
                    && style.white_space == Some(inrux_style::WhiteSpace::Pre),
            },
        )?
    };
    Ok(LayoutNode { id, children })
}
