use super::Wrapper;
use crate::app::App;
use crate::command_line::CommandLine;
use crate::types::string::CefString;

use cef_sys::{
    cef_app_t, cef_browser_process_handler_t, cef_command_line_t, cef_render_process_handler_t,
    cef_string_t,
};

extern "system" fn get_render_process_handler<I: App>(
    this: *mut cef_app_t,
) -> *mut cef_render_process_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.render_process_handler() {
        super::render_process_handler::wrap(handler)
    } else {
        std::ptr::null_mut()
    }
}

extern "system" fn get_browser_process_handler<I: App>(
    this: *mut cef_app_t,
) -> *mut cef_browser_process_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.browser_process_handler() {
        super::browser_process_handler::wrap(handler)
    } else {
        std::ptr::null_mut()
    }
}

extern "system" fn on_before_command_line_processing<I: App>(
    this: *mut cef_app_t, process_type: *const cef_string_t, command_line: *mut cef_command_line_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let process_type = CefString::from(process_type);
    let cmd = CommandLine::from_raw_borrowed(command_line);

    obj.interface
        .on_before_command_line_processing(process_type, cmd);
}

extern "system" fn on_register_custom_schemes<I: App>(
    this: *mut cef_app_t, registrar: *mut cef_sys::cef_scheme_registrar_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    obj.interface.on_register_custom_schemes(registrar);
}

pub fn wrap<T: App>(app: T) -> *mut cef_app_t {
    let mut object: cef_app_t = unsafe { std::mem::zeroed() };

    object.get_render_process_handler = Some(get_render_process_handler::<T>);
    object.get_browser_process_handler = Some(get_browser_process_handler::<T>);
    object.on_before_command_line_processing = Some(on_before_command_line_processing::<T>);
    object.on_register_custom_schemes = Some(on_register_custom_schemes::<T>);

    let wrapper = Wrapper::new(object, app);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
