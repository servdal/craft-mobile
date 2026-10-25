fn main() {
    println!("cargo:rerun-if-changed=ios/CraftDocumentPicker.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("ios") {
        cc::Build::new()
            .file("ios/CraftDocumentPicker.m")
            .flag("-fobjc-arc")
            .cargo_metadata(false)
            .compile("craft_documents");
        if let Ok(out) = std::env::var("OUT_DIR") {
            println!("cargo:rustc-link-search=native={out}");
        }
        println!("cargo:rustc-link-lib=static:+whole-archive=craft_documents");
        println!("cargo:rustc-link-lib=framework=UIKit");
        println!("cargo:rustc-link-lib=framework=UniformTypeIdentifiers");
    }
}
