use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cef::types::list::List;
use cef_sys::{cef_key_event_t, cef_key_event_type_t};

use winapi::shared::minwindef::{LPARAM, UINT, WPARAM};
use winapi::um::winuser::*;

use crate::browser::manager::{Manager, MouseKey};
use crate::network::NetworkClient;

use client_api::samp::netgame::NetGame;
use client_api::samp::Gamestate;
use client_api::wndproc;

use crossbeam_channel::{Receiver, Sender};

// TODO: nice shutdown
//use detour::GenericDetour;

const CEF_SERVER_PORT: u16 = 7779;
pub const CEF_PLUGIN_VERSION: i32 = 0x00_01_00;

static mut APP: Option<App> = None;

pub enum Event {
    Connect(SocketAddr),
    Timeout,
    NetworkError,
    BadVersion,

    CreateBrowser {
        id: u32,
        url: String,
        listen_events: bool,
    },

    DestroyBrowser(u32),
    HideBrowser(u32, bool),
    BrowserListenEvents(u32, bool),
    EmitEvent(String, List),
    EmitEventOnServer(String, String),

    BlockInput(bool),
    Terminate,
}

pub struct App {
    connected: bool,
    input_blocked: bool,

    manager: Arc<Mutex<Manager>>,
    network: Option<NetworkClient>,

    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,
}

impl App {
    pub fn new() -> App {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let manager = Arc::new(Mutex::new(Manager::new(event_tx.clone())));

        App {
            connected: false,
            input_blocked: false,
            network: None,
            manager,
            event_tx,
            event_rx,
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
        if !app.connected && client_api::samp::gamestate() == Gamestate::Connected {
            if let Some(mut addr) = NetGame::get().addr() {
                addr.set_port(CEF_SERVER_PORT);

                let network = NetworkClient::new(app.event_tx.clone());
                network.send(Event::Connect(addr));

                app.network = Some(network);
                app.connected = true;
            }
        }

        while let Ok(event) = app.event_rx.try_recv() {
            match event {
                Event::BlockInput(block) => {
                    app.input_blocked = block;
                    client_api::samp::inputs::show_cursor(block);
                }

                Event::CreateBrowser {
                    id,
                    url,
                    listen_events,
                } => {
                    let mut manager = app.manager.lock().unwrap();
                    manager.create_browser(id, &url, listen_events);
                }

                Event::DestroyBrowser(id) => {
                    let mut manager = app.manager.lock().unwrap();
                    manager.close_browser(id, true);
                }

                Event::HideBrowser(id, hide) => {
                    let manager = app.manager.lock().unwrap();
                    manager.hide_browser(id, hide);
                }

                Event::BrowserListenEvents(id, listen) => {
                    let manager = app.manager.lock().unwrap();
                    manager.browser_listen_events(id, listen);
                }

                Event::EmitEvent(event, list) => {
                    let manager = app.manager.lock().unwrap();
                    manager.trigger_event(&event, list);
                }

                Event::EmitEventOnServer(event, arguments) => {
                    if let Some(network) = app.network.as_mut() {
                        let event = Event::EmitEventOnServer(event, arguments);
                        network.send(event);
                    }
                }

                _ => (),
            }
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

            _ => return false,
        }

        return app.input_blocked;
    }

    return false;
}
