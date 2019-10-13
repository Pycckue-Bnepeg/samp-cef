use std::sync::{Arc, Mutex};
use std::time::Duration;

use cef_sys::{cef_key_event_t, cef_key_event_type_t};

use winapi::shared::minwindef::{LPARAM, UINT, WPARAM};
use winapi::um::winuser::*;

use crate::browser::manager::{Manager, MouseKey};
use crate::network::Network;

use client_api::wndproc;

static mut APP: Option<App> = None;

pub struct App {
    init: bool,
    manager: Arc<Mutex<Manager>>,
    network: Network,
    //    event_rx
}

impl App {
    pub fn new() -> App {
        let manager = Arc::new(Mutex::new(Manager::new()));
        let network = Network::new();

        App {
            init: false,
            manager,
            network,
        }
    }

    pub fn manager(&self) -> Arc<Mutex<Manager>> {
        self.manager.clone()
    }

    fn get<'a>() -> Option<&'a mut App> {
        unsafe { APP.as_mut() }
    }
}

pub fn initialize() {
    let app = App::new();
    let manager = app.manager();

    unsafe {
        winapi::um::consoleapi::AllocConsole();
        APP = Some(app);
    }

    if client_api::samp::version::is_unknown_version() {
        client_api::utils::error_message_box(
            "Unsupported SA:MP",
            "You have installed an unsupported SA:MP version.\nCurrently supported versions are 0.3.7 R1 and R3.",
        );

        return; // don't waste time
    } else {
        println!(
            "detected version of SAMP is {:?}",
            client_api::samp::version::version()
        );
    }

    crate::render::initialize(manager);

    // apply hook to WndProc
    while !wndproc::initialize(&wndproc::WndProcSettings {
        callback: mainloop,
        hwnd: client_api::gta::hwnd(),
    }) {
        std::thread::sleep(Duration::from_millis(10));
    }

    client_api::wndproc::append_callback(win_event);
}

pub fn uninitialize() {
    crate::render::uninitialize();
    client_api::wndproc::uninitialize();
}

// inside GTA thread
fn mainloop() {
    if let Some(app) = App::get() {
        if !app.init {
            let mut manager = app.manager.lock().unwrap();
            //            manager.create_browser("http://127.0.0.1:5000/index.html"); // "http://5.63.153.185"
            manager.create_browser("http://5.63.153.185/hud.html");

            app.init = true;
        }
    }
}

fn win_event(msg: UINT, wparam: WPARAM, lparam: LPARAM) -> bool {
    if let Some(app) = App::get() {
        let mut manager = app.manager.lock().unwrap();

        match msg {
            WM_MOUSEMOVE => {
                let [x, y] = [
                    ((lparam as u16) & 0xFFFF) as i32,
                    (lparam >> 16) as u16 as i32,
                ];

                manager.send_mouse_move_event(x, y);
            }

            WM_LBUTTONDOWN => manager.send_mouse_click_event(MouseKey::Left, true),
            WM_LBUTTONUP => manager.send_mouse_click_event(MouseKey::Left, false),
            WM_RBUTTONDOWN => manager.send_mouse_click_event(MouseKey::Right, true),
            WM_RBUTTONUP => manager.send_mouse_click_event(MouseKey::Right, false),
            WM_MBUTTONDOWN => manager.send_mouse_click_event(MouseKey::Middle, true),
            WM_MBUTTONUP => manager.send_mouse_click_event(MouseKey::Middle, false),

            WM_MOUSEWHEEL => {
                let delta = if (wparam >> 16) as i16 > 0 { 1 } else { -1 };
                manager.send_mouse_wheel(delta);
            }

            WM_KEYDOWN | WM_KEYUP | WM_CHAR | WM_SYSCHAR | WM_SYSKEYDOWN | WM_SYSKEYUP => {
                let is_system_key = msg == WM_SYSCHAR || msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP;

                let mut event: cef_key_event_t = unsafe { std::mem::zeroed() };

                event.windows_key_code = wparam as i32;
                event.native_key_code = lparam as i32;
                event.modifiers = crate::utils::cef_keyboard_modifiers(wparam, lparam);
                event.is_system_key = if is_system_key { 1 } else { 0 };

                if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                    event.type_ = cef_key_event_type_t::KEYEVENT_RAWKEYDOWN;
                } else if msg == WM_KEYUP || msg == WM_SYSKEYUP {
                    event.type_ = cef_key_event_type_t::KEYEVENT_KEYUP;
                } else if msg == WM_CHAR || msg == WM_SYSCHAR {
                    event.type_ = cef_key_event_type_t::KEYEVENT_CHAR;

                    let bytes = [wparam as u8];
                    if let Some(ch) = encoding_rs::WINDOWS_1251.decode(&bytes).0.chars().next() {
                        event.windows_key_code = ch as _;
                    }
                }

                manager.send_keyboard_event(event);
            }
            _ => (),
        }
    }

    return false;
}
