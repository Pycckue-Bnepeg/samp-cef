use std::net::SocketAddr;
use std::sync::{Arc, Once};
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use cef::types::list::List;
use cef_sys::{cef_key_event_t, cef_key_event_type_t};

use winapi::shared::minwindef::{LPARAM, UINT, WPARAM};
use winapi::um::winuser::*;

use crate::audio::Audio;
use crate::browser::manager::{Manager, MouseKey};
use crate::external::CallbackList;
use crate::network::NetworkClient;

use client_api::gta::camera::CCamera;
use client_api::gta::menu_manager::CMenuManager;
use client_api::samp::inputs;
use client_api::samp::netgame::NetGame;
use client_api::samp::objects::Object;
use client_api::samp::players::local_player;

use client_api::wndproc;

use crossbeam_channel::{Receiver, Sender};

use detour::GenericDetour;

const CEF_SERVER_PORT_OFFSET: u16 = 2;
pub const CEF_PLUGIN_VERSION: i32 = 0x00_01_00;

static mut APP: Option<App> = None;

pub enum Event {
    Connect(SocketAddr),
    Timeout,
    NetworkError,
    NetworkJoined,
    BadVersion,

    CreateBrowser {
        id: u32,
        url: String,
        hidden: bool,
        focused: bool,
    },

    CreateExternBrowser(ExternalBrowser),

    DestroyBrowser(u32),
    HideBrowser(u32, bool),
    FocusBrowser(u32, bool),
    EmitEvent(String, List),
    EmitEventOnServer(String, String),
    BrowserCreated(u32, i32),
    AppendToObject(u32, i32),
    RemoveFromObject(u32, i32),
    ToggleDevTools(u32, bool),
    SetAudioSettings(u32, crate::audio::BrowserAudioSettings),
    LoadUrl(u32, String),

    CefInitialize,

    AlwaysListenKeys(u32, bool),
    Terminate,
}

#[derive(Debug)]
pub struct ExternalBrowser {
    pub id: u32,
    pub texture: String,
    pub url: String,
    pub scale: i32,
}

pub struct App {
    connected: bool,
    window_focused: bool,
    cef_ready: bool,
    samp_ready: bool,

    manager: Arc<Mutex<Manager>>,
    audio: Arc<Audio>,
    network: Option<NetworkClient>,
    callbacks: CallbackList,
    keystate_hook: GenericDetour<extern "stdcall" fn(i32) -> u16>,
    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,

    //
    key_state: [bool; 512],

    // debug
    initialization: Instant,
}

impl Drop for App {
    fn drop(&mut self) {
        log::trace!("App::drop");

        {
            let mut manager = self.manager.lock();
            manager.close_all_browsers();
            manager.shutdown_cef();
        }

        self.network.take();
        self.audio.terminate();

        quit();
    }
}

impl App {
    pub fn new() -> App {
        log::trace!("App::new()");

        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        log::trace!("Audio::new()");

        let audio = Audio::new();

        log::trace!("Manager::new()");

        let manager = Arc::new(Mutex::new(Manager::new(event_tx.clone(), audio.clone())));

        log::trace!("crate::external::initialize");

        let callbacks = crate::external::initialize(event_tx.clone(), manager.clone());

        let keystate_hook = client_api::utils::find_function::<extern "stdcall" fn(i32) -> u16>(
            "user32.dll",
            "GetAsyncKeyState",
        )
        .map(|func| unsafe {
            let hook = GenericDetour::new(func, async_key_state).unwrap();
            hook.enable().unwrap();
            hook
        })
        .unwrap();

        App {
            connected: false,
            cef_ready: false,
            samp_ready: false,
            window_focused: true,
            network: None,
            initialization: Instant::now(),
            manager,
            keystate_hook,
            event_tx,
            event_rx,
            callbacks,
            audio,

            //
            key_state: [false; 512],
        }
    }

    pub fn initialize_hooks() {
        log::trace!("App::initialize_hooks()");
        log::trace!("Trying to hook WndProc.");

        // apply hook to WndProc
        while !wndproc::initialize(&wndproc::WndProcSettings {
            callback: shitty,
            hwnd: client_api::gta::hwnd(),
        }) {
            std::thread::sleep(Duration::from_millis(10));
        }

        log::trace!("Append WndProc callback.");

        client_api::wndproc::append_callback(win_event);

        log::trace!("Hooking destroy functions.");

        NetGame::on_destroy(|| {
            log::trace!("NetGame::on_destroy calling unitialize");
            uninitialize();
        });

        NetGame::on_reconnect(|| {
            if let Some(app) = App::get() {
                app.disconnect();
            }
        });

        client_api::samp::deathwindow::DeathWindow::on_draw(|| {
            crate::render::render();
        });

        client_api::gta::game::on_shutdown(|| {
            log::trace!("gta::game::on_shutdown calling unitialize");
            uninitialize();
        });

        log::trace!("Initialize done.");
    }

    pub fn connect(&mut self) {
        if let Some(mut addr) = NetGame::get().addr() {
            if !self.samp_ready {
                App::initialize_hooks();
                self.samp_ready = true;
                self.manager.lock().initialize_cef();
            }

            log::trace!("SAMP: CNetGame address: {}", addr);

            addr.set_port(addr.port() + CEF_SERVER_PORT_OFFSET);

            log::trace!(
                "Event::Connect({}). Elapsed {:?}",
                addr,
                self.initialization.elapsed()
            );

            log::trace!("NetworkClient::new");

            let network = NetworkClient::new(self.event_tx.clone());
            network.send(Event::Connect(addr));

            self.network = Some(network);
            self.connected = true;
        }
    }

    pub fn disconnect(&mut self) {
        // disconnected
        log::trace!("App::disconnect");

        crate::external::call_disconnect();

        let mut manager = self.manager.lock();
        manager.close_all_browsers();
        self.network.take();
        self.connected = false;
    }

    pub fn manager(&self) -> Arc<Mutex<Manager>> {
        self.manager.clone()
    }

    fn get<'a>() -> Option<&'a mut App> {
        unsafe { APP.as_mut() }
    }
}

pub fn initialize() {
    log::trace!("app::initialize()");
    log::trace!("App::new() ->");

    let app = App::new();
    let manager = app.manager();

    log::trace!("CEF: crate::render::initalize()");

    crate::render::initialize(manager);

    unsafe {
        APP = Some(app);
    }

    if client_api::samp::version::is_unknown_version() {
        log::error!("unknown samp version");

        client_api::utils::error_message_box(
                "Unsupported SA:MP",
                "You have installed an unsupported SA:MP version.\nCurrently supported versions are 0.3.7 R1 and R3.",
            );

        // don't waste time
    }
}

pub fn uninitialize() {
    static DESTROY: Once = Once::new();

    log::trace!("app::uninitialize()");

    DESTROY.call_once(|| {
        log::trace!("app::uninitialize call once ->");
        unsafe {
            APP.take();
        }
    });
}

fn quit() {
    log::trace!("app::quit()");

    crate::external::quit();
    crate::render::uninitialize();

    client_api::wndproc::uninitialize();
}

fn shitty() {
    if let Some(app) = App::get() {
        if !app.samp_ready {
            log::trace!("SAMP init within {:?}", app.initialization.elapsed());
            app.samp_ready = true;
            app.manager.lock().initialize_cef();
        } else if !app.window_focused {
            mainloop(); //
        }
    }
}

// inside GTA thread
pub fn mainloop() {
    if let Some(app) = App::get() {
        if !app.connected {
            app.connect();
        }

        if !app.samp_ready {
            return;
        }

        {
            let menu = CMenuManager::get();
            let paused = menu.is_active() || !app.window_focused;

            app.audio.set_paused(paused);
            app.audio.set_gain(menu.sfx_volume());

            let show_cursor = {
                let mut manager = app.manager.lock();
                manager.set_corrupted(paused);
                manager.is_input_blocked() && !menu.is_active()
            };

            // do not redraw default cursor
            if show_cursor {
                client_api::samp::inputs::show_cursor(true);
            }
        }

        while let Ok(event) = app.event_rx.try_recv() {
            match event {
                Event::AlwaysListenKeys(browser_id, listen) => {
                    let manager = app.manager.lock();
                    manager.always_listen_keys(browser_id, listen);
                }

                Event::CreateBrowser {
                    id,
                    url,
                    hidden,
                    focused,
                } => {
                    log::trace!(
                        "Request to create browser view with id: {}. URL: {}",
                        id,
                        url
                    );

                    let show_cursor = {
                        let mut manager = app.manager.lock();
                        manager.create_browser(id, app.callbacks.clone(), &url);
                        manager.hide_browser(id, hidden);
                        manager.browser_focus(id, focused);

                        manager.is_input_blocked() && !CMenuManager::is_menu_active()
                    };

                    client_api::samp::inputs::show_cursor(show_cursor);
                }

                Event::CreateExternBrowser(ext) => {
                    log::trace!("Request from server to create external browser with id {}. Texture name: {}", ext.id, ext.texture);
                    let mut manager = app.manager.lock();

                    manager.create_browser_on_texture(&ext, app.callbacks.clone());
                }

                Event::DestroyBrowser(id) => {
                    let mut manager = app.manager.lock();
                    manager.close_browser(id, true);
                }

                Event::HideBrowser(id, hide) => {
                    let manager = app.manager.lock();
                    manager.hide_browser(id, hide);
                }

                Event::FocusBrowser(id, focus) => {
                    let show_cursor = {
                        let mut manager = app.manager.lock();
                        manager.browser_focus(id, focus);
                        manager.is_input_blocked() && !CMenuManager::is_menu_active()
                    };

                    client_api::samp::inputs::show_cursor(show_cursor);
                }

                Event::EmitEvent(event, list) => {
                    let manager = app.manager.lock();
                    manager.trigger_event(&event, list);
                }

                Event::EmitEventOnServer(event, arguments) => {
                    if let Some(network) = app.network.as_mut() {
                        let event = Event::EmitEventOnServer(event, arguments);
                        network.send(event);
                    }
                }

                Event::BrowserCreated(id, code) => {
                    log::trace!(
                        "Browser {} created. Status code: {}. Network available? {}",
                        id,
                        code,
                        app.network.is_some()
                    );

                    if let Some(network) = app.network.as_mut() {
                        let event = Event::BrowserCreated(id, code);
                        network.send(event);
                    }

                    crate::external::browser_created(id, code);
                }

                Event::CefInitialize => {
                    log::trace!("Initialized. Elapsed: {:?}", app.initialization.elapsed());

                    app.cef_ready = true;
                    crate::external::call_initialize();
                }

                Event::AppendToObject(browser, object) => {
                    let mut manager = app.manager.lock();
                    manager.browser_append_to_object(browser, object);
                }

                Event::RemoveFromObject(browser, object) => {
                    let mut manager = app.manager.lock();
                    manager.browser_remove_from_object(browser, object);
                }

                Event::ToggleDevTools(browser, enabled) => {
                    let manager = app.manager.lock();
                    manager.toggle_dev_tools(browser, enabled);
                }

                Event::NetworkJoined => {
                    crate::external::call_connect();
                }

                Event::SetAudioSettings(browser, audio_settings) => {
                    let mut manager = app.manager.lock();
                    manager.set_audio_settings(browser, audio_settings);
                }

                Event::LoadUrl(browser, url) => {
                    let manager = app.manager.lock();
                    manager.load_url(browser, &url);
                }

                _ => (),
            }
        }

        if app.cef_ready && app.connected {
            crate::external::call_mainloop();

            if let Some(local) = local_player() {
                let position = local.position();
                let velocity = local.velocity();
                let matrix = CCamera::get().matrix();

                app.audio.set_position(position);
                app.audio.set_velocity(velocity);
                app.audio.set_orientation(matrix);

                let mut manager = app.manager.lock();

                for browser in manager.external_browsers() {
                    for &object_id in browser.object_ids.iter() {
                        if let Some(object) = Object::get(object_id) {
                            let obj_position = object.position();
                            let velocity = object.velocity();
                            let heading = object.heading();

                            if client_api::utils::distance(&position, &obj_position)
                                <= browser.audio_settings.max_distance
                            {
                                app.audio.set_object_settings(
                                    object_id,
                                    obj_position,
                                    velocity,
                                    heading,
                                    browser.audio_settings,
                                );
                            } else {
                                app.audio.object_mute(object_id);
                            }
                        } else {
                            app.audio.object_mute(object_id);
                        }
                    }
                }
            }
        }
    }
}

// TODO: Save key state. Mouse too?
fn win_event(msg: UINT, wparam: WPARAM, lparam: LPARAM) -> bool {
    if let Some(app) = App::get() {
        let mut manager = app.manager.lock();
        let mut notify_key_down = false;

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

                // notify GTA. should be notified only once
                let key_index = wparam as usize;
                if key_index < 512 {
                    if manager.is_input_blocked() {
                        // allowed keys (screenshot and chat cycle)
                        let is_allowed_key = wparam == VK_F8 as usize || wparam == VK_F7 as usize;

                        if (app.key_state[key_index]
                            && event.type_ == cef_key_event_type_t::KEYEVENT_KEYUP)
                            || is_allowed_key
                        {
                            app.key_state[key_index] = false;
                            notify_key_down = true;
                        }
                    } else if event.type_ != cef_key_event_type_t::KEYEVENT_CHAR {
                        app.key_state[key_index] =
                            event.type_ == cef_key_event_type_t::KEYEVENT_RAWKEYDOWN;
                    }
                }

                let input_active = inputs::Input::is_active()
                    || inputs::Dialog::is_input_focused()
                    || CMenuManager::is_menu_active();

                if !input_active {
                    manager.send_keyboard_event(event);
                }
            }

            WM_ACTIVATE => {
                let status = (wparam & 0xFFFF) as u16;
                let active = status != WA_INACTIVE;

                crate::external::window_active(active);
                app.window_focused = active;
                manager.set_corrupted(!active);
                manager.do_not_draw(!active);

                return false;
            }

            _ => return false,
        }

        // game on pause or the window isn't active
        // allow user to use menu ...
        if manager.is_input_corrupted() {
            return false;
        }

        return manager.is_input_blocked() && !notify_key_down;
    }

    false
}

// TODO: Add ability to return the right AsyncKeyState result
extern "stdcall" fn async_key_state(key: i32) -> u16 {
    if let Some(app) = App::get() {
        let result = app.keystate_hook.call(key);

        if let Some(manager) = app.manager.try_lock() {
            if manager.is_input_blocked() {
                return 0;
            } else {
                return result;
            }
        }
    }

    0
}
