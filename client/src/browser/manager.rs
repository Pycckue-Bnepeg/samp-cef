use crate::app::Event;
use crate::browser::client::WebClient;

use cef::types::list::{List, ValueType};
use cef::types::string::CefString;
use cef_sys::{cef_event_flags_t, cef_key_event_t, cef_mouse_button_type_t, cef_mouse_event_t};

use std::collections::HashMap;
use std::sync::Arc;

use crossbeam_channel::Sender;

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

pub struct Manager {
    clients: HashMap<u32, Arc<WebClient>>,
    event_tx: Sender<Event>,
    mouse: Mouse,
    view_width: usize,
    view_height: usize,
}

impl Manager {
    pub fn new(event_tx: Sender<Event>) -> Manager {
        // init cef
        crate::browser::cef::initialize();

        let mut keys = HashMap::new();

        keys.insert(MouseKey::Left, false);
        keys.insert(MouseKey::Middle, false);
        keys.insert(MouseKey::Right, false);

        let mouse = Mouse { x: 0, y: 0, keys };

        Manager {
            clients: HashMap::new(),
            view_height: 0,
            view_width: 0,
            mouse,
            event_tx,
        }
    }

    pub fn create_browser(&mut self, id: u32, url: &str) {
        let client = WebClient::new(self.event_tx.clone());
        crate::browser::cef::create_browser(client.clone(), url);

        if let Some(client) = self.clients.insert(id, client) {
            client
                .browser()
                .map(|br| br.host())
                .map(|host| host.close_browser(true));
        }
    }

    pub fn draw(&self) {
        for (_, browser) in &self.clients {
            browser.update_view();
            browser.draw();
        }
    }

    pub fn on_lost_device(&self) {
        for (_, browser) in &self.clients {
            browser.on_lost_device();
        }
    }

    pub fn on_reset_device(&self) {
        for (_, browser) in &self.clients {
            browser.on_reset_device();
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if width == self.view_width && height == self.view_height {
            return;
        }

        self.view_width = width;
        self.view_height = height;

        for (_, browser) in &self.clients {
            browser.resize(width, height);
        }
    }

    pub fn send_mouse_move_event(&mut self, x: i32, y: i32) {
        for (_, client) in &self.clients {
            if let Some(host) = client.browser().map(|browser| browser.host()) {
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
    }

    pub fn send_mouse_click_event(&mut self, button: MouseKey, is_down: bool) {
        for (_, client) in &self.clients {
            if let Some(host) = client.browser().map(|browser| browser.host()) {
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
    }

    pub fn send_mouse_wheel(&self, delta: i32) {
        for (_, client) in &self.clients {
            if let Some(host) = client.browser().map(|browser| browser.host()) {
                host.send_mouse_wheel(self.mouse.x, self.mouse.y, delta);
            }
        }
    }

    pub fn send_keyboard_event(&self, event: cef_key_event_t) {
        for (_, client) in &self.clients {
            if let Some(host) = client.browser().map(|browser| browser.host()) {
                host.send_keyboard_event(event.clone());
            }
        }
    }

    pub fn trigger_event(&self, event_name: &str, list: List) {
        for (_, client) in &self.clients {
            if let Some(frame) = client.browser().map(|browser| browser.main_frame()) {
                let name = CefString::new(event_name);
                let msg = cef::process_message::ProcessMessage::create("trigger_event");

                let args = msg.argument_list();
                args.set_string(0, &name);

                for idx in 0..list.len() {
                    match list.get_type(idx) {
                        ValueType::String => args.set_string(idx + 1, &list.string(idx)),
                        ValueType::Integer => args.set_integer(idx + 1, list.integer(idx)),
                        ValueType::Double => args.set_double(idx + 1, list.double(idx)),
                        ValueType::Bool => args.set_bool(idx + 1, list.bool(idx)),
                        _ => (),
                    }
                }

                frame.send_process_message(cef::ProcessId::Renderer, msg);
            }
        }
    }

    pub fn close_browser(&mut self, id: u32, force_close: bool) {
        if let Some(client) = self.clients.remove(&id) {
            client
                .browser()
                .map(|br| br.host())
                .map(|host| host.close_browser(force_close));
        }
    }
}
