use super::Wrapper;
use crate::handlers::render_process::RenderProcessHandler;
use std::sync::Arc;

use crate::browser::{Browser, Frame};
use crate::process_message::ProcessMessage;
use crate::v8::V8Context;
use crate::ProcessId;
use cef_sys::{
    cef_browser_t, cef_frame_t, cef_process_message_t, cef_render_process_handler_t,
    cef_v8context_t,
};

extern "stdcall" fn on_context_created<I: RenderProcessHandler>(
    this: *mut cef_render_process_handler_t, browser: *mut cef_browser_t, frame: *mut cef_frame_t,
    context: *mut cef_v8context_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    let frame = Frame::from_raw(frame);
    let context = V8Context::from_raw(context);

    obj.interface.on_context_created(browser, frame, context);
}

extern "stdcall" fn on_context_released<I: RenderProcessHandler>(
    this: *mut cef_render_process_handler_t, browser: *mut cef_browser_t, frame: *mut cef_frame_t,
    context: *mut cef_v8context_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    let frame = Frame::from_raw(frame);
    let context = V8Context::from_raw(context);

    obj.interface.on_context_released(browser, frame, context);
}

extern "stdcall" fn on_webkit_initialized<I: RenderProcessHandler>(
    this: *mut cef_render_process_handler_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    obj.interface.on_webkit_initialized();
}

extern "stdcall" fn on_process_message<I: RenderProcessHandler>(
    this: *mut cef_render_process_handler_t, browser: *mut cef_browser_t, frame: *mut cef_frame_t,
    source_process: cef_sys::cef_process_id_t::Type, message: *mut cef_process_message_t,
) -> i32 {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    let browser = Browser::from_raw(browser);
    let frame = Frame::from_raw(frame);
    let process_id: ProcessId = ProcessId::from(source_process);
    let message = ProcessMessage::from_raw(message);

    let result = obj
        .interface
        .on_process_message(browser, frame, process_id, message);

    if result {
        1
    } else {
        0
    }
}

pub fn wrap<T: RenderProcessHandler>(handler: Arc<T>) -> *mut cef_render_process_handler_t {
    let mut object: cef_render_process_handler_t = unsafe { std::mem::zeroed() };

    object.on_context_created = Some(on_context_created::<T>);
    object.on_context_released = Some(on_context_released::<T>);
    object.on_web_kit_initialized = Some(on_webkit_initialized::<T>);
    object.on_process_message_received = Some(on_process_message::<T>);

    let wrapper = Wrapper::new(object, handler);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
