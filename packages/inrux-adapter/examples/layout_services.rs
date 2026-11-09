//! Headless layout specimen; its fixed text metrics are illustrative, not native shaping or a benchmark.
use inrux_adapter::{
    layout::TextMeasure,
    services::{
        ProviderInfo, ProviderKind,
        layout::{LayoutRequest, LayoutService, TaffyLayout},
        text::{TextService, TextSize},
    },
};
use inrux_components::Node;
use inrux_style::{Size, Style};
/// Resolve a small Row with the same selected fallback used by the native playgrounds.
fn main() -> Result<(), inrux_adapter::AdapterError> {
    let root = Node::row(vec![Node::text("One".into()), Node::text("Two".into())])
        .styled(Style::new().gap(12.0).padding(8.0).width(Size::Px(300.0)));
    let text = TextService::new(
        ProviderInfo {
            name: "specimen fixed metrics",
            kind: ProviderKind::Fallback,
        },
        |request: TextMeasure<'_>| {
            Ok(TextSize {
                width: request.text.chars().count() as f32 * 8.0,
                height: 18.0,
            })
        },
    );
    let mut service = LayoutService::new(TaffyLayout::new(text));
    let boxes = service.compute(LayoutRequest {
        root: &root,
        width: 300.0,
        height: None,
    })?;
    assert_eq!(boxes.children.len(), 2);
    assert_eq!(boxes.children[1].x, 44.0);
    println!(
        "{}: {} × {}, second item x = {}",
        service.provider().name,
        boxes.width,
        boxes.height,
        boxes.children[1].x
    );
    Ok(())
}
