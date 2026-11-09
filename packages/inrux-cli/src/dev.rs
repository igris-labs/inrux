//! Native file notifications reuse notify; Cargo decides which changed inputs need recompilation.
use crate::preview::{self, Preview};
use notify::{RecursiveMode, Watcher};
use serde_json::Value;
use std::{
    path::PathBuf,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

/// Coalesce edits into rebuilds and preserve the last successful preview on compilation failure.
pub(crate) fn run(args: &[String]) -> Result<u8, Box<dyn std::error::Error>> {
    let separator = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());
    let build_args = &args[..separator];
    let application_args = args.get(separator + 1..).unwrap_or(&[]);
    let inputs = watch_inputs(build_args)?;
    let sources: Vec<_> = inputs
        .iter()
        .filter(|(_, mode)| *mode == RecursiveMode::Recursive)
        .map(|(path, _)| normalized_path(path))
        .collect();
    let (tx, rx) = mpsc::sync_channel(1);
    let events = tx.clone();
    let mut watcher =
        notify::recommended_watcher(move |event: notify::Result<notify::Event>| match event {
            Ok(event)
                if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() =>
            {
                if event.paths.iter().any(|path| relevant(path, &sources)) {
                    let _ = events.try_send(());
                }
            }
            Err(error) => eprintln!("Watcher failed: {error}"),
            _ => {}
        })?;
    for (path, recursive) in inputs {
        watcher.watch(&path, recursive)?;
    }
    let stopped = Arc::new(AtomicBool::new(false));
    let interrupt = stopped.clone();
    ctrlc::set_handler(move || {
        interrupt.store(true, Ordering::Release);
        let _ = tx.try_send(());
    })?;
    let mut preview = Preview::default();
    eprintln!(
        "Watching Rust, .iru, style, and Cargo inputs. Rust changes rebuild and restart the preview."
    );
    loop {
        if let Some(executable) = preview::build(build_args)?
            && !stopped.load(Ordering::Acquire)
        {
            preview.replace(executable, application_args)?;
        }
        if stopped.load(Ordering::Acquire) {
            break;
        }
        rx.recv()?;
        // Debouncing is bounded; events queued during a build trigger another pass afterward.
        while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
            if stopped.load(Ordering::Acquire) {
                break;
            }
        }
        if stopped.load(Ordering::Acquire) {
            break;
        }
    }
    Ok(0)
}
/// Watch every local package source, including path dependencies, without observing target outputs.
fn watch_inputs(
    args: &[String],
) -> Result<Vec<(PathBuf, RecursiveMode)>, Box<dyn std::error::Error>> {
    let mut command = Command::new("cargo");
    command.args(["metadata", "--format-version=1"]);
    if let Some(index) = args.iter().position(|arg| arg == "--manifest-path") {
        command
            .arg("--manifest-path")
            .arg(args.get(index + 1).ok_or("Missing --manifest-path value")?);
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)?;
    let mut inputs = vec![];
    for package in metadata["packages"]
        .as_array()
        .ok_or("Cargo metadata omitted packages")?
    {
        if package["source"].is_string() {
            continue;
        }
        let manifest = PathBuf::from(
            package["manifest_path"]
                .as_str()
                .ok_or("Cargo metadata omitted manifest path")?,
        );
        let directory = manifest.parent().ok_or("Manifest has no parent")?;
        inputs.push((directory.to_path_buf(), RecursiveMode::NonRecursive));
        for name in ["src", "assets"] {
            let path = directory.join(name);
            if path.is_dir() {
                inputs.push((path, RecursiveMode::Recursive));
            }
        }
    }
    Ok(inputs)
}

/// Ignore build output and runtime-written files while retaining source/assets and Cargo configuration edits.
fn relevant(path: &std::path::Path, sources: &[PathBuf]) -> bool {
    if path.components().any(|part| {
        matches!(
            part.as_os_str().to_str(),
            Some("target" | "node_modules" | ".git")
        )
    }) {
        return false;
    }
    let normalized = normalized_path(path);
    sources.iter().any(|source| normalized.starts_with(source))
        || path.file_name().is_some_and(|name| {
            matches!(
                name.to_str(),
                Some("Cargo.toml" | "Cargo.lock" | "build.rs" | "rust-toolchain.toml")
            )
        })
}

/// FSEvents may report /var while Cargo resolves /private/var; normalize existing ancestors even on deletion.
fn normalized_path(path: &std::path::Path) -> PathBuf {
    let mut ancestor = path;
    let mut missing = vec![];
    loop {
        if let Ok(mut normalized) = ancestor.canonicalize() {
            for suffix in missing.iter().rev() {
                normalized.push(suffix);
            }
            return normalized;
        }
        let Some(name) = ancestor.file_name() else {
            return path.to_path_buf();
        };
        missing.push(name.to_owned());
        let Some(parent) = ancestor.parent() else {
            return path.to_path_buf();
        };
        ancestor = parent;
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    /// Canonical watcher paths must match Cargo source roots reached through a symlink, including deletions.
    #[test]
    fn source_events_match_symlinked_roots_after_file_removal() {
        let root = std::env::temp_dir().join(format!("inrux-watch-test-{}", std::process::id()));
        std::fs::create_dir_all(root.join("real/src")).unwrap();
        let alias = root.join("alias");
        std::os::unix::fs::symlink(root.join("real"), &alias).unwrap();
        let source = alias.join("src/main.rs");
        std::fs::write(&source, "fn main() {}\n").unwrap();
        let canonical = source.canonicalize().unwrap();
        let sources = vec![normalized_path(&alias.join("src"))];
        assert!(relevant(&canonical, &sources));
        std::fs::remove_file(source).unwrap();
        assert!(relevant(&canonical, &sources));
        assert!(!relevant(&root.join("real/started.txt"), &sources));
        std::fs::remove_dir_all(root).unwrap();
    }
}
