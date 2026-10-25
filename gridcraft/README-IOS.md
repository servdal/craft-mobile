# GridCraft — iOS mobile port

Native entry point: `apps/gridcraft-ios`. This folder is the mobile derivative; the original desktop repository remains separate. RoboDhan integration has been removed from this copy.

Keep the parent `craft-mobile` structure, including `mobile-shell` and `mobile-files`. On macOS with Xcode and Rust 1.95+, run:

```sh
rustup target add aarch64-apple-ios-sim aarch64-apple-ios
cd ~/htdocs/craft-mobile
python3 packaging/ios/build.py gridcraft
# Physical-device architecture, unsigned:
python3 packaging/ios/build.py gridcraft --device --release
```

Simulator output is in `../dist/simulator/`. Device output in `../dist/device-unsigned/` still requires Apple signing and provisioning before installation. Read the [parent guide](../README.md) for simulator installation, file import/export, limitations and signing requirements. The upstream README below describes desktop/web behavior and is not a promise of iOS feature parity.

For USB installation using an existing Apple development profile, build with `--device`, then use `../packaging/ios/deploy.py` from the parent mobile directory. See the parent guide's signing/deployment section. The helper creates signed device `.app` / `.ipa` outputs and verifies installation; simulator applications cannot be installed on a physical iPhone.
