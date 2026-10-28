# FilmCraft — iOS mobile port

Native entry point: `apps/filmcraft-ios`. This mobile copy is independent of the desktop source and contains no RoboDhan integration.

```sh
rustup target add aarch64-apple-ios-sim aarch64-apple-ios
cd ~/htdocs/craft-mobile
python3 packaging/ios/build.py filmcraft
python3 packaging/ios/build.py filmcraft --device --release
```

The simulator bundle is written to `../dist/simulator/`; a device bundle is written to `../dist/device-unsigned/` and must be signed with an Apple development profile using the parent deployment guide. The shared toolbar exposes native Files import/export; video workflows are still subject to mobile hardware and codec support.
