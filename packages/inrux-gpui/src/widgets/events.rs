//! Native activations dispatch only through current framework event handles.
use inrux_adapter::AdapterError;
use inrux_components::{Node, NodeKind};
/// Native pointer and keyboard activation share one bounded-dispatch path.
pub(crate) fn activate(node: &Node) -> Result<(), AdapterError> {
    if node.disabled {
        return Ok(());
    }
    let result = match &node.kind {
        NodeKind::Button(_, Some(event)) => event.emit(()),
        NodeKind::Checkbox { checked, event, .. } => event.emit(!checked),
        _ => return Ok(()),
    };
    result.map_err(|error| AdapterError::Application(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    /// Disabled controls must not dispatch even if an activation arrives late.
    #[test]
    fn disabled_activation_never_dispatches() {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let mut node = Node::button(
            "Disabled".into(),
            Some(inrux_core::Emit::new(move |_| {
                observed.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
        );
        node.disabled = true;
        activate(&node).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
