fn main() {
    println!("cargo:rustc-link-search=native=E:\\cef_redist");
    println!("cargo:rustc-link-lib=static=libcef");
}
