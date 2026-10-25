# Files access and mascot launcher update — 2026-10-25

All seven native physical-device applications were rebuilt, signed, packaged as development IPAs, installed on the connected iPhone 14 Pro (iOS 27.0.1), and confirmed as version **0.1.1, build 2**.

## Implemented

- Persistent per-project `packaging/ios/entitlements.plist` templates expanded from the existing Apple profile during signing. Expanded plists are retained under `dist/device/`.
- Native UIKit document picker for Open, importing a copy through coordinated reads and balanced security-scoped access. Imports remain confined to the app Documents directory; external originals are preserved.
- Save As keeps documents locally. Export to Files selects a saved local document and opens the native destination picker. Files sharing remains enabled through Info.plist. A Projects folder makes the app Documents location available after launch.
- Separate user-mascot poses for all seven applications, opaque 1024 px masters, retained generated sources, full prompts, catalog size variants and compiled Assets.car / Info.plist metadata for both iPhone and iPad.

## Verified

- All seven final physical-device builds completed successfully.
- All seven strict Apple signature checks passed. Embedded signing entitlements matched each expanded template.
- All seven IPA archives passed integrity checks and included icon assets and matching bundle metadata.
- All seven installed application versions/builds confirmed through Xcode devicectl.
- GridCraft launched on the physical iPhone with its debug-only Files-opening QA flag and its process remained alive. The launch flag confirms the request was sent; it does not prove that the system picker was visible or a document was selected.
- Three file-layer tests passed: asynchronous selection, traversal rejection, confinement and atomic replacement.
- All seven original desktop repositories remained clean.

The user confirmed on the physical iPhone that GridCraft displayed the native Files picker and a document could be selected. Successful document decoding/editing, export destination selection, and other applications’ interactive file workflows remain unverified. The selected document provider was not specified. No broad filesystem entitlement is claimed: iOS grants document access through the system picker. The Files update has not been rerun on simulators; retained simulator evidence belongs to the earlier build.

Evidence: `../dist/device/device-validation.json`, `deployment-status.json`, per-app installed/signing logs, `gridcraft-files-picker-launch.json`, `gridcraft-files-process.json` and `files-update-tests.log`. Icon source and prompts: `../assets/launcher/README.md` and `prompts.json`.

User device confirmation: “Files tampil dan dokumen bisa dipilih.” This confirms picker presentation and document selection in GridCraft; it does not establish successful parsing or editing of the chosen document.
