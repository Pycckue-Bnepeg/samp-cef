use crate::browser::{Browser, ContextMenuParams, Frame, MenuModel};
use crate::handlers::context_menu::ContextMenuHandler;
use crate::rust_to_c::Wrapper;

use cef_sys::{
    cef_browser_t, cef_context_menu_handler_t, cef_context_menu_params_t, cef_frame_t,
    cef_menu_model_t,
};

use std::sync::Arc;

unsafe extern "stdcall" fn on_before_context_menu<I: ContextMenuHandler>(
    this: *mut cef_context_menu_handler_t, browser: *mut cef_browser_t, frame: *mut cef_frame_t,
    params: *mut cef_context_menu_params_t, model: *mut cef_menu_model_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    let browser = Browser::from_raw(browser);
    let frame = Frame::from_raw(frame);
    let params = ContextMenuParams::from_raw(params);
    let model = MenuModel::from_raw(model);

    obj.interface
        .on_before_context_menu(browser, frame, params, model);
}

pub fn wrap<T: ContextMenuHandler>(context_menu: Arc<T>) -> *mut cef_context_menu_handler_t {
    let mut object: cef_context_menu_handler_t = unsafe { std::mem::zeroed() };

    object.on_before_context_menu = Some(on_before_context_menu::<T>);

    let wrapper = Wrapper::new(object, context_menu);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
