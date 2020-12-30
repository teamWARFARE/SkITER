# Prerequisites

## The uniffi-bindgen cli tool

Install the `uniffi-bindgen` binary on your system using:

`cargo install uniffi_bindgen`

You can see what it can do with `uniffi-bindgen --help`, but let's leave it aside for now.

## Build your crate as a cdylib

Ensure your crate builds as a `cdylib` by adding
```toml
crate-type = ["cdylib"]
name = "<library name>"
```
to your crate's `Cargo.toml`.
