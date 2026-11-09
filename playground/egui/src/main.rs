//! Independent native catalogue application; engine selection is confined to bootstrap.
mod composables;
mod theme;
use inrux::Adapter;
mod catalogue;
mod engine;
mod features;
mod navigation;
include!(concat!(env!("OUT_DIR"), "/inrux_app.rs"));

/// Mount App.iru with local global styles and the same bundled face in every application.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = engine::adapter();
    adapter.require_services(&[
        inrux::services::Service::Layout,
        inrux::services::Service::TextMeasurement,
        inrux::services::Service::Clipping,
        inrux::services::Service::Scrolling,
    ])?;
    let mut window = inrux::WindowOptions::new(engine::NAME);
    window.width = 1200.0;
    window.height = 860.0;
    window.min_width = 900.0;
    window.min_height = 820.0;
    if std::env::args().any(|arg| arg == "--compact") {
        window.width = 900.0;
        window.height = 820.0;
    }
    window.fonts.push(inrux::FontAsset::new(
        "Noto Sans",
        include_bytes!("assets/fonts/NotoSans-Regular.ttf").as_slice(),
    ));
    window.fonts.push(inrux::FontAsset::new(
        "Noto Sans Mono",
        include_bytes!("assets/fonts/NotoSansMono.ttf").as_slice(),
    ));
    let state = app::setup();
    let scope = app::scope().provide(state.theme.sheet.clone());
    inrux::mount_with(adapter, window, state, scope, inrux_global_styles())?;
    Ok(())
}
#[cfg(test)]
mod tests;
#[cfg(test)]
mod typecheck;

#[cfg(test)]
mod native_tests;
