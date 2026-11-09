//! Startup composition is the sole framework entry point that selects an engine.
use crate::{Adapter, AdapterError, Component, Rule, Scope, WindowOptions};
/// Mount using the single engine selected in Cargo features.
/// No engine is selected by default while comparative validation remains incomplete.
pub fn mount<S: Component>(title: &str, initial: S, scope: Scope) -> Result<(), AdapterError> {
    mount_with_styles(title, initial, scope, vec![])
}
/// Mount with global rules using the single engine enabled for this application.
pub fn mount_with_styles<S: Component>(
    title: &str,
    initial: S,
    scope: Scope,
    styles: Vec<Rule>,
) -> Result<(), AdapterError> {
    #[cfg(all(feature = "egui", not(any(feature = "masonry", feature = "gpui"))))]
    let adapter = crate::backends::Egui;
    #[cfg(all(feature = "masonry", not(any(feature = "egui", feature = "gpui"))))]
    let adapter = crate::backends::Masonry;
    #[cfg(all(feature = "gpui", not(any(feature = "egui", feature = "masonry"))))]
    let adapter = crate::backends::Gpui;
    #[cfg(any(
        all(feature = "egui", not(any(feature = "masonry", feature = "gpui"))),
        all(feature = "masonry", not(any(feature = "egui", feature = "gpui"))),
        all(feature = "gpui", not(any(feature = "egui", feature = "masonry")))
    ))]
    {
        mount_with(adapter, WindowOptions::new(title), initial, scope, styles)
    }
    #[cfg(any(
        all(feature = "egui", feature = "masonry"),
        all(feature = "egui", feature = "gpui"),
        all(feature = "masonry", feature = "gpui"),
        not(any(feature = "egui", feature = "masonry", feature = "gpui"))
    ))]
    {
        let _ = (title, initial, scope, styles);
        Err(AdapterError::Configuration("Enable exactly one engine feature: `egui`, `masonry` or `gpui`, or use mount_with for an explicitly linked adapter. Masonry and GPUI are experimental.".into()))
    }
}
/// Run an explicit adapter, allowing comparison binaries to link several engines deliberately.
/// The same component and styles are passed to the common contract unchanged.
pub fn mount_with<A: Adapter, S: Component>(
    adapter: A,
    window: WindowOptions,
    initial: S,
    scope: Scope,
    styles: Vec<Rule>,
) -> Result<(), AdapterError> {
    window.validate()?;
    adapter.run(window, initial, scope, styles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Events, Node};
    #[derive(Clone)]
    struct Example;
    impl Component for Example {
        type Action = ();
        /// No application actions are needed for this startup contract check.
        fn update(&mut self, _: (), _: &crate::Context<Self>) {}
        /// A minimal backend-neutral component.
        fn view(&self, _: &Events<Self>, _: &Scope) -> Node {
            Node::text("Example".into())
        }
    }
    struct RecordingAdapter;
    impl Adapter for RecordingAdapter {
        /// A test adapter opens no native window.
        fn capabilities(&self) -> crate::Capabilities {
            crate::Capabilities {
                name: "recording",
                primitives: &[crate::Primitive::Text],
                interaction_styles: false,
            }
        }
        /// Check startup arguments without importing a graphics engine.
        fn run<S: Component>(
            self,
            options: WindowOptions,
            _: S,
            _: Scope,
            styles: Vec<Rule>,
        ) -> Result<(), AdapterError> {
            assert_eq!(options.title, "Contract");
            assert_eq!(styles.len(), 1);
            Ok(())
        }
    }
    /// Startup accepts independently implemented adapters and forwards global rules.
    #[test]
    fn explicit_adapter_uses_only_the_common_contract() {
        mount_with(
            RecordingAdapter,
            WindowOptions::new("Contract"),
            Example,
            Scope::default(),
            crate::parse_css(".app { gap: 4px; }").unwrap(),
        )
        .unwrap();
    }
    /// Ambiguous and missing engine features produce actionable configuration errors.
    #[cfg(any(
        all(feature = "egui", feature = "masonry"),
        all(feature = "egui", feature = "gpui"),
        all(feature = "masonry", feature = "gpui"),
        not(any(feature = "egui", feature = "masonry", feature = "gpui"))
    ))]
    #[test]
    fn automatic_mount_requires_exactly_one_engine() {
        assert!(matches!(
            mount("Contract", Example, Scope::default()),
            Err(AdapterError::Configuration(_))
        ));
    }
}

#[cfg(all(test, any(feature = "egui", feature = "masonry", feature = "gpui")))]
mod service_tests {
    use super::*;
    use crate::services::{ProviderKind, Service, ServiceSupport};
    /// Every selected binding advertises its real provider and rejects unfinished component prerequisites.
    #[test]
    fn native_bindings_report_selected_services_and_missing_prerequisites() {
        let bindings: &[(&'static str, ServiceSupport)] = &[
            #[cfg(feature = "egui")]
            ("egui", crate::backends::Egui.services()),
            #[cfg(feature = "masonry")]
            ("masonry", crate::backends::Masonry.services()),
            #[cfg(feature = "gpui")]
            ("gpui", crate::backends::Gpui.services()),
        ];
        for &(name, support) in bindings {
            support
                .require(name, &[Service::Layout, Service::TextMeasurement])
                .unwrap();
            assert_eq!(
                support.provider(Service::Layout).unwrap().kind,
                ProviderKind::Fallback
            );
            assert_eq!(
                support.provider(Service::TextMeasurement).unwrap().kind,
                ProviderKind::Fallback
            );
            assert!(
                support
                    .require(name, &[Service::Scrolling, Service::Overlays])
                    .is_err()
            );
        }
    }
}
