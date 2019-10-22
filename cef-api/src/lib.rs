use libloading::{Library, Symbol};

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

pub use cef::types::{list::List, string::CefString};
pub use cef_sys::cef_list_value_t;
pub use cef_sys::cef_string_userfree_t;

pub const CEF_EVENT_CONTINUE: c_int = 0;
pub const CEF_EVENT_BREAK: c_int = 1;

pub type EventCallback = extern "C" fn(*const c_char, *mut cef_list_value_t) -> c_int;

pub struct CefApi {
    library: Library,
    funcs: Symbols,
}

pub struct Symbols {
    create_browser:
        Symbol<'static, extern "C" fn(id: u32, url: *const c_char, hidden: bool, focused: bool)>,
    destroy_browser: Symbol<'static, extern "C" fn(id: u32)>,
    hide_browser: Symbol<'static, extern "C" fn(id: u32, hide: bool)>,
    focus_browser: Symbol<'static, extern "C" fn(id: u32, focus: bool)>,
    create_list: Symbol<'static, extern "C" fn() -> *mut cef_list_value_t>,
    emit_event: Symbol<'static, extern "C" fn(event: *const c_char, args: *mut cef_list_value_t)>,

    subscribe: Symbol<'static, extern "C" fn(event: *const c_char, callback: EventCallback)>,
    input_available: Symbol<'static, extern "C" fn(id: u32) -> bool>,
    try_focus_browser: Symbol<'static, extern "C" fn(id: u32) -> bool>,
    //    is_ready: Symbol<'static, extern "C" fn() -> bool>,
}

impl CefApi {
    pub fn wait_loading() -> Option<CefApi> {
        while !std::env::current_dir()
            .map(|dir| {
                std::env::current_exe()
                    .map(|exe| exe.parent().unwrap() == dir)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
        {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let current = std::env::current_dir().unwrap();
        let temp_dir = current.join("./cef");

        std::env::set_current_dir(temp_dir).unwrap();

        let cef_api = Library::new("client.dll").ok().map(|mut lib| {
            let library: &'static mut Library = unsafe { &mut *(&mut lib as *mut Library) };

            let funcs = unsafe {
                Symbols {
                    create_browser: library.get(b"cef_create_browser").unwrap(),
                    destroy_browser: library.get(b"cef_destroy_browser").unwrap(),
                    hide_browser: library.get(b"cef_hide_browser").unwrap(),
                    focus_browser: library.get(b"cef_focus_browser").unwrap(),
                    create_list: library.get(b"cef_create_list").unwrap(),
                    emit_event: library.get(b"cef_emit_event").unwrap(),
                    subscribe: library.get(b"cef_subscribe").unwrap(),
                    input_available: library.get(b"cef_input_available").unwrap(),
                    try_focus_browser: library.get(b"cef_try_focus_browser").unwrap(),
                }
            };

            CefApi {
                library: lib,
                funcs,
            }
        });

        std::env::set_current_dir(current);

        cef_api
    }

    pub fn create_browser(&self, id: u32, url: &str, hidden: bool, focused: bool) {
        let url_cstr = CString::new(url).unwrap();
        (self.funcs.create_browser)(id, url_cstr.as_ptr(), hidden, focused);
    }

    pub fn destroy_browser(&self, id: u32) {
        (self.funcs.destroy_browser)(id);
    }

    pub fn hide_browser(&self, id: u32, hide: bool) {
        (self.funcs.hide_browser)(id, hide);
    }

    pub fn focus_browser(&self, id: u32, focus: bool) {
        (self.funcs.focus_browser)(id, focus);
    }

    pub fn create_list(&self) -> List {
        let list = (self.funcs.create_list)();

        List::try_from_raw(list).unwrap()
    }

    pub fn emit_event(&self, event: &str, args: &List) {
        let list = args.clone().into_cef();
        let event = CString::new(event).unwrap();
        (self.funcs.emit_event)(event.as_ptr(), list);
    }

    pub fn subscribe(&self, event: &str, callback: EventCallback) {
        let event = CString::new(event).unwrap();
        (self.funcs.subscribe)(event.as_ptr(), callback);
    }

    pub fn is_input_available(&self, browser: u32) -> bool {
        (self.funcs.input_available)(browser)
    }

    pub fn try_focus_browser(&self, browser: u32) -> bool {
        (self.funcs.try_focus_browser)(browser)
    }
}
