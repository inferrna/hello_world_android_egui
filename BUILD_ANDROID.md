## Tools to be installed

1. `cargo install cargo-apk`
2. `rustup target add aarch64-linux-android`
3. Android NDK version 25
4. Android SDK components:
- SDK version 26
- platform version 25 (matches NDK version)
- build tools
- platform tools

## To build
`ANDROID_HOME=your_sdk_home NDK_HOME=your_ndk_home cargo apk build`

Where *your_sdk_home* must contain `build-tools` directory and *your_ndk_home* must contain `ndk-*` binaries: `ndk-build` and other.