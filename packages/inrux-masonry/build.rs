//! Align shader diagnostic features on the host dependency graph.
//! Naga 25 (GPUI) enables codespan termcolor; Naga 26 (Vello) must opt into it too.
//! The build dependency makes that upstream feature combination compile in comparison builds.
/// Cargo needs a build target to activate the host-side dependency feature.
fn main() {}
