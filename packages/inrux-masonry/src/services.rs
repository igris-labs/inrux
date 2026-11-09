//! Services actually wired by this binding; advanced component prerequisites remain unavailable.
use inrux_adapter::services::{
    ProviderInfo, ProviderKind, Service, ServiceDescriptor, ServiceSupport, layout::TAFFY_PROVIDER,
};
/// Shared shaping and glyph coverage, with private native texture uploads.
pub(crate) const TEXT: ProviderInfo = ProviderInfo {
    name: "inrux-shared-text",
    kind: ProviderKind::Fallback,
};
/// Only advertise independently operational contracts, not incidental primitive internals.
pub(crate) const SUPPORT: ServiceSupport = ServiceSupport(&[
    ServiceDescriptor {
        service: Service::Clipping,
        provider: ProviderInfo {
            name: "native-rectangular-clip",
            kind: ProviderKind::Native,
        },
    },
    ServiceDescriptor {
        service: Service::Scrolling,
        provider: ProviderInfo {
            name: "inrux-viewport",
            kind: ProviderKind::Fallback,
        },
    },
    ServiceDescriptor {
        service: Service::Layout,
        provider: TAFFY_PROVIDER,
    },
    ServiceDescriptor {
        service: Service::TextMeasurement,
        provider: TEXT,
    },
]);
