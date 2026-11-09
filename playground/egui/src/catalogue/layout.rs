//! Focused geometry specimens demonstrate one physical layout component at a time.
use super::{Example, controls};
use crate::views::component_preview::State;
use inrux::{Color, Events, FlexDirection, Node, Size, Style};
/// Changing the boolean demonstrates layout updates without changing component identity.
pub fn preview(state: &State, events: &Events<State>) -> Node {
    use Example::*;
    let items = || vec![tile("Alpha"), tile("Beta"), tile("Gamma")];
    let specimen = match state.props.example {
        Row => Node::row(items()).styled(
            Style::new()
                .gap(if state.checked { 24.0 } else { 8.0 })
                .wrap(state.checked),
        ),
        Column => {
            Node::column(items()).styled(Style::new().gap(if state.checked { 24.0 } else { 8.0 }))
        }
        Center => Node::center(vec![tile("Centered on both axes")])
            .styled(surface().height(Size::Px(if state.checked { 220.0 } else { 150.0 }))),
        Spacer => Node::row(vec![
            tile("Start"),
            Node::spacer().styled(Style::new().flex_grow(if state.checked { 2.0 } else { 1.0 })),
            tile("Middle"),
            Node::spacer(),
            tile("End"),
        ])
        .styled(surface().height(Size::Px(80.0))),
        Flex => Node::flex(items()).styled(surface().gap(12.0).flex_direction(if state.checked {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        })),
        Grid => Node::grid(vec![
            tile("Alpha"),
            tile("Beta"),
            tile("Gamma"),
            tile("Delta"),
            tile("Epsilon"),
            tile("Zeta"),
        ])
        .styled(
            surface()
                .grid_columns(if state.checked { 2 } else { 3 })
                .row_gap(if state.checked { 8.0 } else { 20.0 })
                .column_gap(12.0),
        ),
        AspectRatio => Node::aspect_ratio(vec![
            Node::center(vec![Node::text(
                if state.checked { "1 : 1" } else { "16 : 9" }.into(),
            )])
            .styled(Style::new().height(Size::Fill)),
        ])
        .styled(
            surface()
                .width(Size::Px(240.0))
                .aspect_ratio(if state.checked { 1.0 } else { 16.0 / 9.0 }),
        ),
        ScrollArea => Node::scroll_area(
            (1..=12)
                .map(|row| {
                    Node::button(
                        format!("Preview row {row}"),
                        Some(events.handler(super::increment)),
                    )
                    .styled(controls::secondary())
                })
                .collect(),
        )
        .styled(
            surface()
                .height(Size::Px(if state.checked { 240.0 } else { 180.0 }))
                .gap(8.0),
        ),
        Container => Node::box_container(vec![
            Node::container(vec![tile("Centered, width-constrained content")])
                .styled(surface().max_width(Size::Px(if state.checked { 240.0 } else { 420.0 }))),
        ]),
        _ => Node::box_container(vec![
            tile("Contained content"),
            tile("Ordinary child nodes"),
        ])
        .styled(
            surface()
                .padding(if state.checked { 28.0 } else { 12.0 })
                .gap(12.0),
        ),
    };
    Node::column(vec![
        specimen,
        Node::checkbox(
            "Use alternate layout".into(),
            state.checked,
            events.model(|state, value, _| state.checked = value),
        ),
        Node::text(format!(
            "Alternate: {} · Child actions: {}",
            state.checked, state.count
        )),
    ])
    .styled(Style::new().gap(16.0))
}
/// A readable child surface makes alignment and spacing visible without fake data.
fn tile(label: &str) -> Node {
    Node::text(label.into()).styled(
        Style::new()
            .padding(10.0)
            .background(Color(225, 235, 228))
            .rounded(4.0),
    )
}
/// Bound the demonstration surface separately from the dashboard canvas.
fn surface() -> Style {
    Style::new()
        .width(Size::Fill)
        .padding(12.0)
        .background(Color(240, 245, 242))
        .rounded(5.0)
}
