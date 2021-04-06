use libloading::{Library, Symbol};

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

pub use cef::types::{
    list::{List, ValueType},
    string::CefString,
};

pub use cef_sys::cef_list_value_t;
pub use cef_sys::cef_string_userfree_t;

pub const CEF_EVENT_CONTINUE: c_int = 0;
pub const CEF_EVENT_BREAK: c_int = 1;

pub type BrowserReadyCallback = extern "C" fn(u32);
pub type EventCallback = extern "C" fn(*const c_char, *mut cef_list_value_t) -> c_int;

static mut API: *mut InternalApi = std::ptr::null_mut();

#[repr(C)]
#[derive(Clone)]
pub struct InternalApi {
    cef_create_browser:
        unsafe extern "C" fn(id: u32, url: *const c_char, hidden: bool, focused: bool),
    cef_destroy_browser: unsafe extern "C" fn(id: u32),
    cef_hide_browser: unsafe extern "C" fn(id: u32, hide: bool),
    cef_focus_browser: unsafe extern "C" fn(id: u32, focus: bool),
    cef_create_list: unsafe extern "C" fn() -> *mut cef_list_value_t,
    cef_emit_event: unsafe extern "C" fn(event: *const c_char, list: *mut cef_list_value_t),
    cef_subscribe: unsafe extern "C" fn(event: *const c_char, callback: Option<EventCallback>),
    cef_input_available: unsafe extern "C" fn(browser: u32) -> bool,
    cef_ready: unsafe extern "C" fn() -> bool,
    cef_try_focus_browser: unsafe extern "C" fn(browser: u32) -> bool,
    cef_browser_exists: unsafe extern "C" fn(browser: u32) -> bool,
    cef_browser_ready: unsafe extern "C" fn(browser: u32) -> bool,
    cef_on_browser_ready: unsafe extern "C" fn(browser: u32, callback: BrowserReadyCallback),
    cef_gta_window_active: unsafe extern "C" fn() -> bool,
}

pub struct CefApi;

impl CefApi {
    pub fn initialize(api: *mut InternalApi) {
        unsafe {
            let boxed = Box::new((*api).clone());
            API = Box::into_raw(boxed);
        }
    }

    pub fn uninitialize() {
        unsafe {
            let _ = Box::from_raw(API);
            API = std::ptr::null_mut();
        }
    }

    pub fn create_browser(id: u32, url: &str, hidden: bool, focused: bool) {
        let url_cstr = CString::new(url).unwrap();
        unsafe {
            ((*API).cef_create_browser)(id, url_cstr.as_ptr(), hidden, focused);
        }
    }

    pub fn destroy_browser(id: u32) {
        unsafe {
            ((*API).cef_destroy_browser)(id);
        }
    }

    pub fn hide_browser(id: u32, hide: bool) {
        unsafe {
            ((*API).cef_hide_browser)(id, hide);
        }
    }

    pub fn focus_browser(id: u32, focus: bool) {
        unsafe {
            ((*API).cef_focus_browser)(id, focus);
        }
    }

    pub fn create_list() -> List {
        let list = unsafe { ((*API).cef_create_list)() };

        List::try_from_raw(list).unwrap()
    }

    pub fn emit_event(event: &str, args: &List) {
        let list = args.clone().into_cef();
        let event = CString::new(event).unwrap();
        unsafe {
            ((*API).cef_emit_event)(event.as_ptr(), list);
        }
    }

    pub fn subscribe(event: &str, callback: EventCallback) {
        let event = CString::new(event).unwrap();
        unsafe {
            ((*API).cef_subscribe)(event.as_ptr(), Some(callback));
        }
    }

    pub fn is_input_available(browser: u32) -> bool {
        unsafe { ((*API).cef_input_available)(browser) }
    }

    pub fn try_focus_browser(browser: u32) -> bool {
        unsafe { ((*API).cef_try_focus_browser)(browser) }
    }

    pub fn browser_exists(browser: u32) -> bool {
        unsafe { ((*API).cef_browser_exists)(browser) }
    }

    pub fn browser_ready(browser: u32) -> bool {
        unsafe { ((*API).cef_browser_ready)(browser) }
    }

    pub fn on_browser_ready(browser: u32, callback: BrowserReadyCallback) {
        unsafe {
            ((*API).cef_on_browser_ready)(browser, callback);
        }
    }

    pub fn is_window_active() -> bool {
        unsafe { ((*API).cef_gta_window_active)() }
    }
}
