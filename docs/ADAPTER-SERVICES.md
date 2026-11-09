# Adapter services and component prerequisites

The current layout/text service foundation is experimental; see [alpha status](STATUS.md). The layout/text foundation is extended by [clipping and scrolling](SCROLLING.md); the entire native service catalogue is not complete. Component code still uses Inrux APIs; providers and native resources stay inside private bindings.

## Operational boundaries

| Service | egui | Masonry | GPUI |
| --- | --- | --- | --- |
| Layout | Selected Taffy fallback | Selected Taffy fallback | Selected Taffy fallback |
| Text measurement | Shared inrux-text provider | Shared inrux-text provider | Shared inrux-text provider |
| Rectangular paint/input clipping | Native clip | Native clip | Native clip |
| UI-owned scrolling / commands / queries | Shared Inrux fallback | Shared Inrux fallback | Shared Inrux fallback |
| Independent focus-command / overlay contracts | Not yet operational | Not yet operational | Not yet operational |
| General asset / accessibility service contracts | Not yet operational | Not yet operational | Not yet operational |

Existing primitive focus, font assets and egui Dialog remain usable within their documented coverage. They do not constitute the separate advanced services above. Readiness means an operational framework contract, not merely an underlying engine capability. All three remain subject to [native conformance/platform limits](STATUS.md).

## Interface and ownership

- `Adapter::services()` reports operational `ServiceDescriptor` entries and `ProviderInfo` provenance. `require_services()` rejects unavailable prerequisites with `AdapterError::Unsupported`. Metadata names are diagnostic labels, not runtime provider dispatch keys. Hosting-only/custom adapters default to empty readiness until explicitly updated.
- `services::layout::LayoutProvider` owns its implementation state and any measurement context. `LayoutService<P>::compute(LayoutRequest)` validates the request/style/identity, executes the selected provider and validates finite/nonnegative geometry and matching child structure. Child order and CSS semantics remain provider conformance obligations; shape validation alone does not prove them.
- `services::text::TextMeasurer` is fallible; `TextService<F>` adapts provider callbacks and validates requests/results. Text bounds exclude framework control padding; width may exceed the wrapping constraint for unbreakable content. Positive infinite width means unwrapped text; negative/nonfinite results are invalid.
- `TaffyLayout<M>` owns or borrows its measurer and bridges errors into Taffy's infallible callback. The first measurement failure prevents publication of partial geometry. Native errors retain their original AdapterError variant/context.
- `LayoutRequest` borrows the resolved Node tree and uses logical pixels. Output is currently owned `LayoutBox` geometry with parent-relative positions and children in description order. Negative finite offsets are valid; content extents are reported separately and feed the clipping/scrolling contract.

Layout/text services that share a mutable native window can remain together inside one provider. The interface does not force competing mutable borrows, expose a native window, require Send/Sync, or box every call. Pure headless providers may run without a UI thread; native context borrows must obey their engine's thread affinity.

Static selection removes a mandatory runtime provider lookup, not all execution costs. Existing bindings still rebuild layout each frame; validation and geometry snapshots have real costs. Retained native results/queries, patches and caching require a measured later increment. No native layout substitution or speed ranking is claimed.

## Dependency boundary

`inrux-adapter` exposes the optional `taffy-layout` feature. A direct standalone dependency keeps that feature by default for the existing `layout::compute` API. The workspace facade disables adapter defaults; each current native binding explicitly enables the fallback it uses. A headless facade or custom provider with defaults disabled compiles no Taffy dependency.

The public `layout::compute` tuple-callback helper remains available with `taffy-layout`; it now delegates through the same service/validation path. New bindings use the fallible service interface. Taffy types, IDs and errors stay private. A future conforming native provider can use `LayoutService::new(NativeProvider { ... })` without enabling this fallback.

## Application readiness check

Ordinary applications need no provider-specific imports or engine branches. A generic bootstrap can check a component family's requirements before native allocation:

```rust
use inrux::{Adapter, AdapterError, services::Service};

fn require_basic_scene(adapter: &impl Adapter) -> Result<(), AdapterError> {
    adapter.require_services(&[Service::Layout, Service::TextMeasurement])
}
```

Each standalone playground's bootstrap additionally requires Clipping and Scrolling for its viewport specimens. All three also support `Clipping` and `Scrolling`; asking for `Overlays` still returns an explicit diagnostic. Components express their native primitive needs through the existing description/capability system; this does not add a component-to-adapter dependency cycle.

## Working specimens and checks

```sh
cargo run -p inrux-adapter --example layout_services --locked
cargo test -p inrux-adapter --locked
cargo test -p inrux-adapter --no-default-features --locked
cargo test -p inrux --no-default-features --locked
python3 tooling/backend-check/check.py
```

The headless example uses explicitly illustrative fixed text metrics, not a native font or performance measurement. Contract tests compile custom providers without Taffy or engine dependencies and cover retained non-Send state, invalid request/geometry, errors and fallback parity. The native standalone playgrounds exercise the same interface with real shaping, editing and event handling. See [the validation checkpoint](STATUS.md).

The private integration points are [egui rendering](../packages/inrux-egui/src/render.rs), [Masonry layout](../packages/inrux-masonry/src/layout.rs), [GPUI layout](../packages/inrux-gpui/src/layout.rs) and each binding's `services.rs` metadata. Framework service implementation stays modular under `inrux-adapter/src/services/`.

## Next component boundaries

Basic Row/Column/Text and the implemented [Box/Container/Center/Spacer group](LAYOUT-COMPONENTS.md) use this foundation. The [Flex/uniform Grid/AspectRatio group](LAYOUT-COMPONENTS.md#flex-grid-and-aspectratio) is now operational in the selected Taffy provider. Stack and richer grid placement require additional semantics and conformance. Spacer now uses validated growth in the selected fallback. [ScrollArea](SCROLLING.md) now provides clipping, scroll state/queries and input routing. Virtualized List still requires virtualization and semantic work; Select/Popover/Dialog require shared overlay placement/lifetime, dismissal and focus ownership. Generation-safe focus/ref commands need a target/command boundary. Implement those with specimens before advertising those component families as portable. The entire adapter foundation is not marked complete by this first service increment.

[Card, Badge and decorative Separator](CONTENT-COMPONENTS.md) also use the existing layout/text/primitive surface contracts. They add no native service, binding or semantic accessibility role. The public implementations live once in inrux-components; no adapter-specific copies are required.

The [shared typography correction](TEXT-RENDERING.md) now supplies the text provider for all three bindings, including glyph coverage and editor geometry. Font assets are registered once with the common registry before mounting. Native image uploads remain private; native measurement-only delegation is no longer the default text path.
