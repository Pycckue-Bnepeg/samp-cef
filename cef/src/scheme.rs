use crate::types::string::CefString;

pub fn register_custom_scheme(
    registrar: *mut cef_sys::cef_scheme_registrar_t, scheme_name: &str, options: i32,
) -> bool {
    if registrar.is_null() {
        return false;
    }

    let Some(add_custom_scheme) = (unsafe { (*registrar).add_custom_scheme }) else {
        return false;
    };

    let scheme = CefString::new(scheme_name);
    let result = unsafe { add_custom_scheme(registrar, scheme.as_cef_string(), options) };
    result != 0
}
