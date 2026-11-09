//! Validate the initial Masonry surface instead of silently approximating missing controls.
use inrux_adapter::{AdapterError, Capabilities, Primitive};
use inrux_components::Node;
/// Controls implemented by the first retained Masonry adapter.
pub(crate) const CAPABILITIES: Capabilities = Capabilities {
    name: "masonry",
    primitives: &[
        Primitive::Column,
        Primitive::Row,
        Primitive::Text,
        Primitive::Button,
        Primitive::Input,
        Primitive::TextArea,
        Primitive::Checkbox,
        Primitive::Switch,
        Primitive::Progress,
        Primitive::Meter,
        Primitive::Empty,
    ],
    interaction_styles: true,
};
/// Validate before translating a frame; the host displays failures as diagnostics.
pub(crate) fn validate(node: &Node) -> Result<(), AdapterError> {
    CAPABILITIES.validate(node)?;
    inrux_adapter::layout::validate(node)
}
