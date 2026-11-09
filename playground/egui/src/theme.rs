//! Build-time CSS compilation and application-owned theme state, independent of native hosts.
use inrux::{CompiledStyles, Context, Stylesheet};
use std::sync::{Arc, OnceLock};
/// Application theme handles are shared; theme choices belong to this application instance.
#[derive(Clone)]
pub struct Theme {
    pub dark: bool,
    pub sheet: Stylesheet,
}
impl Theme {
    /// Reuse generated typed rules and their index; no CSS is parsed during startup or toggling.
    pub fn new() -> Self {
        Self {
            dark: false,
            sheet: Stylesheet::from_compiled(light()),
        }
    }
    /// Replace only the immutable style index, preserving local component and native editor state.
    pub fn toggle(&mut self) {
        self.dark = !self.dark;
        self.sheet
            .replace_compiled(if self.dark { dark() } else { light() });
    }
}
/// Lazily index the generated light rule set once per process.
fn light() -> Arc<CompiledStyles> {
    static SHEET: OnceLock<Arc<CompiledStyles>> = OnceLock::new();
    SHEET
        .get_or_init(|| {
            Arc::new(CompiledStyles::new(include!(concat!(
                env!("OUT_DIR"),
                "/light_styles.rs"
            ))))
        })
        .clone()
}
/// Lazily index the generated dark rule set once per process.
fn dark() -> Arc<CompiledStyles> {
    static SHEET: OnceLock<Arc<CompiledStyles>> = OnceLock::new();
    SHEET
        .get_or_init(|| {
            Arc::new(CompiledStyles::new(include!(concat!(
                env!("OUT_DIR"),
                "/dark_styles.rs"
            ))))
        })
        .clone()
}
/// Handle theme requests through the normal coordinator, never inside an engine callback.
pub fn toggle(state: &mut crate::app::State, _: &Context<crate::app::State>) {
    state.theme.toggle();
}
