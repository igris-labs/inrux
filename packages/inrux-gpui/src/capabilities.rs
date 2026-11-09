//! Explicit gates keep incomplete native services from silently changing Inrux behavior.
use inrux_adapter::{AdapterError, Capabilities, Primitive};
use inrux_components::Node;
/// Initial portable surface; native editing and overlays require separate conformance work.
pub(crate) const CAPABILITIES: Capabilities = Capabilities {
    name: "gpui",
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
/// Validate the complete description before creating any native element.
pub(crate) fn validate(node: &Node) -> Result<(), AdapterError> {
    CAPABILITIES.validate(node)?;
    inrux_adapter::layout::validate(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Unsupported popup selection remains an explicit capability error.
    #[test]
    fn unsupported_selection_is_an_actionable_capability_error() {
        let input = Node::select(
            "value".into(),
            vec!["value".into()],
            inrux_core::Emit::new(|_| Ok(())),
        );
        assert!(matches!(
            validate(&Node::column(vec![input])),
            Err(AdapterError::Unsupported {
                backend: "gpui",
                ..
            })
        ));
    }
}
