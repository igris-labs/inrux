//! Portable local starter generation; never overwrites an existing application directory.
use std::{fs, path::Path};

/// Build a reviewable starter using explicit dependency paths until registry publication.
pub(crate) fn create(
    destination: &Path,
    framework: &Path,
    backend: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !matches!(backend, "egui" | "masonry" | "gpui") {
        return Err(format!("Backend `{backend}` is unavailable. Choose egui, experimental masonry or experimental gpui.").into());
    }
    let framework = framework.canonicalize()?;
    for package in ["inrux", "inrux-compiler"] {
        if !framework
            .join("packages")
            .join(package)
            .join("Cargo.toml")
            .is_file()
        {
            return Err(format!("{} is not an Inrux workspace", framework.display()).into());
        }
    }
    // Build all text before creating files, and fail if any destination already exists.
    let manifest = format!(
        "[package]\nname = \"my-inrux-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\ninrux = {{ path = {}, default-features = false, features = [\"{backend}\"] }}\n\n[build-dependencies]\ninrux-compiler = {{ path = {} }}\n",
        toml_path(&framework.join("packages/inrux"))?,
        toml_path(&framework.join("packages/inrux-compiler"))?
    );
    fs::create_dir(destination)?;
    for directory in ["src", "src/components", "src/composables", "src/styles"] {
        fs::create_dir(destination.join(directory))?;
    }
    let files = [
        ("Cargo.toml", manifest.as_str()),
        ("build.rs", include_str!("templates/build.rs.txt")),
        ("src/main.rs", include_str!("templates/main.rs.txt")),
        ("src/App.iru", include_str!("templates/App.iru.txt")),
        (
            "src/components/Counter.iru",
            include_str!("templates/Counter.iru.txt"),
        ),
        ("src/composables/mod.rs", "pub mod use_counter;\n"),
        (
            "src/composables/use_counter.rs",
            include_str!("templates/use_counter.rs.txt"),
        ),
        (
            "src/styles/global.css",
            ".app { padding: 24px; gap: 16px; background: #f5f7f6; }\n",
        ),
        ("README.md", include_str!("templates/README.md.txt")),
        (".gitignore", "/target\n"),
    ];
    for (name, text) in files {
        fs::write(destination.join(name), text)?;
    }
    println!(
        "Created {}. Run `cargo run` from that directory.",
        destination.display()
    );
    Ok(())
}
/// JSON-compatible quoted escapes also encode valid TOML basic strings, including apostrophes.
fn toml_path(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let text = path.to_str().ok_or("Dependency path is not valid UTF-8")?;
    Ok(serde_json::to_string(text)?)
}
