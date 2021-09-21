use cef_sys::{
    cef_browser_host_t, cef_browser_settings_t, cef_browser_t, cef_context_menu_params_t,
    cef_frame_t, cef_key_event_t, cef_menu_model_t, cef_mouse_event_t, cef_point_t,
    cef_window_info_t,
};

use crate::client::Client;
use crate::handlers::render::PaintElement;
use crate::process_message::ProcessMessage;
use crate::ref_counted::RefGuard;
use crate::types::string::CefString;
use crate::v8::V8Context;
use crate::ProcessId;
use std::sync::Arc;

pub struct Browser {
    inner: RefGuard<cef_browser_t>,
}

impl Browser {
    #[inline]
    pub(crate) fn from_raw(raw: *mut cef_browser_t) -> Browser {
        if raw.is_null() {
            panic!("Browser::from_raw null pointer.");
        }

        Browser {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn host(&self) -> BrowserHost {
        if let Some(get_host) = self.inner.get_host {
            let host = unsafe { get_host(self.inner.get_mut()) };

            return BrowserHost::from_raw(host);
        }

        panic!("Browser::host() no get_host function.");
    }

    pub fn main_frame(&self) -> Frame {
        let get_main_frame = self
            .inner
            .get_main_frame
            .expect("Browser::main_frame() No main frame");

        let frame = unsafe { get_main_frame(self.inner.get_mut()) };

        Frame::from_raw(frame)
    }

    pub fn is_loading(&self) -> bool {
        let is_loading = self.inner.is_loading.unwrap();

        unsafe { is_loading(self.inner.get_mut()) == 1 }
    }
}

impl Clone for Browser {
    fn clone(&self) -> Browser {
        Browser {
            inner: self.inner.clone(),
        }
    }
}

pub struct BrowserHost {
    inner: RefGuard<cef_browser_host_t>,
}

impl BrowserHost {
    pub(crate) fn from_raw(raw: *mut cef_browser_host_t) -> BrowserHost {
        if raw.is_null() {
            panic!("BrowserHost::from_raw null pointer.");
        }

        BrowserHost {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn create_browser<T: Client>(
        wnd_info: &cef_window_info_t, client: Option<Arc<T>>, url: &CefString,
        settings: &cef_browser_settings_t,
    ) -> i32 {
        let client_ptr = client
            .map(|client| crate::rust_to_c::client::wrap(client))
            .unwrap_or(std::ptr::null_mut());

        unsafe {
            cef_sys::cef_browser_host_create_browser(
                wnd_info,
                client_ptr,
                url.as_cef_string(),
                settings,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }
    }

    pub fn close_browser(&self, force_close: bool) {
        let close = self.inner.close_browser.unwrap();
        unsafe {
            close(self.inner.get_mut(), if force_close { 1 } else { 0 });
        }
    }

    pub fn was_hidden(&self, hidden: bool) {
        let hide = self.inner.was_hidden.unwrap();
        unsafe {
            hide(self.inner.get_mut(), if hidden { 1 } else { 0 });
        }
    }

    pub fn was_resized(&self) {
        let was_resized = self.inner.was_resized.unwrap();
        unsafe {
            was_resized(self.inner.get_mut());
        }
    }

    pub fn open_dev_tools(&self, window: &cef_window_info_t, settings: &cef_browser_settings_t) {
        let dev_tools = self.inner.show_dev_tools.unwrap();

        unsafe {
            dev_tools(
                self.inner.get_mut(),
                window,
                std::ptr::null_mut(),
                settings,
                &cef_point_t { x: 0, y: 0 },
            );
        }
    }

    pub fn close_dev_tools(&self) {
        let dev_tools = self.inner.close_dev_tools.unwrap();

        unsafe {
            dev_tools(self.inner.get_mut());
        }
    }

    pub fn send_mouse_move(&self, event: cef_mouse_event_t) {
        if let Some(smme) = self.inner.send_mouse_move_event {
            unsafe {
                smme(self.inner.get_mut(), &event, 0);
            }
        }
    }

    pub fn send_mouse_click(
        &self, key: cef_sys::cef_mouse_button_type_t::Type, event: cef_mouse_event_t, is_down: bool,
    ) {
        if let Some(smce) = self.inner.send_mouse_click_event {
            unsafe {
                let up = if is_down { 0 } else { 1 };

                smce(self.inner.get_mut(), &event, key, up, 1);
            }
        }
    }

    pub fn send_mouse_wheel(&self, x: i32, y: i32, delta: i32) {
        if let Some(smwe) = self.inner.send_mouse_wheel_event {
            unsafe {
                let event = cef_mouse_event_t { x, y, modifiers: 0 };

                smwe(self.inner.get_mut(), &event, 0, delta * 40);
            }
        }
    }

    pub fn send_keyboard_event(&self, event: cef_key_event_t) {
        if let Some(ske) = self.inner.send_key_event {
            unsafe {
                ske(self.inner.get_mut(), &event);
            }
        }
    }

    pub fn invalidate(&self, paint_type: PaintElement) {
        let inv = self.inner.invalidate.unwrap();
        let ty = paint_type.into();

        unsafe {
            inv(self.inner.get_mut(), ty);
        }
    }

    pub fn set_audio_muted(&self, mute: bool) {
        let set = self.inner.set_audio_muted.unwrap();

        unsafe {
            set(self.inner.get_mut(), if mute { 1 } else { 0 });
        }
    }

    pub fn set_windowless_frame_rate(&self, framerate: i32) {
        let set = self.inner.set_windowless_frame_rate.unwrap();

        unsafe {
            set(self.inner.get_mut(), framerate);
        }
    }

    pub fn windowless_frame_rate(&self) -> i32 {
        let get = self.inner.get_windowless_frame_rate.unwrap();

        unsafe { get(self.inner.get_mut()) }
    }
}

#[derive(Clone)]
pub struct Frame {
    inner: RefGuard<cef_frame_t>,
}

impl Frame {
    pub(crate) fn from_raw(raw: *mut cef_frame_t) -> Frame {
        Frame {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn browser(&self) -> Browser {
        let get_br = self.inner.get_browser.unwrap();
        let browser = unsafe { get_br(self.inner.get_mut()) };

        Browser::from_raw(browser)
    }

    pub fn context(&self) -> V8Context {
        let get_ctx = self.inner.get_v8context.unwrap();
        let ctx = unsafe { get_ctx(self.inner.get_mut()) };

        V8Context::from_raw(ctx)
    }

    pub fn load_url(&self, url: &str) {
        let url = CefString::new(url);
        let load = self.inner.load_url.unwrap();

        unsafe {
            load(self.inner.get_mut(), url.as_cef_string());
        }
    }

    pub fn send_process_message(&self, target_process: ProcessId, message: ProcessMessage) {
        let send = self
            .inner
            .send_process_message
            .expect("Frame::send_process_message doesn't exist");

        let pid = target_process.into();

        unsafe {
            send(self.inner.get_mut(), pid, message.into_cef());
        }
    }

    pub fn is_main(&self) -> bool {
        let is_main = self.inner.is_main.unwrap();

        unsafe { is_main(self.inner.get_mut()) == 1 }
    }
}

#[derive(Clone)]
pub struct ContextMenuParams {
    inner: RefGuard<cef_context_menu_params_t>,
}

impl ContextMenuParams {
    pub(crate) fn from_raw(raw: *mut cef_context_menu_params_t) -> ContextMenuParams {
        ContextMenuParams {
            inner: RefGuard::from_raw(raw),
        }
    }
}

#[derive(Clone)]
pub struct MenuModel {
    inner: RefGuard<cef_menu_model_t>,
}

impl MenuModel {
    pub(crate) fn from_raw(raw: *mut cef_menu_model_t) -> MenuModel {
        MenuModel {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn clear(&self) {
        let clear = self.inner.clear.unwrap();
        unsafe {
            clear(self.inner.get_mut());
        }
    }
}
