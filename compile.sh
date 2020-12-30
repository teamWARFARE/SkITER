#!/usr/bin/env bash
cargo build --release
cp -f target/release/libuniffi_skiter.so kotlin/src/main/resources/linux-x86-64/
uniffi-bindgen generate src/skiter.idl -l kotlin
cp -f -r src/army kotlin/src/main/kotlin/
cd kotlin
./gradlew
cd ..
cp -f kotlin/build/libs/skiter-1.0-SNAPSHOT.jar ./
