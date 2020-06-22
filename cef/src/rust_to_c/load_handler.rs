use crate::browser::{Browser, Frame};
use crate::handlers::load::LoadHandler;
use crate::rust_to_c::Wrapper;

use cef_sys::{cef_browser_t, cef_frame_t, cef_load_handler_t};

use std::os::raw::c_int;
use std::sync::Arc;

unsafe extern "stdcall" fn on_loading_state_change<I: LoadHandler>(
    this: *mut cef_load_handler_t, browser: *mut cef_browser_t, is_loading: c_int,
    can_go_back: c_int, can_go_forward: c_int,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    let browser = Browser::from_raw(browser);
    let is_loading = is_loading == 1;
    let can_go_back = can_go_back == 1;
    let can_go_forward = can_go_forward == 1;

    obj.interface
        .on_loading_state_change(browser, is_loading, can_go_back, can_go_forward);
}

unsafe extern "stdcall" fn on_load_end<I: LoadHandler>(
    this: *mut cef_load_handler_t, browser: *mut cef_browser_t, frame: *mut cef_frame_t,
    status_code: c_int,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    let browser = Browser::from_raw(browser);
    let frame = Frame::from_raw(frame);

    obj.interface.on_load_end(browser, frame, status_code);
}

pub fn wrap<T: LoadHandler>(load: Arc<T>) -> *mut cef_load_handler_t {
    let mut object: cef_load_handler_t = unsafe { std::mem::zeroed() };

    object.on_loading_state_change = Some(on_loading_state_change::<T>);
    object.on_load_end = Some(on_load_end::<T>);

    let wrapper = Wrapper::new(object, load);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
