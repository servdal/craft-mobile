# CADCraft — iOS mobile port

Native entry point: `apps/cadcraft-ios`. This folder is the mobile derivative; the original desktop repository remains separate. RoboDhan integration has been removed from this copy.

Keep the parent `craft-mobile` structure, including `mobile-shell`, `mobile-files` and `mobile-platform`. On macOS with Xcode and Rust 1.95+, run:

```sh
rustup target add aarch64-apple-ios-sim aarch64-apple-ios
cd ~/htdocs/craft-mobile
python3 packaging/ios/build.py cadcraft
# Physical-device architecture, unsigned:
python3 packaging/ios/build.py cadcraft --device --release
```

Simulator output is in `../dist/simulator/`. Device output in `../dist/device-unsigned/` still requires Apple signing and provisioning before installation. Read the [parent guide](../README.md) for simulator installation, file import/export, limitations and signing requirements. The upstream README below describes desktop/web behavior and is not a promise of iOS feature parity.

For USB installation using an existing Apple development profile, build with `--device`, then use `../packaging/ios/deploy.py` from the parent mobile directory. See the parent guide's signing/deployment section. The helper creates signed device `.app` / `.ipa` outputs and verifies installation; simulator applications cannot be installed on a physical iPhone.

## Files access and launcher icon

Version 0.1.1 (build 2) includes a native Files picker for Open and Export to Files. Save As saves locally first; export copies the saved file to your chosen Files destination. Documents are imported as copies, preserving their external originals. `packaging/ios/entitlements.plist` is the signing template for this app; deployment expands its identifiers from your Apple profile. The shared build helper compiles this app’s mascot AppIcon catalog. See the parent guide for permission details, icon sources and validation limits.

## Photos / Library

Version 0.1.2 (build 4) adds **Photos** to select an image and **Save to Photos** to add a locally saved PNG/JPEG/HEIC image to the library. Add-only permission is requested when saving. The parent guide documents each editor’s image insertion support, import normalization and remaining test scope. The picker provides selected images without requesting full-library read access.
