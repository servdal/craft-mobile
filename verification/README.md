> **Subsequent physical-device deployment:** All seven apps were later installed on iPhone 14 Pro. See [DEVICE-DEPLOYMENT.md](DEVICE-DEPLOYMENT.md) for device-specific evidence. This page records the preceding simulator stage.

# Craft Mobile verification — 2026-10-25

All seven native iOS simulator applications were built, ad-hoc signed, signature-verified, installed, launched and captured on both a dedicated iPad Pro 11-inch (M5) simulator and an iPhone 17 simulator, running iOS / iPadOS 27.0. Xcode 27.1 / SDK 27.1, Rust 1.95, target aarch64-apple-ios-sim, minimum deployment target 16.0.

- `build-results.json`: final bundle locations, binary SHA-256 digests, signing and startup status.
- `../dist/simulator/qa/ipad/` and `iphone/`: screenshots and launch reports for the delivered binaries.
- `host-clippy-status.json`: strict Clippy for all seven iOS host packages, all targets, with dependencies excluded from linting. All passed. PdfCraft's UI dependency still emits three iOS-specific dead-code warnings; those are not warnings in the host package.
- `layer-module-tests.log`: 77 upstream architecture-rule unit tests passed, using the exact source modules from each project's xtask.
- `layer-module-checks.log`: all seven actual Cargo workspace dependency graphs passed those rules. The lightweight driver lives in `../tools/layer-gate/`.
- `file-browser-tests.log`: three tests passed for path/name confinement, atomic replacement and asynchronous picker completion.
- `host-guard-tests.log`: two tests passed for escaped callback panic handling and normal error/result preservation.
- `pdfcraft-layer-tests.log` / `pdfcraft-layers.log`: corrected PdfCraft classification also passed its ordinary xtask checks.
- `../ORIGINAL-STATUS.json`: all seven original desktop repositories remained clean.

`initial/` contains earlier diagnostic results, including failures subsequently corrected. It is not the final verification status.

This is startup and host verification, not a complete editing/file-format/touch regression suite. The editors still use their desktop-oriented ribbons; phone layout, background recovery, clipboard, media and other desktop-only services need further mobile work. Physical-device compilation, provisioning, signed IPA export and installation on an actual iPhone / iPad were not verified. The delivered .app bundles are for Simulator only. No RoboDhan, Python runtime bridge or AI model is bundled.

Temporary compilation caches created for this work were removed after preserving the application bundles, screenshots and logs. Future builds create a fresh ignored `build/rust` cache inside the mobile tree; downloaded Cargo source dependencies remain available in the normal Cargo cache.
