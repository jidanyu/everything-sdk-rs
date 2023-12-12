fn main() {
    println!("cargo:warning=Hello everything-sdk-sys!");
    #[cfg(windows)]
    {
        let vendored = std::env::var("CARGO_FEATURE_VENDORED").is_ok();
        let link_dll = std::env::var("CARGO_FEATURE_DLL").is_ok();

        assert!(
            vendored,
            "now only support build everything-sdk from source code"
        );
        assert!(!link_dll, "now only support link everything-sdk in static");

        // now the rerun settings are by default
        // Ref: https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed
        // println!("cargo:rerun-if-changed=Everything-SDK");

        // Build everything from source code
        cc::Build::new()
            .file("Everything-SDK/src/Everything.c")
            .compile("everything-sdk");

        // !Depr: build from source code
        // Tell cargo to look for shared libraries in the specified directory
        // println!("cargo:rustc-link-search=native=Everything-SDK");
        // println!("cargo:rustc-link-lib=Everything64"); // for Everything64.lib

        // !Depr: dynamic link by windows-rs
        // Tell cargo to tell rustc to link the system user32 and shell32 shared library.
        // println!("cargo:rustc-link-lib=user32"); // for User32.lib
        // println!("cargo:rustc-link-lib=shell32"); // for shell32.lib
    }

    println!("cargo:warning=Goodbye everything-sdk-sys!");
}
