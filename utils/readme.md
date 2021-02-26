`wrapper.h`: генерация биндингов для CEF

запускать из папки релиза CEF (либо в `-I` указать путь до нее)

команда: `bindgen wrapper.h -o lib.rs --rust-target nightly --default-enum-style=rust_non_exhaustive --whitelist-type cef_.* --whitelist-function cef_.*  --bitfield-enum .*_mask_t -- -I E:\sources\c\cef\chromium\src\cef\binary_distrib\cef_binary_89.0.5+gc1f90d8+chromium-89.0.4389.40_windows32_minimal`

`bindgen wrapper.h -o lib.rs --size_t-is-usize --rust-target nightly --default-enum-style=moduleconsts --whitelist-type cef_.* --whitelist-function cef_.*  --bitfield-enum .*_mask_t -- -I E:\sources\c\cef\chromium\src\cef\binary_distrib\cef_binary_89.0.5+gc1f90d8+chromium-89.0.4389.40_windows32_minimal -m32`