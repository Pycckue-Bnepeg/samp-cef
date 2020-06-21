fn main() {
    println!("cargo:rustc-link-search=native=D:\\sources\\c\\cef\\Release");
    println!("cargo:rustc-link-lib=static=libcef");
}
