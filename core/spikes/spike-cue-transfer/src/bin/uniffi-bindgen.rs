//! Binding generator entry point for the spike.
//!
//! Mirrors `nen-ffi`'s: kept inside the workspace so binding generation needs
//! no separately installed tool — see `scripts/spike-cues.sh`.
fn main() {
    uniffi::uniffi_bindgen_main()
}
