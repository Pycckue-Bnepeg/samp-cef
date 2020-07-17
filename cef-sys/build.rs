fn main() {
    println!("cargo:rustc-link-search=native=C:\\Users\\zottce\\cef");
    println!("cargo:rustc-link-lib=static=libcef");
}
