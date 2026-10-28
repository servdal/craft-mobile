# Craft Mobile Android release

Native Rust/egui/wgpu Android applications based on the same mobile editors as iOS. Each app has a separate application ID and mascot icon. No RoboDhan service, Python bridge or model download is needed on Android. Build scripts run on the Mac; the installed apps contain native Rust libraries and Android platform integration.

The build supports PhotoCraft, VectorCraft, PdfCraft, WordCraft, GridCraft, DeckCraft, CADCraft, FilmCraft, EffectCraft and LightCraft.

## Requirements on macOS

- Rust stable 1.95+ and `rustup target add aarch64-linux-android`.
- Java 17, Android SDK platform 36 / build-tools 36.0.0 and NDK 28.2.13676358.
- The provided Gradle 8.14 wrapper downloads/verifies the official distribution if needed.
- `ANDROID_HOME` points to the SDK (default `~/Library/Android/sdk`); `ANDROID_NDK_HOME` can override the default NDK path.

The initial release supports **ARM64 Android 10+ (API 29)**, targets API 36, and uses 16 KiB native load alignment. Other ABIs and older Android versions are not included.

## Local signing files

Keep the user-provided keystore and `key.properties` locally at the craft-mobile root. They are ignored by Git and must not be added to commits. The properties file uses `storeFile`, `storePassword`, `keyAlias` and `keyPassword`. Relative storeFile paths are resolved against the properties directory; if that path is stale, a file with the same basename beside key.properties is used. No new signing key is generated. For a properties file elsewhere, pass `--key-properties /absolute/path/key.properties`. Do not put passwords on the command line.

## Build release bundles

From the parent craft-mobile directory:

```sh
export ANDROID_HOME="$HOME/Library/Android/sdk"
rustup target add aarch64-linux-android
python3 packaging/android/build.py --apk
# One application:
python3 packaging/android/build.py gridcraft --apk
```

Each app is compiled in release mode (optimization level 2, no LTO, unwind panic handling) with its own checked-in dependency lock, then packaged/signed through Gradle. Native source wrappers are under `android-apps/<project>/`; their build scripts reuse the corresponding iOS adapter source so editor behavior stays consistent. The isolated Android entry wrapper exports the required android_main ABI. Other editor code remains safe Rust.

Outputs: `dist/android/<App>-release.aab` for Play Console and, with `--apk`, `<App>-release.apk` for direct installation. An AAB cannot be installed directly as an APK. Logs and release-status.json are in the same directory. Nothing is uploaded to Google Play by this script.

`--native-only` compiles the Rust libraries. `--package-only --apk` packages previously built libraries; use it only after a successful current native build. `--offline` works when all dependencies and the Gradle distribution are cached. Reuse `build/android-rust` to avoid recompiling dependencies.

To install an APK on a connected Android device with USB debugging enabled:

```sh
"$ANDROID_HOME/platform-tools/adb" install -r dist/android/GridCraft-release.apk
```

## Files and Photos

Open uses the Android system document picker. Photos uses the platform photo picker on Android 13+ and an image chooser on Android 10–12. Selected documents are copied into the app’s private Documents/Imports folder; originals are preserved. Photo imports are normalized to an orientation-correct PNG, downsampled to at most 8192 pixels per side, and rejected above 64 MiB encoded. PhotoCraft opens the image; VectorCraft places it; WordCraft/GridCraft/DeckCraft use their existing picture insertion command. CADCraft/PdfCraft currently stage the photo only.

Save As writes a safe local copy, then opens Android's destination picker so the finished document can be saved to Documents, Downloads, a cloud provider, or another user-selected public folder through `ACTION_CREATE_DOCUMENT`. Export to Files selects an already-saved local document and copies it to a destination chosen through the same picker. Save to Photos adds a selected saved PNG/JPEG/HEIC image through MediaStore to Pictures/CraftMobile. Broad storage or entire-library read permissions are not requested. Cloud document providers may need network access through Android. Full archival photo metadata/RAW/Live Photo preservation is not claimed.

The existing editor ribbons remain broad desktop-style layouts. These are initial mobile ports; complete phone layout, editing, all file formats, rotation and background recovery require further device testing. Read ../../verification/ANDROID.md for exact validation evidence and limits.

## Platform dependencies

The wrapper uses android-activity 0.6.1 with GameActivity 4.4.0, as required by the upstream adapter. Gradle includes AppCompat and a Kotlin BOM to align transitive Android dependencies. The built-in native C++ glue is compiled by android-activity; Gradle prefab support is deliberately disabled to avoid linking a second glue layer. Existing upstream licenses/notices and mascot attribution remain applicable.

## Validation helpers

`python3 packaging/android/verify.py --bundletool /path/to/bundletool-all.jar` validates bundles, signatures, application IDs, native ARM64/16 KiB alignment and matching loaded editor code/data before/after Gradle strips non-loaded symbols. Obtain bundletool from the official Google bundletool releases (the initial validation used 1.18.3). `python3 packaging/android/smoke.py --serial emulator-5554` installs and launches the APKs on a dedicated emulator, captures screens and closes each app. It does not accept physical-device serials because it force-stops applications during testing.

The user-supplied root proguard-rules.pro contains Flutter/plugin rules and is retained unchanged as optional release configuration. These native Rust apps do not use Flutter. Java shrinking is currently disabled to keep the JNI integration straightforward; native Rust code is still release-optimized. The app-local ProGuard rules preserve JNI entry names if shrinking is enabled later.
