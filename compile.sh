#!/usr/bin/env bash
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
cp -f target/release/libuniffi_skiter.so kotlin/src/main/resources/linux-x86-64/
cp -f target/x86_64-pc-windows-gnu/release/uniffi_skiter.dll kotlin/src/main/resources/win32-x86-64/
rm -rf src/army
uniffi-bindgen generate src/skiter.idl -l kotlin
cp -f -r src/army kotlin/src/main/kotlin/
cd kotlin
./gradlew build
cd ..
cp -f kotlin/build/libs/skiter-1.0-SNAPSHOT.jar ./
