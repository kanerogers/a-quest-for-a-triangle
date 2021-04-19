use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let android = target.contains("android");

    // Export shared libraries search path.
    if android {
        println!(
            "cargo:rustc-link-search={}\\src\\libs",
            env!("CARGO_MANIFEST_DIR")
        );
        println!("cargo:rustc-link-lib=dylib=VkLayer_core_validation");
    }
}
