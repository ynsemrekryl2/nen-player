//! Binding generator entry point for the spike.
//!
//! Mirrors `nen-ffi`'s: kept inside the workspace so binding generation needs
//! no separately installed tool — see `scripts/spike-reverse-ffi.sh`.
fn main() {
    uniffi::uniffi_bindgen_main()
}
