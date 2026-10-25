// SPDX-License-Identifier: MIT OR Apache-2.0
// The only unsafe marker in this wrapper exports the ABI required by android-activity.
mod editor {
    #![forbid(unsafe_code)]
    include!(concat!(env!("OUT_DIR"), "/editor.rs"));
}
#[unsafe(no_mangle)]
fn android_main(app: craft_mobile_shell::AndroidApp) {
    craft_mobile_shell::init_android(app);
    if let Err(error) = editor::start() {
        eprintln!("Craft startup failed: {error}");
    }
}
