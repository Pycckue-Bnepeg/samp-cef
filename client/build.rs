fn main() {
    println!("cargo:rustc-link-search=native=C:\\Program Files (x86)\\Microsoft DirectX SDK (August 2007)\\Lib\\x86");
    println!("cargo:rustc-link-lib=static=d3dx9");
}