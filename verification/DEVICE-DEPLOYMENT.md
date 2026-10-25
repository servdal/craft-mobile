# Physical iPhone deployment — 2026-10-25

All seven applications were compiled for `aarch64-apple-ios`, signed with an existing Apple Development identity and a profile authorizing the connected device, installed over USB and confirmed in the physical device's application list.

Device: **iPhone 14 Pro, iOS 27.0.1**. Application version: **0.1.2 (4)**.

| Application | Native device build | Apple signature / IPA validated | Installed |
|---|---|---|---|
| PhotoCraft | Passed | Passed | Yes |
| VectorCraft | Passed | Passed | Yes |
| PdfCraft | Passed | Passed | Yes |
| WordCraft | Passed | Passed | Yes |
| GridCraft | Passed | Passed | Yes |
| DeckCraft | Passed | Passed | Yes |
| CADCraft | Passed | Passed | Yes |

GridCraft and PhotoCraft were also launched on the physical iPhone; their processes remained present after launch. Full interactive editing, every file format, background recovery and all mobile layouts have not been verified. The earlier simulator stage tested startup of all seven apps on both iPhone and iPad simulators.

Signed device applications, development IPAs and machine-readable evidence are in `../dist/device/`:

- `deployment-status.json`: installation results.
- `installed-craft-apps.json`: Xcode's on-device application listing.
- `device-validation.json`: native platform, signature, IPA integrity and binary hashes.
- Per-application install / installed / deployment logs.
- GridCraft and PhotoCraft launch / process reports.

These are development builds using the existing certificate/profile. Simulator outputs remain separately in `../dist/simulator/`. No RoboDhan runtime or separate Python AI bridge is required on the phone. Build/deployment scripts run on the Mac.

The standalone mobile sources and signing instructions are in the parent README. All seven original desktop repositories remained clean.

The Files / mascot icon update is documented in [FILES-ICONS.md](FILES-ICONS.md). The latest device validation concerns version 0.1.1 (2); retained earlier simulator/startup evidence has its own scope.

Photos / Library support and the verified PhotoCraft import fix are recorded in [PHOTOS.md](PHOTOS.md). Current device-validation.json refers to version 0.1.2 (4); earlier Files/icon evidence is retained as historical scope.
