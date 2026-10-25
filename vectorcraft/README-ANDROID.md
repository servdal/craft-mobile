# VectorCraft — Android mobile release

This is the mobile derivative; the original desktop project is separate. Keep the complete parent craft-mobile tree. The Android native wrapper is `../android-apps/vectorcraft` and reuses this project’s mobile editor adapter.

From the parent craft-mobile directory on a Mac with Rust 1.95+, Java 17, Android SDK 36 and NDK 28.2.13676358:

```sh
rustup target add aarch64-linux-android
python3 packaging/android/build.py vectorcraft --apk
```

Provide the existing private keystore and key.properties locally at the craft-mobile root, or use --key-properties. They are ignored by Git; never commit signing passwords or keystores. No new signing key is generated.

Outputs: `../dist/android/VectorCraft-release.aab` for Play Console and `VectorCraft-release.apk` for direct installation. Supported initial target: ARM64 Android 10+ (API 29), target SDK 36. The AAB is a bundle rather than a directly installable APK. No Google Play upload is performed.

Read the [shared Android guide](../packaging/android/README.md) for setup, signing, Files/Photos behavior, build flags and validation limits. See [Android verification](../verification/ANDROID.md) for tested scope.
