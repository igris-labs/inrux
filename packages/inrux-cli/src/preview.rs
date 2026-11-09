//! Build-first preview replacement keeps the last good process alive after compilation errors.
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
};
type Error = Box<dyn std::error::Error>;
/// Ask Cargo for executable artifacts, preserving its rendered compiler diagnostics.
pub(crate) fn build(args: &[String]) -> Result<Option<PathBuf>, Error> {
    let mut child = Command::new("cargo")
        .args(["build", "--message-format=json-render-diagnostics"])
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut executables = BTreeSet::new();
    let output = child
        .stdout
        .take()
        .ok_or("Cannot capture Cargo build output")?;
    for line in BufReader::new(output).lines() {
        let line = line?;
        let Ok(message) = serde_json::from_str::<Value>(&line) else {
            eprintln!("{line}");
            continue;
        };
        if let Some(rendered) = message["message"]["rendered"].as_str() {
            eprint!("{rendered}");
        }
        if let Some(executable) = message["executable"].as_str() {
            executables.insert(PathBuf::from(executable));
        }
    }
    if !child.wait()?.success() {
        eprintln!("Build failed; the previous preview remains running.");
        return Ok(None);
    }
    if executables.len() != 1 {
        return Err(
            "Select one application with --bin <name> (and -p <package> in a workspace)".into(),
        );
    }
    Ok(executables.pop_first())
}
/// Own and reap the child process on restart and development-server shutdown.
#[derive(Default)]
pub(crate) struct Preview {
    child: Option<Child>,
    executable: Option<PathBuf>,
}
impl Preview {
    /// Restart only after the next executable has successfully compiled.
    pub(crate) fn replace(&mut self, executable: PathBuf, args: &[String]) -> Result<(), Error> {
        let copy = preview_copy(&executable)?;
        self.stop();
        self.executable = Some(copy.clone());
        self.child = Some(Command::new(copy).args(args).spawn()?);
        Ok(())
    }
    /// Reap a stopped child so repeated edits cannot leak zombie processes.
    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(path) = self.executable.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}
impl Drop for Preview {
    /// Ctrl-C and error paths use the same process cleanup as a successful restart.
    fn drop(&mut self) {
        self.stop();
    }
}

/// A distinct executable copy lets Windows rebuild the Cargo artifact while the previous preview is running.
fn preview_copy(source: &std::path::Path) -> Result<PathBuf, Error> {
    use std::{
        fs::{File, OpenOptions},
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let extension = source.extension().map_or(String::new(), |extension| {
        format!(".{}", extension.to_string_lossy())
    });
    let path = source
        .parent()
        .ok_or("Executable has no directory")?
        .join(format!(
            "inrux-preview-{}-{}{extension}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    let mut input = File::open(source)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    if let Err(error) = std::io::copy(&mut input, &mut output)
        .and_then(|_| std::fs::set_permissions(&path, input.metadata()?.permissions()))
    {
        drop(output);
        let _ = std::fs::remove_file(&path);
        return Err(error.into());
    }
    Ok(path)
}
