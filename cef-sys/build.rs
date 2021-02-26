fn main() {
    println!("cargo:rustc-link-search=native=E:\\sources\\c\\cef\\chromium\\src\\cef\\binary_distrib\\cef_binary_89.0.5+gc1f90d8+chromium-89.0.4389.40_windows32_minimal\\Release");
    println!("cargo:rustc-link-lib=static=libcef");
}
