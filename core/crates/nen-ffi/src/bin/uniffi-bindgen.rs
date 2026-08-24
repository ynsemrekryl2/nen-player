//! Binding generator entry point.
//!
//! Kept inside the workspace so that binding generation needs no separately
//! installed tool — see `scripts/build-apple.sh`.
fn main() {
    uniffi::uniffi_bindgen_main()
}
