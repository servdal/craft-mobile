#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path="../../../photocraft/xtask/src/layers.rs"] mod photocraft;
#[allow(dead_code)]
#[path="../../../vectorcraft/xtask/src/layers.rs"] mod vectorcraft;
#[allow(dead_code)]
#[path="../../../pdfcraft/xtask/src/layers.rs"] mod pdfcraft;
#[allow(dead_code)]
#[path="../../../wordcraft/xtask/src/layers.rs"] mod wordcraft;
#[allow(dead_code)]
#[path="../../../gridcraft/xtask/src/layers.rs"] mod gridcraft;
#[allow(dead_code)]
#[path="../../../deckcraft/xtask/src/layers.rs"] mod deckcraft;
#[allow(dead_code)]
#[path="../../../cadcraft/xtask/src/layers.rs"] mod cadcraft;

fn main() -> Result<(), Box<dyn std::error::Error>> {
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 let mut failed=false;
 macro_rules! inspect { ($name:ident) => {{
  let name=stringify!($name);
  let output=std::process::Command::new("cargo").args(["metadata","--offline","--no-deps","--format-version","1"]).current_dir(root.join(name)).output()?;
  if !output.status.success() { return Err(format!("{name}: {}",String::from_utf8_lossy(&output.stderr)).into()); }
  let metadata=serde_json::from_slice(&output.stdout)?;
  let crates=$name::from_metadata(&metadata)?;
  let violations=$name::check(&crates);
  if violations.is_empty() { println!("{name}: architecture PASS"); }
  else { failed=true;for violation in violations {eprintln!("{name}: {violation}");} }
 }}; }
 inspect!(photocraft);
 inspect!(vectorcraft);
 inspect!(pdfcraft);
 inspect!(wordcraft);
 inspect!(gridcraft);
 inspect!(deckcraft);
 inspect!(cadcraft);
 if failed {Err("Architecture violations found".into())} else {Ok(())}
}
