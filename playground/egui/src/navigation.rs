//! Simple in-app view selection; no framework router is required.
use crate::app::State;
use inrux::Context;
/// Explicit local screens, each mounted only while selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Example(crate::catalogue::Example),
    Layout,
    LayoutMore,
    Content,
    Controls,
    Composition,
    Background,
    Engine,
    Scrolling,
}
/// Queue a typed navigation choice through the normal application event channel.
pub fn navigate(state: &mut State, page: Page, _: &Context<State>) {
    state.page = page;
}
impl Page {
    /// Identify individual component pages without string-based component lookup.
    pub fn example(self) -> Option<crate::catalogue::Example> {
        if let Self::Example(example) = self {
            Some(example)
        } else {
            None
        }
    }
    /// Keep header and breadcrumb labels synchronized with navigation state.
    pub fn title(self) -> &'static str {
        match self {
            Self::Example(example) => example.name(),
            Self::Layout => "Layout",
            Self::LayoutMore => "More layout",
            Self::Controls => "Controls",
            Self::Content => "Content",
            Self::Composition => "Composition",
            Self::Background => "Background",
            Self::Engine => "Engine",
            Self::Scrolling => "Scrolling",
        }
    }
    /// Describe the selected task rather than showing unrelated dashboard metrics.
    pub fn description(self) -> &'static str {
        match self {
            Self::Example(example) => example.description(),
            Self::Layout | Self::LayoutMore => {
                "Explore physical layout, sizing and retained child controls."
            }
            Self::Controls => "Edit models and try enabled, disabled and saved states.",
            Self::Content => "Compose surfaces, labels and separators with working controls.",
            Self::Composition => {
                "Typed props, emitted events, computed values and retained children."
            }
            Self::Background => {
                "Start and cancel worker tasks while native controls remain interactive."
            }
            Self::Engine => {
                "Inspect the selected engine's implemented capabilities and limitations."
            }
            Self::Scrolling => {
                "Test nested viewports, clipping, scroll commands and native text editing."
            }
        }
    }
}

/// Issue a fresh scroll command without owning subsequent user scrolling.
pub fn top(state: &mut State, _: &Context<State>) {
    state.request.revision += 1;
    state.request.offset.y = 0.0;
}
/// The native viewport clamps this finite destination to the actual page extent.
pub fn end(state: &mut State, _: &Context<State>) {
    state.request.revision += 1;
    state.request.offset.y = 1_000_000.0;
}
