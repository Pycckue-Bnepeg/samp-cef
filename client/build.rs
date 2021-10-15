const DIRECTX_DEFAULT_PATH: &str =
    "C:\\Program Files (x86)\\Microsoft DirectX SDK (June 2010)\\Lib\\x86";

fn main() {
    let directx_sdk = std::env::var("DX_SDK").unwrap_or(DIRECTX_DEFAULT_PATH.to_string());

    println!("cargo:rustc-link-search=native={}", directx_sdk);
    println!("cargo:rustc-link-lib=static=d3dx9");
}
