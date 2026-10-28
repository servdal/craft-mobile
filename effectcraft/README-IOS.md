# EffectCraft — iOS mobile port

Native entry point: `apps/effectcraft-ios`. This mobile copy is independent of the desktop source and contains no RoboDhan integration.

```sh
rustup target add aarch64-apple-ios-sim aarch64-apple-ios
cd ~/htdocs/craft-mobile
python3 packaging/ios/build.py effectcraft
python3 packaging/ios/build.py effectcraft --device --release
```

The simulator bundle is written to `../dist/simulator/`; a device bundle is written to `../dist/device-unsigned/` and must be signed with an Apple development profile using the parent deployment guide. The shared toolbar exposes native Files import/export; complex effects should be tested on target hardware.
