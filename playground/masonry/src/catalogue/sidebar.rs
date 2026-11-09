//! Accessible native buttons carry typed navigation; no pointer-only text links.
use crate::navigation::Page;
use inrux::{Emit, Node, Style};
/// Compose grouped navigation once, using only framework descriptions.
pub fn sidebar(selected: Page, navigate: Emit<Page>) -> Node {
    let mut children = vec![heading("Components")];
    for example in super::EXAMPLES {
        children.push(item(
            example.name(),
            Page::Example(*example),
            selected,
            navigate.clone(),
        ));
    }
    children.push(heading("Examples"));
    for (label, page) in [
        ("Layout", Page::Layout),
        ("Controls", Page::Controls),
        ("Composition", Page::Composition),
        ("Background", Page::Background),
        ("More layout", Page::LayoutMore),
        ("Content", Page::Content),
        ("Engine", Page::Engine),
        ("Scrolling", Page::Scrolling),
    ] {
        children.push(item(label, page, selected, navigate.clone()));
    }
    Node::column(children).styled(Style::new().gap(4.0))
}
/// Group labels describe the options below without pretending to be interactive.
fn heading(label: &str) -> Node {
    Node::text(label.into()).classes("sidebar-group")
}
/// Preserve keyboard focus, disabled admission and ordinary event dispatch through Button.
fn item(label: &str, page: Page, selected: Page, navigate: Emit<Page>) -> Node {
    Node::button(label.into(), Some(Emit::new(move |_| navigate.emit(page))))
        .key(label)
        .classes(if page == selected {
            "nav-item selected"
        } else {
            "nav-item"
        })
}
