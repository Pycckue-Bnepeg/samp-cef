fn main() {
    println!("cargo:rustc-link-search=native=C:\\Users\\zottc\\cef");
    println!("cargo:rustc-link-lib=static=libcef");
}
