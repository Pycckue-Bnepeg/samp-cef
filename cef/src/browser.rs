use cef_sys::{
    cef_browser_host_t, cef_browser_settings_t, cef_browser_t, cef_frame_t, cef_key_event_t,
    cef_mouse_event_t, cef_window_info_t,
};

use crate::client::Client;
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
}
