/// Compile the reachable component graph and both themes before application startup.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    inrux_compiler::build_app("src/App.iru", &out)?;
    let light = std::fs::read_to_string("src/styles/global.css")?;
    let dark = std::fs::read_to_string("src/styles/dark.css")?;
    println!("cargo:rerun-if-changed=src/styles/global.css");
    println!("cargo:rerun-if-changed=src/styles/dark.css");
    for (name, css) in [
        ("light", light.clone()),
        ("dark", format!("{light}\n{dark}")),
    ] {
        let compiled = inrux_compiler::compile_stylesheet(&css)
            .map_err(|error| format!("{name} stylesheet: {error}"))?;
        std::fs::write(out.join(format!("{name}_styles.rs")), compiled)?;
    }
    Ok(())
}
