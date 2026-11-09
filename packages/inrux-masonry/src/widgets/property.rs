//! Do not invalidate native layout/paint for unchanged local property values.
use masonry::core::{Property, Widget, WidgetMut};
/// Preserve authored-vs-default ownership while avoiding Masonry's unconditional property-change callbacks.
pub(crate) fn set<W: Widget + ?Sized, P: Property + PartialEq>(
    widget: &mut WidgetMut<'_, W>,
    value: Option<P>,
) {
    match value {
        Some(value) if !widget.contains_prop::<P>() || widget.get_prop::<P>() != &value => {
            widget.insert_prop(value);
        }
        None if widget.contains_prop::<P>() => {
            widget.remove_prop::<P>();
        }
        _ => {}
    }
}
