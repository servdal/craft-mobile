# Craft Mobile — native iOS ports

Seven separate source projects based on the local Craft applications: PhotoCraft, VectorCraft, PdfCraft, WordCraft, GridCraft, DeckCraft and CADCraft. Original projects in the parent folder are untouched. Exact upstream source revisions are recorded in `BASES.json`. These copies start before the RoboDhan integration; no local AI server, Python bridge or model download is required.

This is an initial native Rust/egui/wgpu mobile port, not a web wrapper. Editors retain their original document engines. `mobile-shell` includes a small original Objective-C UIKit scene delegate for current iOS lifecycle requirements and provides common touch-accessible file actions and `mobile-files` provides an asynchronous document browser confined to the application's Documents folder. Keep the complete folder structure: each project uses these two sibling libraries.

## Build on macOS

Install Xcode, select its developer tools, accept its license and install an iOS simulator runtime. Install Rust stable 1.95 or newer, then:

```sh
rustup target add aarch64-apple-ios-sim aarch64-apple-ios
cd ~/htdocs/craft-mobile
python3 packaging/ios/build.py
```

This compiles all seven simulator applications to `dist/simulator/*.app`. To build just one:

```sh
python3 packaging/ios/build.py wordcraft
```

The script uses locked dependency versions, writes individual build logs and `build-status.json`, and ad-hoc signs simulator applications. Initial builds need internet access for Rust dependencies. Running the finished application does not need RoboDhan.

Install into a booted Apple Silicon simulator:

```sh
xcrun simctl install booted dist/simulator/WordCraft.app
xcrun simctl launch booted com.duidev.craftmobile.wordcraft
```

`--release` builds optimized binaries. `--target-dir /absolute/cache/path` allows reuse of a shared Cargo cache.

## Physical iPhone / iPad

```sh
python3 packaging/ios/build.py --device --release
```

Output is `dist/device-unsigned/*.app`, compiled for aarch64-apple-ios. Use the signing/deployment helper below with an existing Apple development identity and provisioning profile to create installable device bundles and IPAs. Windows cannot perform this native iOS build: use a Mac with Xcode.

## Sign and install on a connected physical device

A valid Apple development identity and an iOS provisioning profile authorizing the connected device and each application bundle ID are required. Build physical-device binaries first:

```sh
python3 packaging/ios/build.py --device
python3 packaging/ios/deploy.py --device YOUR_DEVICE_UDID --profile /absolute/path/development.mobileprovision
```

`deploy.py --wait` can run while the device build is in progress. It selects a matching local signing identity, validates the profile/device/bundle IDs, signs each native device bundle, creates a development IPA, installs it with Xcode's devicectl and verifies its presence. Signed bundles, IPAs and deployment logs are in `dist/device/`. These development IPAs are limited to devices authorized by the profile. Unlock the iPhone when launching an application for testing.

## Documents and current limitations

Use the iOS Files app, under On My iPhone / On My iPad → the application, to copy documents into its Documents folder. Then use Open inside the editor. Save As writes to that folder; export/share saved files through Files. Existing files require explicit overwrite confirmation in the shared browser. User data must be saved before terminating the app; full background recovery has not been implemented in the common mobile host.

The existing editor ribbons are retained and are best evaluated on iPad first. This is not yet a completed phone-specific interface. Desktop-only integration (printing, external processes, system clipboard, media playback, screen capture, some import/export dialogs) is not guaranteed on iOS. Use the common mobile toolbar for Open / Save As; synchronous desktop file pickers are disabled rather than repurposed for unrelated document actions. Multi-file dialogs currently select one document per operation. No standalone AI features are included. Simulator compile success alone does not establish complete editing or device compatibility.

## Source and licensing

Every application retains its upstream README, contribution rules, source notices, assets and licenses. See each project's `README-IOS.md` for its mobile entry point. New common host/file code is licensed MIT OR Apache-2.0. No system fonts, AI models or proprietary assets are added. Shared code originates in this mobile port; original engine dependencies retain their own licenses.

## Lifecycle compatibility

Apple requires scene lifecycle adoption for applications built against the iOS 27 SDK. winit 0.30 creates a native window and observes application notifications, so `mobile-shell/ios/CraftSceneDelegate.m` attaches that existing window to the single application scene. The bundle declares the scene in Info.plist. See [Apple scene migration guidance](https://developer.apple.com/documentation/uikit/transitioning-to-the-uikit-scene-based-life-cycle) and [winit issue 4224](https://github.com/rust-windowing/winit/issues/4224). This adapter is original platform integration code, not a separate running bridge.

For startup QA in a dedicated, booted simulator (not a device containing personal test data):

```sh
python3 packaging/ios/smoke.py SIMULATOR_UUID --label ipad
```

The script installs and launches each simulator app, checks for a running process, captures its screen and closes it. Results are recorded in `dist/simulator/qa/<label>/launch-status.json`. Startup QA does not cover all editing, file formats, background restoration or touch interactions.

## Verified build status (2026-10-25)

All seven simulator .app bundles were built and opened successfully on iPhone 17 and iPad Pro 11-inch (M5) simulators with iOS / iPadOS 27.0. All seven iOS host Clippy checks passed, along with 77 architecture tests and five file/host-boundary tests. See [verification/README.md](verification/README.md) for exact scope, logs and remaining work. Physical-device deployment uses `packaging/ios/deploy.py`; current installation results are recorded in `dist/device/deployment-status.json`. This is an initial mobile port; phone layout and full editing workflows need further testing.

Temporary compilation caches were cleaned after validation. The first full rebuild may take considerable time. To rerun the architecture checks without compiling unrelated xtask format/oracle dependencies:

```sh
cargo test --manifest-path tools/layer-gate/Cargo.toml
cargo run --manifest-path tools/layer-gate/Cargo.toml
```

## Physical deployment completed (2026-10-25)

PhotoCraft, VectorCraft, PdfCraft, WordCraft, GridCraft, DeckCraft and CADCraft were built for physical iOS, Apple-development-signed and installed successfully on the connected iPhone 14 Pro running iOS 27.0.1. GridCraft and PhotoCraft were launched and their running processes confirmed. Signed `.app` bundles and `.ipa` files are in `dist/device/`. See [the physical deployment report](verification/DEVICE-DEPLOYMENT.md) for precise validation scope. These remain development builds of the initial mobile ports.
