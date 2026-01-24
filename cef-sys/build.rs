fn main() {
    println!("cargo:rerun-if-env-changed=CEF_PATH");
    println!("cargo:rerun-if-env-changed=CEF_SYS_SKIP_LINK");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_MIRI");

    if std::env::var("CEF_SYS_SKIP_LINK").is_ok() || std::env::var("CARGO_CFG_MIRI").is_ok() {
        println!("cargo:warning=Skipping libcef.lib linkage (CEF_SYS_SKIP_LINK or Miri).");
        return;
    }

    let cef_redist = std::env::var("CEF_PATH")
        .expect("No CEF_PATH env variable. It should point to the libcef.lib.");

    println!("cargo:rustc-link-search=native={}", cef_redist);
    println!("cargo:rustc-link-lib=static=libcef");
}
