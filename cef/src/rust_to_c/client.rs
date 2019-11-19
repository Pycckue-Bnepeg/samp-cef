use super::Wrapper;
use crate::client::Client;

use std::ptr::null_mut;
use std::sync::Arc;

use crate::browser::{Browser, Frame};
use crate::process_message::ProcessMessage;
use crate::ProcessId;
use cef_sys::{
    cef_audio_handler_t, cef_browser_t, cef_client_t, cef_context_menu_handler_t,
    cef_dialog_handler_t, cef_display_handler_t, cef_download_handler_t, cef_drag_handler_t,
    cef_find_handler_t, cef_focus_handler_t, cef_frame_t, cef_jsdialog_handler_t,
    cef_keyboard_handler_t, cef_life_span_handler_t, cef_load_handler_t, cef_process_message_t,
    cef_render_handler_t, cef_request_handler_t,
};

// audio

extern "stdcall" fn audio<I: Client>(this: *mut cef_client_t) -> *mut cef_audio_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.audio_handler() {
        super::audio_handler::wrap(handler)
    } else {
        null_mut()
    }
}

// context menu

#[inline(never)]
extern "stdcall" fn context_menu<I: Client>(
    this: *mut cef_client_t,
) -> *mut cef_context_menu_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.context_menu_handler() {
        super::context_menu_handler::wrap(handler)
    } else {
        null_mut()
    }
}

// dialog

extern "stdcall" fn dialog<I: Client>(this: *mut cef_client_t) -> *mut cef_dialog_handler_t {
    println!("dialog");
    null_mut()
}

// display

extern "stdcall" fn display<I: Client>(this: *mut cef_client_t) -> *mut cef_display_handler_t {
    println!("display");
    null_mut()
}

// download

extern "stdcall" fn download<I: Client>(this: *mut cef_client_t) -> *mut cef_download_handler_t {
    println!("download");
    null_mut()
}

// drag

extern "stdcall" fn drag<I: Client>(this: *mut cef_client_t) -> *mut cef_drag_handler_t {
    println!("drag");
    null_mut()
}

// find

extern "stdcall" fn find<I: Client>(this: *mut cef_client_t) -> *mut cef_find_handler_t {
    println!("find");
    null_mut()
}

// focus

extern "stdcall" fn focus<I: Client>(this: *mut cef_client_t) -> *mut cef_focus_handler_t {
    println!("focus");
    null_mut()
}

// jsdialog

extern "stdcall" fn jsdialog<I: Client>(this: *mut cef_client_t) -> *mut cef_jsdialog_handler_t {
    println!("jsdialog");
    null_mut()
}

// keyboard

extern "stdcall" fn keyboard<I: Client>(this: *mut cef_client_t) -> *mut cef_keyboard_handler_t {
    println!("keyboard");
    null_mut()
}

// load

extern "stdcall" fn load<I: Client>(this: *mut cef_client_t) -> *mut cef_load_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.load_handler() {
        super::load_handler::wrap(handler)
    } else {
        null_mut()
    }
}

// render

extern "stdcall" fn render<I: Client>(this: *mut cef_client_t) -> *mut cef_render_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.render_handler() {
        super::render_handler::wrap(handler)
    } else {
        null_mut()
    }
}

// request

extern "stdcall" fn request<I: Client>(this: *mut cef_client_t) -> *mut cef_request_handler_t {
    println!("request");
    null_mut()
}

// message received

extern "stdcall" fn on_process_message<I: Client>(
    this: *mut cef_client_t, browser: *mut cef_browser_t, frame: *mut cef_frame_t,
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

// lifespan handler

extern "stdcall" fn lifespan<I: Client>(this: *mut cef_client_t) -> *mut cef_life_span_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.lifespan_handler() {
        super::lifespan_handler::wrap::<I::LifespanHandler>(handler)
    } else {
        null_mut()
    }
}

pub fn wrap<T: Client>(client: Arc<T>) -> *mut cef_client_t {
    let mut object: cef_client_t = unsafe { std::mem::zeroed() };

    object.get_life_span_handler = Some(lifespan::<T>);
    object.on_process_message_received = Some(on_process_message::<T>);
    object.get_request_handler = Some(request::<T>);
    object.get_render_handler = Some(render::<T>);
    object.get_load_handler = Some(load::<T>);
    object.get_keyboard_handler = Some(keyboard::<T>);
    object.get_jsdialog_handler = Some(jsdialog::<T>);
    object.get_focus_handler = Some(focus::<T>);
    object.get_find_handler = Some(find::<T>);
    object.get_drag_handler = Some(drag::<T>);
    object.get_download_handler = Some(download::<T>);
    object.get_display_handler = Some(display::<T>);
    object.get_dialog_handler = Some(dialog::<T>);
    object.get_context_menu_handler = Some(context_menu::<T>);
    object.get_audio_handler = Some(audio::<T>);

    let wrapper = Wrapper::new(object, client);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
