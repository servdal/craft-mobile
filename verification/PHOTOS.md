# Photos / Library update — 2026-10-25

Final device version: **0.1.2 (build 4)** for all seven Craft applications.

## Implemented

The shared native platform adapter provides a single-image PHPickerViewController, coordinated async result delivery, orientation-correct PNG import into an isolated Documents/Imports folder, and explicit add-only Photos saving. The common toolbar exposes Photos and Save to Photos. Info.plist includes NSPhotoLibraryAddUsageDescription. No full-library read authorization, camera access, Python bridge or local AI dependency is introduced. The picker only supplies the image selected by the user.

PhotoCraft opens the imported image. VectorCraft places it; WordCraft, GridCraft and DeckCraft route to insert.picture. PdfCraft and CADCraft currently stage the selected photo in Documents/Imports and explain that toolbar insertion is unsupported. Save to Photos chooses an existing PNG/JPEG/HEIC/HEIF file, validates it before requesting authorization, adds a new Photos asset, and reports denial/save failure. It does not automatically export the active editor canvas.

Imports normalize to PNG, up to 8192 px per side and 64 MiB encoded. Larger dimensions are downsampled. Original Photos assets remain unchanged; RAW metadata, Live Photo motion and original encoding are not preserved in this derived editing copy.

## Device findings and fix

The user first confirmed that the Photos gallery appeared, but reported: “photo copied to documen, but the editor could not insert it : unkown command 'file.open'”. Application-container metadata confirmed that the selected image had been copied; the user’s photo contents were not retrieved. PhotoCraft’s mobile adapter had routed UI file actions directly to the engine-only run method. It now calls the existing UI menus::invoke dispatcher, which handles file.open and the other UI file actions.

After the fixed application was installed and relaunched, the user confirmed: **“Foto sudah terbuka di editor.”** This verifies gallery presentation, selection, import and opening in PhotoCraft on the physical iPhone 14 Pro / iOS 27.0.1.

## Validation

- All seven native device builds completed; all seven final packages signed and installed.
- Final version, signing entitlements, Photos usage message, compiled icon catalog, strict signature verification and IPA integrity are checked in ../dist/device/device-validation.json.
- Native Objective-C source compiled for physical iOS. Clang static analysis reported zero diagnostics.
- Three shared file-layer tests passed; see files-update-tests.log.
- PhotoCraft’s targeted regression test mobile_open_routes_through_ui_file_import passed (1 passed, 0 failed), using an original synthetic 8×8 PNG rather than a personal photo; see photocraft-mobile-import-test.log.

Interactive Save to Photos authorization/denial and library additions, imports into the other editors, full document editing and retained simulator builds are not covered by this device confirmation. This update does not implement custom album creation or direct photo insertion in PdfCraft/CADCraft.

Apple API references: [PHPickerViewController](https://developer.apple.com/documentation/photosui/phpickerviewcontroller), [add-only usage description](https://developer.apple.com/documentation/bundleresources/information-property-list/nsphotolibraryaddusagedescription).

The broader strict Clippy run encountered an existing nonminimal_bool lint at photocraft/crates/ui-egui/src/canvas.rs:4415 (unmodified by this update). See photocraft-photos-clippy.log. This is a lint failure, not a device build or runtime failure. The separate --no-deps Clippy check of the updated mobile adapter passed with -D warnings; see photocraft-photos-adapter-clippy.log.

Reproduce the regression test from the photocraft mobile workspace with `cargo test --locked -p photocraft-ios mobile_open_routes_through_ui_file_import`. Original desktop repositories were checked again and remained clean.
