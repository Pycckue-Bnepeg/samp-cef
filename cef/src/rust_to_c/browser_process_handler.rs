use crate::handlers::browser_process::BrowserProcessHandler;
use crate::rust_to_c::Wrapper;

use std::sync::Arc;

use cef_sys::cef_browser_process_handler_t;

extern "stdcall" fn on_context_initialized<I: BrowserProcessHandler>(
    this: *mut cef_browser_process_handler_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    obj.interface.on_context_initialized();
}

pub fn wrap<T: BrowserProcessHandler>(handler: Arc<T>) -> *mut cef_browser_process_handler_t {
    let mut object: cef_browser_process_handler_t = unsafe { std::mem::zeroed() };

    object.on_context_initialized = Some(on_context_initialized::<T>);

    let wrapper = Wrapper::new(object, handler);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
