fn main() {
    let cef_redist = std::env::var("CEF_PATH")
        .expect("No CEF_PATH env variable. It should point to the libcef.lib.");

    println!("cargo:rustc-link-search=native={}", cef_redist);
    println!("cargo:rustc-link-lib=static=libcef");
}
