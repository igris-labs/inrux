//! Service contracts work without Taffy or any native engine dependency.
use inrux_adapter::{
    AdapterError,
    layout::{LayoutBox, TextMeasure},
    services::{
        ProviderInfo, ProviderKind, Service, ServiceDescriptor, ServiceSupport,
        layout::{LayoutProvider, LayoutRequest, LayoutService},
        text::{TextMeasurer, TextService, TextSize},
    },
};
use inrux_components::{Node, NodeKind};
use inrux_style::Style;
use std::{cell::Cell, rc::Rc};
const TEST_PROVIDER: ProviderInfo = ProviderInfo {
    name: "contract specimen",
    kind: ProviderKind::Fallback,
};
struct Specimen {
    calls: Rc<Cell<usize>>,
    output: LayoutBox,
}
impl LayoutProvider for Specimen {
    /// This test provider supplies controlled geometry, not a production layout implementation.
    fn provider(&self) -> ProviderInfo {
        TEST_PROVIDER
    }
    /// Preserve state across requests without requiring a Send native context.
    fn layout(&mut self, _: LayoutRequest<'_>) -> Result<LayoutBox, AdapterError> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.output.clone())
    }
}
/// Build a request without registering a renderer or choosing an algorithm.
fn request(root: &Node) -> LayoutRequest<'_> {
    LayoutRequest {
        root,
        width: 300.0,
        height: None,
    }
}
/// Bind test-controlled output and an observable invocation count.
fn specimen(output: LayoutBox) -> (LayoutService<Specimen>, Rc<Cell<usize>>) {
    let calls = Rc::new(Cell::new(0));
    (
        LayoutService::new(Specimen {
            calls: calls.clone(),
            output,
        }),
        calls,
    )
}
/// A caller-selected provider can retain non-Send state and run without a layout-library dependency.
#[test]
fn custom_provider_is_used_and_retains_its_state() {
    let root = Node::text("Specimen".into());
    let expected = LayoutBox {
        width: 80.0,
        height: 18.0,
        ..Default::default()
    };
    let (mut service, calls) = specimen(expected.clone());
    assert_eq!(service.compute(request(&root)).unwrap(), expected);
    service.compute(request(&root)).unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(service.provider(), TEST_PROVIDER);
}
/// Invalid constraints never invoke the native/fallback provider.
#[test]
fn invalid_viewports_are_rejected_before_execution() {
    let root = Node::text("Specimen".into());
    let (mut service, calls) = specimen(LayoutBox::default());
    for width in [f32::NAN, f32::INFINITY, -1.0] {
        assert!(
            service
                .compute(LayoutRequest {
                    width,
                    ..request(&root)
                })
                .is_err()
        );
    }
    assert!(
        service
            .compute(LayoutRequest {
                height: Some(f32::INFINITY),
                ..request(&root)
            })
            .is_err()
    );
    assert_eq!(calls.get(), 0);
}
/// Duplicate identities and malformed styles fail at the common boundary before native calls.
#[test]
fn invalid_tree_is_rejected_before_execution() {
    let duplicate = Node::column(vec![
        Node::text("a".into()).key("same"),
        Node::text("b".into()).key("same"),
    ]);
    let invalid_style = Node::text("a".into()).styled(Style::new().padding(f32::NAN));
    let (mut service, calls) = specimen(LayoutBox::default());
    for node in [duplicate, invalid_style] {
        assert!(service.compute(request(&node)).is_err());
    }
    assert_eq!(calls.get(), 0);
}
/// Invalid provider bounds and missing child geometry cannot reach native placement/painting.
#[test]
fn malformed_provider_geometry_is_rejected() {
    let root = Node::text("Specimen".into());
    for geometry in [
        LayoutBox {
            width: -1.0,
            ..Default::default()
        },
        LayoutBox {
            x: f32::NAN,
            ..Default::default()
        },
        LayoutBox {
            height: f32::INFINITY,
            ..Default::default()
        },
        LayoutBox {
            children: vec![LayoutBox::default()],
            ..Default::default()
        },
    ] {
        let (mut service, _) = specimen(geometry);
        let error = service.compute(request(&root)).unwrap_err();
        assert!(error.to_string().contains("contract specimen"));
    }
}
/// Overflow is a separate clip/scroll contract; layout does not silently clamp valid offsets.
#[test]
fn finite_overflow_geometry_remains_available() {
    let root = Node::text("Overflow".into());
    let expected = LayoutBox {
        x: -10.0,
        width: 600.0,
        height: 18.0,
        ..Default::default()
    };
    let (mut service, _) = specimen(expected.clone());
    assert_eq!(service.compute(request(&root)).unwrap(), expected);
}
/// Readiness reports only implemented contracts and provides a useful missing-prerequisite error.
#[test]
fn service_readiness_does_not_imply_advanced_component_support() {
    const SUPPORT: ServiceSupport = ServiceSupport(&[ServiceDescriptor {
        service: Service::Layout,
        provider: TEST_PROVIDER,
    }]);
    SUPPORT.require("specimen", &[Service::Layout]).unwrap();
    assert_eq!(SUPPORT.provider(Service::Layout), Some(TEST_PROVIDER));
    let error = SUPPORT
        .require("specimen", &[Service::Layout, Service::Scrolling])
        .unwrap_err();
    assert!(matches!(
        error,
        AdapterError::Unsupported {
            backend: "specimen",
            ..
        }
    ));
    assert!(error.to_string().contains("Scrolling service contract"));
    assert!(
        ServiceSupport::default()
            .provider(Service::Layout)
            .is_none()
    );
}
/// Native text failures retain their original error type/context rather than becoming zero bounds.
#[test]
fn text_service_preserves_native_errors() {
    let expected = AdapterError::Native {
        backend: "specimen",
        message: "shaping failed".into(),
    };
    let mut text = TextService::new(TEST_PROVIDER, |_: TextMeasure<'_>| Err(expected.clone()));
    let actual = text.measure(TextMeasure {
        text: "Hello",
        font_size: 15.0,
        font_family: None,
        max_width: f32::INFINITY,
    });
    assert_eq!(actual.unwrap_err(), expected);
}
/// Text contract validation handles native output as well as caller inputs.
#[test]
fn text_service_rejects_invalid_requests_and_results() {
    let calls = Cell::new(0);
    let mut text = TextService::new(TEST_PROVIDER, |_: TextMeasure<'_>| {
        calls.set(calls.get() + 1);
        Ok(TextSize {
            width: f32::NAN,
            height: 18.0,
        })
    });
    assert!(
        text.measure(TextMeasure {
            text: "Hello",
            font_size: -1.0,
            font_family: None,
            max_width: 300.0
        })
        .is_err()
    );
    assert_eq!(calls.get(), 0);
    assert!(
        text.measure(TextMeasure {
            text: "Hello",
            font_size: 15.0,
            font_family: None,
            max_width: 300.0
        })
        .is_err()
    );
    assert_eq!(calls.get(), 1);
}
/// Providers may keep related mutable native layout/text context together behind the same API.
#[test]
fn provider_can_own_its_measurement_context() {
    struct Measured<M> {
        text: M,
    }
    impl<M: TextMeasurer> LayoutProvider for Measured<M> {
        /// Identify this limited text specimen without claiming a native engine implementation.
        fn provider(&self) -> ProviderInfo {
            TEST_PROVIDER
        }
        /// Use the owned context internally, without competing mutable window borrows in the API.
        fn layout(&mut self, request: LayoutRequest<'_>) -> Result<LayoutBox, AdapterError> {
            let NodeKind::Text(value) = &request.root.kind else {
                return Err(AdapterError::Unsupported {
                    backend: "specimen",
                    feature: "non-text fixture".into(),
                });
            };
            let size = self.text.measure(TextMeasure {
                text: value,
                font_size: 15.0,
                font_family: None,
                max_width: request.width,
            })?;
            Ok(LayoutBox {
                width: size.width,
                height: size.height,
                ..Default::default()
            })
        }
    }
    let context = Rc::new(Cell::new(0));
    let native_context = context.clone();
    let text = TextService::new(TEST_PROVIDER, move |_: TextMeasure<'_>| {
        native_context.set(native_context.get() + 1);
        Ok(TextSize {
            width: 42.0,
            height: 18.0,
        })
    });
    let root = Node::text("Hello".into());
    assert_eq!(
        LayoutService::new(Measured { text })
            .compute(request(&root))
            .unwrap()
            .width,
        42.0
    );
    assert_eq!(context.get(), 1);
}

#[cfg(feature = "taffy-layout")]
mod fallback {
    use super::*;
    use inrux_adapter::services::layout::TaffyLayout;
    /// An infallible third-party callback must not cause partial geometry to mask a native error.
    #[test]
    fn fallback_propagates_first_measurement_failure() {
        let calls = Cell::new(0);
        let failure = AdapterError::Native {
            backend: "specimen",
            message: "missing font".into(),
        };
        let text = TextService::new(TEST_PROVIDER, |_: TextMeasure<'_>| {
            calls.set(calls.get() + 1);
            Err(failure.clone())
        });
        let root = Node::column(vec![Node::text("a".into()), Node::text("b".into())]);
        assert_eq!(
            LayoutService::new(TaffyLayout::new(text))
                .compute(request(&root))
                .unwrap_err(),
            failure
        );
        assert_eq!(calls.get(), 1);
    }
    /// The selected service and compatibility wrapper have identical geometry/rounding behavior.
    #[test]
    fn fallback_matches_compatibility_layout() {
        use inrux_style::Size;
        let root = Node::row(vec![Node::text("a".into()), Node::text("b".into())])
            .styled(Style::new().gap(10.0).padding(8.0).width(Size::Px(300.0)));
        let expected =
            inrux_adapter::layout::compute(&root, 300.0, None, |_| (10.2, 18.2)).unwrap();
        let text = TextService::new(TEST_PROVIDER, |_: TextMeasure<'_>| {
            Ok(TextSize {
                width: 10.2,
                height: 18.2,
            })
        });
        let actual = LayoutService::new(TaffyLayout::new(text))
            .compute(request(&root))
            .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            (actual.children[0].width, actual.children[0].height),
            (11.0, 19.0)
        );
        assert_eq!(actual.children[1].x, 29.0);
    }
    /// Raw custom measurers cannot hand invalid dimensions to Taffy even without a TextService wrapper.
    #[test]
    fn fallback_validates_raw_measurer_output() {
        struct Invalid;
        impl TextMeasurer for Invalid {
            /// Deliberately malformed native result for the fallback boundary test.
            fn measure(&mut self, _: TextMeasure<'_>) -> Result<TextSize, AdapterError> {
                Ok(TextSize {
                    width: -1.0,
                    height: 18.0,
                })
            }
        }
        let root = Node::text("a".into());
        assert!(
            LayoutService::new(TaffyLayout::new(Invalid))
                .compute(request(&root))
                .is_err()
        );
    }
}
