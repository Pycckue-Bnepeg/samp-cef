use crate::handlers::audio::AudioHandler;
use crate::rust_to_c::Wrapper;

use crate::browser::Browser;
use cef_sys::{cef_audio_handler_t, cef_audio_parameters_t, cef_browser_t, cef_string_t};
use std::sync::Arc;

unsafe extern "stdcall" fn get_audio_parameters<I: AudioHandler>(
    this: *mut cef_audio_handler_t, browser: *mut cef_browser_t,
    params: *mut cef_audio_parameters_t,
) -> i32 {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);

    if obj.interface.get_audio_parameters(browser, &mut *params) {
        1
    } else {
        0
    }
}

unsafe extern "stdcall" fn on_audio_stream_packet<I: AudioHandler>(
    this: *mut cef_audio_handler_t, browser: *mut cef_browser_t, data: *mut *const f32,
    frames: i32, pts: i64,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    let audio_stream_id = 0; // temp comp

    obj.interface
        .on_audio_stream_packet(browser, audio_stream_id, data, frames, pts);
}

// unsafe extern "stdcall" fn on_audio_stream_started<I: AudioHandler>(
//     this: *mut cef_audio_handler_t, browser: *mut cef_browser_t, audio_stream_id: i32,
//     channels: i32, channel_layout: i32, sample_rate: i32, frames_per_buffer: i32,
// ) {
unsafe extern "stdcall" fn on_audio_stream_started<I: AudioHandler>(
    this: *mut cef_audio_handler_t, browser: *mut cef_browser_t,
    params: *const cef_audio_parameters_t, channels: i32,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);

    let params = &*params;
    let audio_stream_id = 0; // COMP

    obj.interface.on_audio_stream_started(
        browser,
        audio_stream_id,
        channels,
        params.channel_layout,
        params.sample_rate,
        params.frames_per_buffer,
    );
}

unsafe extern "stdcall" fn on_audio_stream_stopped<I: AudioHandler>(
    this: *mut cef_audio_handler_t, browser: *mut cef_browser_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    let audio_stream_id = 0; // COMP

    obj.interface
        .on_audio_stream_stopped(browser, audio_stream_id);
}

unsafe extern "stdcall" fn on_audio_stream_error<I: AudioHandler>(
    this: *mut cef_audio_handler_t, browser: *mut cef_browser_t, error: *const cef_string_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);

    let error = widestring::U16CString::from_ptr((*error).str_, (*error).length)
        .map(|wide| wide.to_string_lossy())
        .unwrap_or_else(|_| String::from("some really baaad error"));

    obj.interface.on_audio_stream_error(browser, error);
}

pub fn wrap<T: AudioHandler>(audio: Arc<T>) -> *mut cef_audio_handler_t {
    let mut object: cef_audio_handler_t = unsafe { std::mem::zeroed() };

    object.get_audio_parameters = Some(get_audio_parameters::<T>);
    object.on_audio_stream_packet = Some(on_audio_stream_packet::<T>);
    object.on_audio_stream_started = Some(on_audio_stream_started::<T>);
    object.on_audio_stream_stopped = Some(on_audio_stream_stopped::<T>);
    object.on_audio_stream_error = Some(on_audio_stream_error::<T>);

    let wrapper = Wrapper::new(object, audio);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
