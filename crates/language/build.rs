fn main() {
    if let Ok(bundled) = std::env::var("VELA_BUNDLE") {
        println!("cargo:rustc-env=VELA_BUNDLE={}", bundled);
    }
}
