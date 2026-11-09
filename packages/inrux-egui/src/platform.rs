//! Platform color configuration stays private to the native binding.
/// Tag OpenGL window surfaces as sRGB so authored colors match color-managed engines.
pub(crate) fn configure_colors() {
    #[cfg(target_os = "macos")]
    if let Some(main) = objc2::MainThreadMarker::new() {
        use objc2_app_kit::{NSApplication, NSColorSpace};
        let application = NSApplication::sharedApplication(main);
        for window in application.windows() {
            window.setColorSpace(Some(&NSColorSpace::sRGBColorSpace()));
        }
    }
}
