mod editor { #![forbid(unsafe_code)] include!(concat!(env!("OUT_DIR"), "/editor.rs")); }
#[unsafe(no_mangle)]
fn android_main(app: craft_mobile_shell_v37::AndroidApp) { craft_mobile_shell_v37::init_android(app); if let Err(error) = editor::start() { eprintln!("Craft startup failed: {error}"); } }
