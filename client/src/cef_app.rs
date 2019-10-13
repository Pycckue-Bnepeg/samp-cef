use crate::application::Event;
use crate::browser::WebClient;
use cef::types::string::CefString;
use cef_sys::{cef_event_flags_t, cef_key_event_t, cef_mouse_button_type_t, cef_mouse_event_t};
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use winapi::shared::windef::RECT;
use winapi::um::libloaderapi::GetModuleHandleA;

struct _App;

impl cef::handlers::render_process::RenderProcessHandler for _App {}

impl cef::app::App for _App {
    type RenderProcessHandler = Self;
}

pub fn cef_create() {
    let instance = unsafe { GetModuleHandleA(std::ptr::null()) };

    let main_args = cef_sys::cef_main_args_t { instance };

    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_settings_t>() };

    let path = CefString::new("./cef/renderer.exe");

    settings.size = std::mem::size_of::<cef_sys::cef_settings_t>();
    settings.no_sandbox = 1;
    settings.browser_subprocess_path = path.to_cef_string();
    settings.windowless_rendering_enabled = 1;
    settings.multi_threaded_message_loop = 1;

    cef::initialize::<_App>(&main_args, &settings, None);
}

pub fn create_browser(tx: Sender<Event>, url: &str) -> Arc<WebClient> {
    let mut window_info = unsafe { std::mem::zeroed::<cef_sys::cef_window_info_t>() };

    window_info.parent_window = client_api::gta::hwnd();
    window_info.windowless_rendering_enabled = 1;

    let url = CefString::new(url);

    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_browser_settings_t>() };

    settings.size = std::mem::size_of::<cef_sys::cef_browser_settings_t>();
    settings.windowless_frame_rate = 60;
    settings.javascript_access_clipboard = cef_sys::cef_state_t::STATE_DISABLED;
    settings.javascript_dom_paste = cef_sys::cef_state_t::STATE_DISABLED;
    settings.webgl = cef_sys::cef_state_t::STATE_ENABLED;
    settings.plugins = cef_sys::cef_state_t::STATE_DISABLED;
    settings.javascript = cef_sys::cef_state_t::STATE_ENABLED;

    let client = WebClient::new(tx);
    cef::browser::BrowserHost::create_browser(&window_info, Some(client.clone()), &url, &settings);

    client
}

#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum MouseKey {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Default)]
struct Mouse {
    x: i32,
    y: i32,
    keys: HashMap<MouseKey, bool>,
}

pub struct BrowserManager {
    browsers: Vec<Arc<WebClient>>,
    focused: Option<Arc<WebClient>>,
    mouse: Mouse,
}

impl BrowserManager {
    pub fn new() -> BrowserManager {
        cef_create();

        let mut keys = HashMap::new();

        keys.insert(MouseKey::Left, false);
        keys.insert(MouseKey::Middle, false);
        keys.insert(MouseKey::Right, false);

        let mouse = Mouse { x: 0, y: 0, keys };

        BrowserManager {
            browsers: Vec::new(),
            focused: None,
            mouse,
        }
    }

    pub fn create_browser(&mut self, tx: Sender<Event>, url: &str) -> Arc<WebClient> {
        let browser = create_browser(tx, url);
        self.browsers.push(browser.clone());
        browser
    }

    pub fn set_focused(&mut self, browser: Arc<WebClient>) {
        self.focused = Some(browser);
    }

    pub fn draw(&self) {
        for browser in &self.browsers {
            browser.update_view();
            browser.draw();
        }
    }

    pub fn on_device_lost(&self) {
        for browser in &self.browsers {
            browser.on_device_lost();
        }
    }

    pub fn on_reset_device(&self) {
        for browser in &self.browsers {
            browser.on_reset_device();
        }
    }

    pub fn send_mouse_move_event(&mut self, x: i32, y: i32) {
        if let Some(host) = self
            .focused
            .as_ref()
            .and_then(|client| client.browser())
            .map(|browser| browser.host())
        {
            self.mouse.x = x;
            self.mouse.y = y;

            let keys = &self.mouse.keys;

            let mut event = cef_mouse_event_t { x, y, modifiers: 0 };

            if keys.get(&MouseKey::Left).cloned().unwrap_or(false) {
                event.modifiers |= cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON as u32;
            }

            if keys.get(&MouseKey::Middle).cloned().unwrap_or(false) {
                event.modifiers |= cef_event_flags_t::EVENTFLAG_MIDDLE_MOUSE_BUTTON as u32;
            }

            if keys.get(&MouseKey::Right).cloned().unwrap_or(false) {
                event.modifiers |= cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON as u32;
            }

            host.send_mouse_move(event);
        }
    }

    pub fn send_mouse_click_event(&mut self, button: MouseKey, is_down: bool) {
        if let Some(host) = self
            .focused
            .as_ref()
            .and_then(|client| client.browser())
            .map(|browser| browser.host())
        {
            self.mouse.keys.insert(button, is_down);

            let event = cef_mouse_event_t {
                x: self.mouse.x,
                y: self.mouse.y,
                modifiers: 0,
            };

            let key = match button {
                MouseKey::Left => cef_mouse_button_type_t::MBT_LEFT,
                MouseKey::Middle => cef_mouse_button_type_t::MBT_MIDDLE,
                MouseKey::Right => cef_mouse_button_type_t::MBT_RIGHT,
            };

            host.send_mouse_click(key, event, is_down);
        }
    }

    pub fn send_mouse_wheel(&self, delta: i32) {
        if let Some(host) = self
            .focused
            .as_ref()
            .and_then(|client| client.browser())
            .map(|browser| browser.host())
        {
            host.send_mouse_wheel(self.mouse.x, self.mouse.y, delta);
        }
    }

    pub fn send_keyboard_event(&self, event: cef_key_event_t) {
        if let Some(host) = self
            .focused
            .as_ref()
            .and_then(|client| client.browser())
            .map(|browser| browser.host())
        {
            host.send_keyboard_event(event);
        }
    }

    pub fn trigger_event(&self, event_name: &str) {
        if let Some(frame) = self
            .focused
            .as_ref()
            .and_then(|client| client.browser())
            .map(|brw| brw.main_frame())
        {
            let name = CefString::new(event_name);
            let msg = cef::process_message::ProcessMessage::create("trigger_event");
            let args = msg.argument_list();
            args.set_string(0, &name);

            frame.send_process_message(cef::ProcessId::Renderer, msg);
        }
    }
}
