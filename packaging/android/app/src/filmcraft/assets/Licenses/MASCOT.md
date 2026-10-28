# Mobile mascot launcher icons

User-supplied identity reference: `mascot-reference.png` (original `mascotonly.png`, provided by @servdal). Generated with the built-in image_gen tool, preserving the monitor head, navy face, gear eyes, cyan pixel pupils, gray torso, red center button and yellow arms. Each application has a matching function pose; see `prompts.json`. Full-resolution generated sources are retained alongside normalized 1024 px masters. These are opaque square images; iOS applies the icon mask. Xcode actool compiles the icon catalogs in `../../packaging/ios/Assets/`.

Prompt set: identity-preserving iOS launcher icon; camera + brush for PhotoCraft, stylus + Bezier nodes for VectorCraft, red folded document for PdfCraft, blue pen + page for WordCraft, green worksheet for GridCraft, orange slide/chart for DeckCraft, drafting compasses + blueprint for CADCraft. No text/watermark; retain the supplied mascot identity, black outlines and original colors, close upper-body framing, full-bleed function-colored background. The built-in tool was used, not the API/CLI fallback.

Artwork supplied/commissioned by the user for their application family. No public redistribution license is inferred for the mascot.

## FilmCraft, EffectCraft, and LightCraft replacements

`FilmCraft.jpg`, `EffectCraft.jpg`, and `LightCraft.jpg` were supplied by @servdal on 2026-10-28 as the approved launcher artwork for those three apps. Their normalized masters are `filmcraft-1024.png`, `effectcraft-1024.png`, and `lightcraft-1024.png`; the build assets in `../../packaging/ios/Assets/` and `../../packaging/android/app/src/` are derived directly from them. LightCraft's portrait source is center-cropped to a square before resizing, so it meets the iOS and Android icon requirements without distortion. No public redistribution license is inferred for these supplied artworks.
