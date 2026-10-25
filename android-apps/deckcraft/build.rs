// SPDX-License-Identifier: MIT OR Apache-2.0
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::path::Path::new("../../deckcraft/apps/deckcraft-ios/src/main.rs");
    println!("cargo:rerun-if-changed={}", source.display());
    let contents = std::fs::read_to_string(source)?;
    if !contents.contains("fn main()") {
        return Err("Mobile editor entry point is missing".into());
    }
    let contents =
        contents
            .replace("#![forbid(unsafe_code)]", "")
            .replacen("fn main()", "pub fn start()", 1);
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR")?).join("editor.rs"),
        contents,
    )?;
    Ok(())
}
