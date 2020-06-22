use crate::handlers::audio::AudioHandler;
use crate::rust_to_c::Wrapper;

use crate::browser::Browser;
use cef_sys::{cef_audio_handler_t, cef_browser_t};
use std::sync::Arc;

unsafe extern "stdcall" fn on_audio_stream_packet<I: AudioHandler>(
    this: *mut cef_audio_handler_t, browser: *mut cef_browser_t, audio_stream_id: i32,
    data: *mut *const f32, frames: i32, pts: i64,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);

    obj.interface
        .on_audio_stream_packet(browser, audio_stream_id, data, frames, pts);
}

unsafe extern "stdcall" fn on_audio_stream_started<I: AudioHandler>(
    this: *mut cef_audio_handler_t, browser: *mut cef_browser_t, audio_stream_id: i32,
    channels: i32, channel_layout: i32, sample_rate: i32, frames_per_buffer: i32,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);

    obj.interface.on_audio_stream_started(
        browser,
        audio_stream_id,
        channels,
        channel_layout,
        sample_rate,
        frames_per_buffer,
    );
}

unsafe extern "stdcall" fn on_audio_stream_stopped<I: AudioHandler>(
    this: *mut cef_audio_handler_t, browser: *mut cef_browser_t, audio_stream_id: i32,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);

    obj.interface
        .on_audio_stream_stopped(browser, audio_stream_id);
}

pub fn wrap<T: AudioHandler>(audio: Arc<T>) -> *mut cef_audio_handler_t {
    let mut object: cef_audio_handler_t = unsafe { std::mem::zeroed() };

    object.on_audio_stream_packet = Some(on_audio_stream_packet::<T>);
    object.on_audio_stream_started = Some(on_audio_stream_started::<T>);
    object.on_audio_stream_stopped = Some(on_audio_stream_stopped::<T>);

    let wrapper = Wrapper::new(object, audio);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
