#!/usr/bin/env bash
rm -rf java/src/main/java/army
mkdir -p java/src/main/java/army/warfare/skiter
touch src/java_glue.rs.in
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
cp -f target/release/libuniffi_skiter.so java/src/main/resources/linux-x86-64/
cp -f target/x86_64-pc-windows-gnu/release/uniffi_skiter.dll java/src/main/resources/win32-x86-64/
cd java
./gradlew build
cd ..
cp -f java/build/libs/skiter-1.0-SNAPSHOT.jar ./
