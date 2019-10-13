use super::Wrapper;
use crate::app::App;

use cef_sys::{cef_app_t, cef_render_process_handler_t};
use std::sync::Arc;

extern "stdcall" fn get_render_process_handler<I: App>(
    this: *mut cef_app_t,
) -> *mut cef_render_process_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.render_process_handler() {
        super::render_process_handler::wrap(handler)
    } else {
        std::ptr::null_mut()
    }
}

pub fn wrap<T: App>(app: Arc<T>) -> *mut cef_app_t {
    let mut object: cef_app_t = unsafe { std::mem::zeroed() };

    object.get_render_process_handler = Some(get_render_process_handler::<T>);

    let wrapper = Wrapper::new(object, app);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
