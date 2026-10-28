fn main() {
    println!("cargo:rerun-if-changed=ios/CraftSceneDelegate.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("ios") {
        cc::Build::new()
            .file("ios/CraftSceneDelegate.m")
            .flag("-fobjc-arc")
            .cargo_metadata(false)
            .compile("craft_mobile_scene");
        let out = std::env::var("OUT_DIR").unwrap_or_default();
        println!("cargo:rustc-link-search=native={out}");
        // A scene class is referenced by name from Info.plist, not by a Rust symbol.
        println!("cargo:rustc-link-lib=static:+whole-archive=craft_mobile_scene");
        println!("cargo:rustc-link-lib=framework=UIKit");
    }
}
