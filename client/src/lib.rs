#![allow(non_snake_case)]
#![feature(abi_thiscall)]
#![feature(arbitrary_self_types)]

use winapi::shared::d3d9::{IDirect3DDevice9, IDirect3DTexture9};
use winapi::shared::minwindef::{HMODULE, LPARAM, UINT, WPARAM};
use winapi::um::libloaderapi::{DisableThreadLibraryCalls, GetModuleHandleA};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use winapi::um::winuser::*;

use cef::app::App;
use cef::browser::Browser;
use cef::client::Client;
use cef::handlers::lifespan::LifespanHandler;
use cef::handlers::render::{DirtyRects, RenderHandler};
use cef::types::string::{cef_string_t, CefString};

use cef_sys::{cef_key_event_t, cef_key_event_type_t, cef_rect_t};

use std::ptr::{null, null_mut};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod application;
pub mod browser;
pub mod cef_app;
pub mod utils;
pub mod view;

use crate::application::{Application, Event};
use crate::cef_app::MouseKey;
use winapi::shared::windef::RECT;

#[no_mangle]
pub extern "stdcall" fn DllMain(instance: HMODULE, reason: u32, _reserved: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            DisableThreadLibraryCalls(instance);
        }

        std::thread::spawn(|| {
            unsafe {
                winapi::um::consoleapi::AllocConsole();
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

            while !client_api::gta::d3d9::set_proxy(Some(render), Some(reset)) {
                std::thread::sleep(Duration::from_millis(10));
            }

            while !client_api::wndproc::initialize(&client_api::wndproc::WndProcSettings {
                callback: mainloop,
                hwnd: client_api::gta::hwnd(),
            }) {
                std::thread::sleep(Duration::from_millis(10));
            }

            Application::create();

            client_api::wndproc::append_callback(keyboard_events);
        });
    }

    if reason == DLL_PROCESS_DETACH {
        client_api::wndproc::uninitialize();
        client_api::gta::d3d9::unset_proxy();
    }

    return true;
}

fn mainloop() {
    if let Some(app) = Application::get() {
        if !app.render_init {
            app.render_init = true;

            //            let browser = app.manager.create_browser("http://5.63.153.185");
            let browser = app
                .manager
                .create_browser(app.event_tx.clone(), "http://127.0.0.1:5000/index.html");

            app.manager.set_focused(browser);
        }

        while let Ok(event) = app.event_rx.try_recv() {
            match event {
                Event::ShowCursor(show) => {
                    app.manager.trigger_event("test_event");
                    client_api::samp::inputs::show_cursor(show);
                }
            }
        }
    }
}

fn keyboard_events(msg: UINT, wparam: WPARAM, lparam: LPARAM) -> bool {
    if let Some(app) = Application::get() {
        match msg {
            WM_MOUSEMOVE => {
                let [x, y] = [
                    ((lparam as u16) & 0xFFFF) as i32,
                    (lparam >> 16) as u16 as i32,
                ];

                app.manager.send_mouse_move_event(x, y);
            }

            WM_LBUTTONDOWN => app.manager.send_mouse_click_event(MouseKey::Left, true),
            WM_LBUTTONUP => app.manager.send_mouse_click_event(MouseKey::Left, false),
            WM_RBUTTONDOWN => app.manager.send_mouse_click_event(MouseKey::Right, true),
            WM_RBUTTONUP => app.manager.send_mouse_click_event(MouseKey::Right, false),
            WM_MBUTTONDOWN => app.manager.send_mouse_click_event(MouseKey::Middle, true),
            WM_MBUTTONUP => app.manager.send_mouse_click_event(MouseKey::Middle, false),

            WM_MOUSEWHEEL => {
                let delta = if (wparam >> 16) as i16 > 0 { 1 } else { -1 };
                app.manager.send_mouse_wheel(delta);
            }

            WM_KEYDOWN | WM_KEYUP | WM_CHAR | WM_SYSCHAR | WM_SYSKEYDOWN | WM_SYSKEYUP => {
                let is_system_key = msg == WM_SYSCHAR || msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP;

                let mut event: cef_key_event_t = unsafe { std::mem::zeroed() };

                event.windows_key_code = wparam as i32;
                event.native_key_code = lparam as i32;
                event.modifiers = utils::cef_keyboard_modifiers(wparam, lparam);
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

                app.manager.send_keyboard_event(event);
            }
            _ => (),
        }
    }

    return false;
}

fn render(_: &mut IDirect3DDevice9) {
    if let Some(app) = Application::get() {
        app.manager.draw();
    }
}

fn reset(device: &mut IDirect3DDevice9, flag: u8) {
    if let Some(app) = Application::get() {
        if flag == 0 {
            app.manager.on_device_lost();
        } else {
            app.manager.on_reset_device();
        }
    }
}
