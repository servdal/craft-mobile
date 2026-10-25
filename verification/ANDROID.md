# Android release verification — 2026-10-25

All seven standalone native mobile ports produced signed **Android App Bundles and APKs**, version **0.1.2 / versionCode 4**. Native ABI: arm64-v8a; min SDK 29 (Android 10); target SDK 36. No store upload was performed.

| Application | Native release | Signed AAB/APK validation | Emulator startup |
|---|---|---|---|
| PhotoCraft | Passed | Passed | Passed |
| VectorCraft | Passed | Passed | Passed |
| PdfCraft | Passed | Passed | Passed |
| WordCraft | Passed | Passed | Passed |
| GridCraft | Passed | Passed | Passed |
| DeckCraft | Passed | Passed | Passed |
| CADCraft | Passed | Passed | Passed |

## Implementation

Android wrappers under android-apps/<project> reuse each iOS adapter’s Rust editor source, retaining the corrected PhotoCraft UI file-command dispatcher. Unique native output names prevent cross-editor cache collisions before each is staged as libcraft.so. eframe/android-activity use GameActivity 4.4.0 with AppCompat. Shared JNI code retains the actual AndroidApp Activity reference rather than treating the application Context as the Activity. System-bar/cutout padding keeps the common toolbar clear; status/navigation backgrounds preserve icon contrast.

Open/Export use Android document intents and app-private copies. Photos uses the system picker (or image chooser on older supported Android versions); PNG normalization supports the existing engines. Save to Photos adds an explicitly selected local image through MediaStore. These flows do not request broad storage or whole-library read permission. PdfCraft uses the shared mobile file-picker facade and excludes desktop clipboard/quit hooks on Android. Its Android UI uses neutral PdfCraft/PDF labels instead of upstream trademark artwork, following the repository brand terms.

Release Rust uses optimization level 2, no LTO, 16 codegen units, unwind panic handling and debuginfo stripping. Gradle removes non-loaded symbols; verification confirms the loaded code/data matches each named editor library. Java minification is disabled for this initial JNI port. The user’s root ProGuard file contains Flutter/plugin rules; it remains unchanged as optional release configuration, alongside app-local JNI keep rules. These applications do not use Flutter.

## Checks completed

- All seven locked native release builds and final Gradle signed release packages passed.
- Google bundletool 1.18.3 validated each AAB; jarsigner verified each AAB signature.
- apksigner verified each APK; all use the same release certificate.
- Package IDs, release/non-debuggable state, ARM64 ABI, SDK levels and native editor identity verified.
- Native ELF load segments and installable APK ZIP alignment verified at 16 KiB.
- Source-stage native libraries match named outputs. Packaged libraries have matching loaded machine code/data after Gradle symbol stripping; AAB and APK native libraries match.
- No private keystore or key.properties entries in AAB/APK archives. Local signing files are Git-ignored and owner-readable only. No credentials printed or copied to release archives.
- All seven APKs installed/launched on the dedicated Craft_QA_API36 emulator (Android 16/API 36, ARM64, SwiftShader); running processes and all seven screenshots checked.
- GridCraft native Files picker appeared. A mascot icon selected from the system Photos picker was copied, inserted and visibly rendered in a sheet. Save to Photos succeeded; MediaStore confirmed Pictures/CraftMobile/Photo.png. No personal photo was retrieved.
- Three shared file-layer tests passed. The locked GridCraft iOS compatibility cargo check passed after shared-library changes.
- Original desktop repositories for all seven apps remained clean.

Evidence: ../dist/android/release-status.json, validation.json, SHA256SUMS.txt, per-app Rust/Gradle/signature/bundle logs; ../dist/android/qa/launch-status.json, file-photos-status.json, screenshots and MediaStore evidence. Shared test log: files-update-tests.log; iOS compatibility: android-ios-compat-check.log.

## Scope and limits

No physical Android device was connected, so physical-device compatibility and performance are unverified. The emulator uses 4096-byte runtime pages; a 16 KiB device runtime was not tested. Device-level whole-editor workflows, all file formats, rotation, failure/denial handling and background recovery remain incomplete. The interfaces retain broad desktop ribbons/panels: portrait screenshots show crowding and clipping, particularly in PhotoCraft, VectorCraft, CADCraft and PdfCraft. This is an initial port, not a complete phone UI redesign or full feature-parity certification. Tablet/landscape usability has not been validated in this run. PdfCraft/CADCraft Photos actions stage images only; direct toolbar insertion is not implemented.

Non-blocking upstream/compiler warnings remain (including three unused desktop-clipboard helpers in PdfCraft’s mobile build and Gradle deprecations). They did not prevent release packaging, validation or startup. No unrelated upstream warning cleanup was performed.

Build/reproduction guide: ../packaging/android/README.md; per-editor README-ANDROID.md files point to it.
