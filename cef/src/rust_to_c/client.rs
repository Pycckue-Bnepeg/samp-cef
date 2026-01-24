use super::Wrapper;
use crate::client::Client;

use std::ptr::null_mut;

use crate::ProcessId;
use crate::browser::{Browser, Frame};
use crate::process_message::ProcessMessage;
use cef_sys::{
    cef_audio_handler_t, cef_browser_t, cef_client_t, cef_context_menu_handler_t,
    cef_dialog_handler_t, cef_display_handler_t, cef_download_handler_t, cef_drag_handler_t,
    cef_find_handler_t, cef_focus_handler_t, cef_frame_t, cef_jsdialog_handler_t,
    cef_keyboard_handler_t, cef_life_span_handler_t, cef_load_handler_t, cef_process_message_t,
    cef_render_handler_t, cef_request_handler_t,
};

// audio

extern "system" fn audio<I: Client>(this: *mut cef_client_t) -> *mut cef_audio_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.audio_handler() {
        super::audio_handler::wrap(handler)
    } else {
        null_mut()
    }
}

// context menu

#[inline(never)]
extern "system" fn context_menu<I: Client>(
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

extern "system" fn dialog(_this: *mut cef_client_t) -> *mut cef_dialog_handler_t {
    println!("dialog");
    null_mut()
}

// display

extern "system" fn display(_this: *mut cef_client_t) -> *mut cef_display_handler_t {
    println!("display");
    null_mut()
}

// download

extern "system" fn download(_this: *mut cef_client_t) -> *mut cef_download_handler_t {
    println!("download");
    null_mut()
}

// drag

extern "system" fn drag(_this: *mut cef_client_t) -> *mut cef_drag_handler_t {
    println!("drag");
    null_mut()
}

// find

extern "system" fn find(_this: *mut cef_client_t) -> *mut cef_find_handler_t {
    println!("find");
    null_mut()
}

// focus

extern "system" fn focus(_this: *mut cef_client_t) -> *mut cef_focus_handler_t {
    println!("focus");
    null_mut()
}

// jsdialog

extern "system" fn jsdialog(_this: *mut cef_client_t) -> *mut cef_jsdialog_handler_t {
    println!("jsdialog");
    null_mut()
}

// keyboard

extern "system" fn keyboard(_this: *mut cef_client_t) -> *mut cef_keyboard_handler_t {
    println!("keyboard");
    null_mut()
}

// load

extern "system" fn load<I: Client>(this: *mut cef_client_t) -> *mut cef_load_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.load_handler() {
        super::load_handler::wrap(handler)
    } else {
        null_mut()
    }
}

// render

extern "system" fn render<I: Client>(this: *mut cef_client_t) -> *mut cef_render_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.render_handler() {
        super::render_handler::wrap(handler)
    } else {
        null_mut()
    }
}

// request

extern "system" fn request(_this: *mut cef_client_t) -> *mut cef_request_handler_t {
    println!("request");
    null_mut()
}

// message received

extern "system" fn on_process_message<I: Client>(
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

    if result { 1 } else { 0 }
}

// lifespan handler

extern "system" fn lifespan<I: Client>(this: *mut cef_client_t) -> *mut cef_life_span_handler_t {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if let Some(handler) = obj.interface.lifespan_handler() {
        super::lifespan_handler::wrap::<I::LifespanHandler>(handler)
    } else {
        null_mut()
    }
}

pub fn wrap<T: Client>(client: T) -> *mut cef_client_t {
    let mut object: cef_client_t = unsafe { std::mem::zeroed() };

    object.get_life_span_handler = Some(lifespan::<T>);
    object.on_process_message_received = Some(on_process_message::<T>);
    object.get_request_handler = Some(request);
    object.get_render_handler = Some(render::<T>);
    object.get_load_handler = Some(load::<T>);
    object.get_keyboard_handler = Some(keyboard);
    object.get_jsdialog_handler = Some(jsdialog);
    object.get_focus_handler = Some(focus);
    object.get_find_handler = Some(find);
    object.get_drag_handler = Some(drag);
    object.get_download_handler = Some(download);
    object.get_display_handler = Some(display);
    object.get_dialog_handler = Some(dialog);
    object.get_context_menu_handler = Some(context_menu::<T>);
    object.get_audio_handler = Some(audio::<T>);

    let wrapper = Wrapper::new(object, client);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
