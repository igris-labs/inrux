//! Geometry, validation and palette behavior are shared rather than copied into components.
use inrux_adapter::{
    controls::{contrasting_ink, toggle_colors, toggle_size},
    layout,
};
use inrux_components::{Node, NodeKind, ToggleKind};
#[cfg(feature = "taffy-layout")]
use inrux_style::Size;
use inrux_style::{Color, Style};
/// Invalid ranges cannot reach native painting or accessibility APIs with NaN/Infinity.
#[test]
fn range_validation_rejects_invalid_names_bounds_and_values() {
    for node in [
        Node::switch(" ".into(), false, inrux_core::Emit::new(|_| Ok(()))),
        Node::progress(" ".into(), 0.0, 100.0),
        Node::progress("Work".into(), -1.0, 100.0),
        Node::progress("Work".into(), 101.0, 100.0),
        Node::progress("Work".into(), 0.0, 0.0),
        Node::progress("Work".into(), f64::NAN, 100.0),
        Node::meter("Value".into(), 0.0, 1.0, 1.0),
        Node::meter("Value".into(), 0.0, -f64::MAX, f64::MAX),
    ] {
        assert!(layout::validate(&node).is_err());
    }
    assert!(layout::validate(&Node::progress("Work".into(), 100.0, 100.0)).is_ok());
}
/// Theme accents inherit into nested controls, with readable marks on bright and dark fills.
#[test]
fn inherited_accent_palette_and_switch_metrics_are_consistent() {
    let mut tree = Node::column(vec![Node::switch(
        "Sync".into(),
        true,
        inrux_core::Emit::new(|_| Ok(())),
    )])
    .styled(Style::new().accent_color(Color(240, 240, 240)));
    tree.inherit(&Style::new());
    let palette = toggle_colors(ToggleKind::Switch, true, &tree.children[0].style());
    assert_eq!(palette.fill, Color(240, 240, 240));
    assert_eq!(palette.mark, contrasting_ink(palette.fill));
    assert_eq!(palette.mark, Color(22, 22, 22));
    assert_eq!(toggle_size(ToggleKind::Switch), (34.0, 20.0));
    assert!(matches!(
        tree.children[0].kind,
        NodeKind::Checkbox {
            kind: ToggleKind::Switch,
            ..
        }
    ));
}
/// Rows and columns are distinct grid axes, independent of flex direction.
#[cfg(feature = "taffy-layout")]
#[test]
fn independent_grid_gaps_and_range_intrinsic_geometry_work() {
    let cell = || Node::box_container(vec![]).styled(Style::new().height(Size::Px(20.0)));
    let node = Node::grid(vec![cell(), cell(), cell(), cell()]).styled(
        Style::new()
            .width(Size::Px(210.0))
            .row_gap(20.0)
            .column_gap(10.0),
    );
    let boxes = layout::compute(&node, 210.0, None, |_| (0.0, 0.0)).unwrap();
    assert_eq!(boxes.children[0].width, 100.0);
    assert_eq!(boxes.children[1].x, 110.0);
    assert_eq!(boxes.children[2].y, 40.0);
    let progress = layout::compute(
        &Node::progress("Work".into(), 25.0, 100.0),
        210.0,
        None,
        |_| panic!("Range indicators must not request intrinsic text measurement"),
    )
    .unwrap();
    assert_eq!((progress.width, progress.height), (210.0, 8.0));
}
