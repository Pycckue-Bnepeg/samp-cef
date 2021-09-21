use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use parking_lot::Mutex;

use cef::types::list::List;
use cef_sys::cef_list_value_t;

use crossbeam_channel::Sender;

use crate::app::Event;
use crate::browser::manager::Manager;

use libloading::{Library, Symbol};

static mut PLUGINS: Option<ExternalManager> = None;

pub type BrowserReadyCallback = extern "C" fn(u32);
pub type EventCallback = extern "C" fn(*const c_char, *mut cef_list_value_t) -> c_int;
pub type CallbackList = Arc<Mutex<HashMap<String, EventCallback>>>;

pub const EXTERNAL_CONTINUE: c_int = 0;
pub const EXTERNAL_BREAK: c_int = 1;

pub struct ExternalManager {
    manager: Arc<Mutex<Manager>>,
    event_tx: Sender<Event>,
    callbacks: CallbackList,
    plugins: Vec<ExtPlugin>,
    window_active: AtomicBool,
}

struct ExtPlugin {
    library: Library,
    initialize: Option<Symbol<'static, extern "C" fn(*mut InternalApi)>>,
    mainloop: Option<Symbol<'static, extern "C" fn()>>,
    dxreset: Option<Symbol<'static, extern "C" fn()>>,
    connect: Option<Symbol<'static, extern "C" fn()>>,
    disconnect: Option<Symbol<'static, extern "C" fn()>>,
    quit: Option<Symbol<'static, extern "C" fn()>>,
    browser_created: Option<Symbol<'static, extern "C" fn(u32, i32)>>,
}

// TODO: cef_emit_server_event, cef_emit_client_event
#[repr(C)]
struct InternalApi {
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

fn make_api_struct() -> InternalApi {
    InternalApi {
        cef_create_browser,
        cef_destroy_browser,
        cef_hide_browser,
        cef_focus_browser,
        cef_create_list,
        cef_emit_event,
        cef_subscribe,
        cef_input_available,
        cef_ready,
        cef_try_focus_browser,
        cef_browser_exists,
        cef_browser_ready,
        cef_on_browser_ready,
        cef_gta_window_active,
    }
}

impl ExternalManager {
    pub fn get<'a>() -> Option<&'a mut ExternalManager> {
        unsafe { PLUGINS.as_mut() }
    }
}

pub fn initialize(event_tx: Sender<Event>, manager: Arc<Mutex<Manager>>) -> CallbackList {
    let callbacks = Arc::new(Mutex::new(HashMap::new()));

    let external = ExternalManager {
        manager,
        event_tx,
        callbacks: callbacks.clone(),
        plugins: Vec::new(),
        window_active: AtomicBool::new(true),
    };

    let external = unsafe {
        PLUGINS = Some(external);
        PLUGINS.as_mut().unwrap()
    };

    let plugins_path = crate::utils::cef_dir().join("plugins");

    if let Ok(rd) = std::fs::read_dir(plugins_path) {
        for dir in rd.filter_map(|dir| dir.ok()) {
            if let Some(ext) = dir.path().extension() {
                if ext.to_string_lossy() == "dll" {
                    unsafe {
                        match Library::new(dir.path().as_os_str()) {
                            Ok(mut lib) => {
                                let library: &'static mut Library =
                                    &mut *(&mut lib as *mut Library);

                                let mainloop =
                                    library.get::<extern "C" fn()>(b"cef_samp_mainloop").ok();

                                let dxreset = library.get::<extern "C" fn()>(b"cef_dxreset").ok();
                                let initialize = library
                                    .get::<extern "C" fn(*mut InternalApi)>(b"cef_initialize")
                                    .ok();

                                let connect = library.get::<extern "C" fn()>(b"cef_connect").ok();
                                let disconnect =
                                    library.get::<extern "C" fn()>(b"cef_disconnect").ok();
                                let quit = library.get::<extern "C" fn()>(b"cef_quit").ok();
                                let browser_created = library
                                    .get::<extern "C" fn(u32, i32)>(b"cef_browser_created")
                                    .ok();

                                let plugin = ExtPlugin {
                                    library: lib,
                                    initialize,
                                    mainloop,
                                    dxreset,
                                    connect,
                                    disconnect,
                                    quit,
                                    browser_created,
                                };

                                external.plugins.push(plugin);

                                log::trace!("loaded plugin: {:?}", dir.path());
                            }

                            Err(e) => log::trace!("error loading library {:?}", e),
                        }
                    }
                }
            }
        }
    }

    callbacks
}

pub fn call_mainloop() {
    if let Some(ext) = ExternalManager::get() {
        for plugin in &ext.plugins {
            if let Some(func) = plugin.mainloop.as_ref() {
                func();
            }
        }
    }
}

pub fn call_dxreset() {
    if let Some(ext) = ExternalManager::get() {
        for plugin in &ext.plugins {
            if let Some(func) = plugin.dxreset.as_ref() {
                func();
            }
        }
    }
}

pub fn call_initialize() {
    if let Some(ext) = ExternalManager::get() {
        let mut api = make_api_struct();

        for plugin in &ext.plugins {
            if let Some(func) = plugin.initialize.as_ref() {
                func(&mut api);
            }
        }
    }
}

pub fn call_connect() {
    if let Some(ext) = ExternalManager::get() {
        for plugin in &ext.plugins {
            if let Some(func) = plugin.connect.as_ref() {
                func();
            }
        }
    }
}

pub fn call_disconnect() {
    if let Some(ext) = ExternalManager::get() {
        for plugin in &ext.plugins {
            if let Some(func) = plugin.disconnect.as_ref() {
                func();
            }
        }
    }
}

pub fn window_active(active: bool) {
    if let Some(ext) = ExternalManager::get() {
        ext.window_active.store(active, Ordering::SeqCst);
    }
}

pub fn quit() {
    if let Some(ext) = ExternalManager::get() {
        for mut plugin in ext.plugins.drain(..) {
            if let Some(quit) = plugin.quit.as_mut() {
                quit();
            }
        }
    }
}

pub fn browser_created(browser_id: u32, status_code: i32) {
    if let Some(ext) = ExternalManager::get() {
        for plugin in &ext.plugins {
            if let Some(func) = plugin.browser_created.as_ref() {
                func(browser_id, status_code);
            }
        }
    }
}

unsafe extern "C" fn cef_create_browser(id: u32, url: *const c_char, hidden: bool, focused: bool) {
    if url.is_null() {
        return;
    }

    if let Some(external) = ExternalManager::get() {
        let url = CStr::from_ptr(url);
        let url_rust = url.to_string_lossy();

        let event = Event::CreateBrowser {
            id,
            hidden,
            focused,
            url: url_rust.to_string(),
        };

        external.event_tx.send(event);
    }
}

unsafe extern "C" fn cef_destroy_browser(id: u32) {
    if let Some(external) = ExternalManager::get() {
        let mut manager = external.manager.lock();
        manager.close_browser(id, true);
    }
}

unsafe extern "C" fn cef_hide_browser(id: u32, hide: bool) {
    if let Some(external) = ExternalManager::get() {
        let event = Event::HideBrowser(id, hide);
        external.event_tx.send(event);
    }
}

unsafe extern "C" fn cef_focus_browser(id: u32, focus: bool) {
    if let Some(external) = ExternalManager::get() {
        let event = Event::FocusBrowser(id, focus);
        external.event_tx.send(event);
    }
}

unsafe extern "C" fn cef_create_list() -> *mut cef_list_value_t {
    let list = List::new();
    list.into_cef()
}

unsafe extern "C" fn cef_emit_event(event: *const c_char, list: *mut cef_list_value_t) {
    if event.is_null() {
        return;
    }

    if let Some(list) = List::try_from_raw(list) {
        if let Some(external) = ExternalManager::get() {
            let manager = external.manager.lock();
            let name = CStr::from_ptr(event);
            manager.trigger_event(&name.to_string_lossy(), list);
        }
    }
}

unsafe extern "C" fn cef_subscribe(event: *const c_char, callback: Option<EventCallback>) {
    if event.is_null() || callback.is_none() {
        return;
    }

    let event = CStr::from_ptr(event);
    let event_rust = event.to_string_lossy();

    if let Some(external) = ExternalManager::get() {
        let mut cbs = external.callbacks.lock();
        cbs.insert(event_rust.to_string(), callback.unwrap());
    }
}

unsafe extern "C" fn cef_input_available(browser: u32) -> bool {
    ExternalManager::get()
        .map(|ext| {
            let manager = ext.manager.lock();
            manager.is_input_available(browser)
        })
        .unwrap_or(false)
}

// TODO: ?
unsafe extern "C" fn cef_ready() -> bool {
    false
}

unsafe extern "C" fn cef_try_focus_browser(browser: u32) -> bool {
    ExternalManager::get()
        .map(|ext| {
            let mut manager = ext.manager.lock();

            if manager.is_input_available(browser) {
                manager.browser_focus(browser, true);
                drop(manager);

                let event = Event::FocusBrowser(browser, true);
                ext.event_tx.send(event);

                true
            } else {
                false
            }
        })
        .unwrap_or(false)
}

unsafe extern "C" fn cef_browser_exists(browser: u32) -> bool {
    ExternalManager::get()
        .map(|ext| {
            let manager = ext.manager.lock();
            manager.browser_exists(browser)
        })
        .unwrap_or(false)
}

unsafe extern "C" fn cef_browser_ready(browser: u32) -> bool {
    ExternalManager::get()
        .map(|ext| {
            let manager = ext.manager.lock();
            manager.browser_ready(browser)
        })
        .unwrap_or(false)
}

unsafe extern "C" fn cef_on_browser_ready(browser: u32, callback: BrowserReadyCallback) {
    if let Some(ext) = ExternalManager::get() {
        let mut manager = ext.manager.lock();
        manager.add_browser_ready(browser, callback);
    }
}

unsafe extern "C" fn cef_gta_window_active() -> bool {
    ExternalManager::get()
        .map(|ext| ext.window_active.load(Ordering::SeqCst))
        .unwrap_or(false)
}
