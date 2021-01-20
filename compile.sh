#!/usr/bin/env bash
./compress_ressources.sh
rm -rf java/src/main/java/army
mkdir -p java/src/main/java/army/warfare/skiter
touch src/java_glue.rs.in
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
cp -f target/release/libskiter.so java/src/main/resources/
cp -f target/x86_64-pc-windows-gnu/release/skiter.dll java/src/main/resources/
cd java
./gradlew build
cd ..
cp -f java/build/libs/skiter-1.0-SNAPSHOT.jar ./
