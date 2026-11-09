//! Thin adapters to standard Cargo commands and an existing development watcher.
use std::{path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
/// Interpret both direct execution and Cargo's extra `inrux` argument.
pub(crate) fn run(mut args: Vec<String>) -> Result<u8, Error> {
    if args.first().is_some_and(|arg| arg == "inrux") {
        args.remove(0);
    }
    match args.first().map(String::as_str) {
        Some("new") => {
            create(&args[1..])?;
            Ok(0)
        }
        Some("check") | Some("build") | Some("run") => {
            let status = Command::new("cargo")
                .arg(&args[0])
                .args(&args[1..])
                .status()?;
            Ok(status
                .code()
                .and_then(|code| u8::try_from(code).ok())
                .unwrap_or(1))
        }
        Some("dev") => crate::dev::run(&args[1..]),
        Some("validate") if args.len() == 2 => {
            let text = std::fs::read_to_string(&args[1])?;
            inrux_compiler::compile_source(inrux_compiler::SourceFile::new(&args[1], text))?;
            Ok(0)
        }
        _ => {
            eprintln!(
                "Usage: cargo inrux new <directory> --framework-path <checkout> --backend egui|masonry|gpui\n       cargo inrux check|build|run [cargo arguments]\n       cargo inrux dev [cargo run arguments]\n       cargo inrux validate <component.iru>\n\nLocal framework path is required until Inrux is published."
            );
            Ok(2)
        }
    }
}

/// Parse named scaffold options before creating a directory; unavailable engines never produce files.
fn create(args: &[String]) -> Result<(), Error> {
    let Some(destination) = args.first().filter(|arg| !arg.starts_with('-')) else {
        return Err("Expected an application directory".into());
    };
    let mut framework = None;
    let mut backend = None;
    let (options, remainder) = args[1..].as_chunks::<2>();
    for option in options {
        match option[0].as_str() {
            "--framework-path" if framework.is_none() => framework = Some(option[1].as_str()),
            "--backend" if backend.is_none() => backend = Some(option[1].as_str()),
            _ => return Err(format!("Unknown or repeated option `{}`", option[0]).into()),
        }
    }
    if !remainder.is_empty() {
        return Err("A scaffold option is missing its value".into());
    }
    let framework = framework.ok_or("--framework-path is required until publication")?;
    let backend = backend.ok_or(
        "Choose --backend egui, --backend masonry or --backend gpui; no performance default has been selected",
    )?;
    crate::scaffold::create(Path::new(destination), Path::new(framework), backend)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Invalid choices must be rejected before any application directory is written.
    #[test]
    fn scaffold_requires_an_available_explicit_engine() {
        let destination =
            std::env::temp_dir().join(format!("inrux-invalid-{}", std::process::id()));
        let framework = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let args = vec![
            destination.display().to_string(),
            "--framework-path".into(),
            framework.display().to_string(),
        ];
        assert!(create(&args).unwrap_err().to_string().contains("--backend"));
        let mut unavailable = args.clone();
        unavailable.extend(["--backend".into(), "unknown-engine".into()]);
        assert!(
            create(&unavailable)
                .unwrap_err()
                .to_string()
                .contains("unavailable")
        );
        assert!(!destination.exists());
    }
}
