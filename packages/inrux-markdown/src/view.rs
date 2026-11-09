//! Shared Nodes preserve the same layout, styling and fonts on every engine.
use crate::Block;
use inrux_components::Node;
use inrux_style::{Align, Overflow, Size, Style, WhiteSpace};
/// Render the immutable block sequence without parsing during a native frame.
pub(crate) fn document(blocks: &[Block]) -> Node {
    Node::column(blocks.iter().map(render_block).collect())
        .component_type("Markdown")
        .classes("markdown")
        .with_defaults(Style::new().gap(12.0))
}
/// Use ordinary components so styles and accessibility remain under the framework contract.
fn render_block(block: &Block) -> Node {
    match block {
        Block::Paragraph(text) => Node::text(text.clone()).classes("md-paragraph"),
        Block::Heading(level, text) => Node::text(text.clone())
            .classes(&format!("md-heading md-h{level}"))
            .with_defaults(Style::new().font_size(match level {
                1 => 28.0,
                2 => 22.0,
                _ => 17.0,
            })),
        Block::Code(text) => Node::scroll_area(vec![
            Node::text(text.clone())
                .classes("md-code-text")
                .with_defaults(
                    Style::new()
                        .font_size(13.0)
                        .white_space(WhiteSpace::Pre)
                        .width(Size::Content),
                ),
        ])
        .classes("md-code")
        .with_defaults(
            Style::new()
                .height(Size::Content)
                .align_items(Align::Start)
                .overflow_x(Overflow::Auto)
                .overflow_y(Overflow::Hidden),
        ),
        Block::Quote(children) => Node::column(vec![document(children)])
            .classes("md-quote")
            .with_defaults(Style::new().padding(12.0)),
        Block::List(start, children) => Node::column(
            children
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    Node::row(vec![
                        Node::text(start.map_or_else(
                            || "•".into(),
                            |start| format!("{}.", start.saturating_add(index as u64)),
                        ))
                        .with_defaults(Style::new().width(Size::Px(28.0))),
                        render_block(item)
                            .with_defaults(Style::new().width(Size::Px(0.0)).flex_grow(1.0)),
                    ])
                    .classes("md-list-item")
                })
                .collect(),
        )
        .classes("md-list")
        .with_defaults(Style::new().gap(8.0)),
        Block::Item(children) => document(children),
        Block::Rule => Node::separator(false),
    }
}
