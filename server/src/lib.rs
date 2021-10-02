use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use log::info;
use messages::packets::EventValue;

use samp::amx::AmxIdent;
use samp::args::Args;
use samp::prelude::*;
use samp::{exec_public, initialize_plugin, native};

mod client;
mod server;
mod utils;

use crate::server::Server;

const INIT_TIMEOUT: Duration = Duration::from_secs(5);
const PORT_OFFSET: u16 = 2;

pub enum Event {
    EmitEvent {
        player_id: i32,
        event: String,
        arguments: String,
    },
    PlayerConnected(i32),
    BrowserCreated {
        player_id: i32,
        browser_id: u32,
        code: i32,
    },
}

struct CefPlugin {
    server: Arc<Mutex<Server>>,
    events: HashMap<String, (AmxIdent, String)>,
    event_rx: Receiver<Event>,
    amx_list: Vec<AmxIdent>,
    await_connect: HashMap<i32, Instant>,
}

impl CefPlugin {
    fn new() -> Self {
        let ip: IpAddr =
            crate::utils::parse_config_field("bind").unwrap_or_else(|| "0.0.0.0".parse().unwrap());

        let port = crate::utils::parse_config_field("port").unwrap_or_else(|| 7777);
        let addr = SocketAddr::from((ip, port + PORT_OFFSET));
        let server = Server::new(addr);

        info!("Bind CEF server on {:?}", addr);

        let event_rx = {
            let s = server.lock().unwrap();
            s.receiver()
        };

        CefPlugin {
            server,
            event_rx,
            events: HashMap::new(),
            amx_list: Vec::new(),
            await_connect: HashMap::new(),
        }
    }

    #[native(name = "cef_on_player_connect")]
    fn on_player_connect(
        &mut self, _amx: &Amx, player_id: i32, player_ip: AmxString,
    ) -> AmxResult<bool> {
        let player_ip = player_ip.to_string();

        if let Ok(addr) = player_ip.parse() {
            {
                let mut server = self.server.lock().unwrap();
                server.allow_connection(player_id, addr);
            }

            self.add_to_await_list(player_id);
        }

        Ok(true)
    }

    #[native(name = "cef_on_player_disconnect")]
    fn on_player_disconnect(&mut self, _: &Amx, player_id: i32) -> AmxResult<bool> {
        {
            let mut server = self.server.lock().unwrap();
            server.remove_connection(player_id);
        }

        self.remove_from_await_list(player_id);

        Ok(true)
    }

    #[native(name = "cef_create_browser")]
    fn create_browser(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, url: AmxString, hidden: bool,
        focused: bool,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.create_browser(player_id, browser_id, url.to_string(), hidden, focused);

        Ok(true)
    }

    #[native(name = "cef_destroy_browser")]
    fn destroy_browser(&mut self, _: &Amx, player_id: i32, browser_id: i32) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.destroy_browser(player_id, browser_id);

        Ok(true)
    }

    #[native(name = "cef_hide_browser")]
    fn hide_browser(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, hide: bool,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.hide_browser(player_id, browser_id, hide);

        Ok(true)
    }

    #[native(name = "cef_focus_browser")]
    fn browser_listen_events(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, focused: bool,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.focus_browser(player_id, browser_id, focused);

        Ok(true)
    }

    #[native(name = "cef_emit_event", raw)]
    fn emit_event(&mut self, _: &Amx, args: Args) -> AmxResult<bool> {
        if args.count() < 2 || (args.count() - 2) % 2 != 0 {
            info!("cef_emit_event invalid count of arguments");
            return Ok(false);
        }

        let mut arguments = Vec::with_capacity((args.count() - 2) / 2);

        let player_id = args.get::<i32>(0).unwrap();
        let event_name = args.get::<AmxString>(1).unwrap().to_string();

        let mut idx = 2;

        loop {
            if idx >= args.count() {
                break;
            }

            if let Some(ty) = args.get::<Ref<i32>>(idx) {
                idx += 1;

                let arg = match *ty {
                    0 => EventValue {
                        string_value: Some(args.get::<AmxString>(idx).unwrap().to_string().into()),
                        float_value: None,
                        integer_value: None,
                    },

                    1 => EventValue {
                        string_value: None,
                        float_value: None,
                        integer_value: Some(*args.get::<Ref<i32>>(idx).unwrap()),
                    },

                    2 => EventValue {
                        string_value: None,
                        float_value: Some(*args.get::<Ref<f32>>(idx).unwrap()),
                        integer_value: None,
                    },

                    _ => break,
                };

                arguments.push(arg);

                idx += 1;
            } else {
                break;
            }
        }

        let mut server = self.server.lock().unwrap();
        server.emit_event(player_id, &event_name, arguments);

        Ok(true)
    }

    #[native(name = "cef_always_listen_keys")]
    fn block_input(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, listen: bool,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.always_listen_keys(player_id, browser_id, listen);

        Ok(true)
    }

    #[native(name = "cef_subscribe")]
    fn subscribe(
        &mut self, amx: &Amx, event_name: AmxString, callback: AmxString,
    ) -> AmxResult<bool> {
        let ident = amx.ident();
        let event_name = event_name.to_string();
        let callback = callback.to_string();

        self.events.insert(event_name, (ident, callback));

        Ok(true)
    }

    #[native(name = "cef_player_has_plugin")]
    fn is_player_has_plugin(&mut self, _: &Amx, player_id: i32) -> AmxResult<bool> {
        let server = self.server.lock().unwrap();
        let has_plugin = server.has_plugin(player_id);
        Ok(has_plugin)
    }

    #[native(name = "cef_create_ext_browser")]
    fn create_external_browser(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, texture: AmxString, url: AmxString,
        scale: i32,
    ) -> AmxResult<bool> {
        let texture = texture.to_string();
        let url = url.to_string();

        let mut server = self.server.lock().unwrap();
        server.create_external_browser(player_id, browser_id, texture, url, scale);

        Ok(true)
    }

    #[native(name = "cef_append_to_object")]
    fn append_to_object(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, object_id: i32,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.append_to_object(player_id, browser_id, object_id);
        Ok(true)
    }

    #[native(name = "cef_remove_from_object")]
    fn remove_from_object(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, object_id: i32,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.remove_from_object(player_id, browser_id, object_id);
        Ok(true)
    }

    #[native(name = "cef_toggle_dev_tools")]
    fn toggle_dev_tools(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, enabled: bool,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.toggle_dev_tools(player_id, browser_id, enabled);
        Ok(true)
    }

    #[native(name = "cef_set_audio_settings")]
    fn set_audio_settings(
        &mut self, _: &Amx, player_id: i32, browser_id: u32, max_distance: f32,
        reference_distance: f32,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.set_audio_settings(player_id, browser_id, max_distance, reference_distance);
        Ok(true)
    }

    #[native(name = "cef_load_url")]
    fn load_url(
        &mut self, _: &Amx, player_id: i32, browser_id: u32, url: AmxString,
    ) -> AmxResult<bool> {
        let url = url.to_string();
        let mut server = self.server.lock().unwrap();

        server.load_url(player_id, browser_id, url);

        Ok(true)
    }

    // utils
    fn notify_timeout(&mut self) {
        let mut keys = Vec::new();

        for (&player_id, timing) in self.await_connect.iter() {
            if timing.elapsed() >= INIT_TIMEOUT {
                keys.push(player_id);
                self.notify_connect(player_id, false);
            }
        }

        keys.into_iter().for_each(|player_id| {
            self.remove_from_await_list(player_id);
        });
    }

    fn notify_connect(&self, player_id: i32, success: bool) {
        self.amx_list.iter().for_each(|&ident| {
            samp::amx::get(ident)
                .map(|amx| exec_public!(amx, "OnCefInitialize", player_id, success));
        });
    }

    fn notify_browser_created(&self, player_id: i32, browser_id: u32, code: i32) {
        self.amx_list.iter().for_each(|&ident| {
            samp::amx::get(ident)
                .map(|amx| exec_public!(amx, "OnCefBrowserCreated", player_id, browser_id, code));
        });
    }

    fn add_to_await_list(&mut self, player_id: i32) {
        self.await_connect.insert(player_id, Instant::now());
    }

    fn remove_from_await_list(&mut self, player_id: i32) -> bool {
        self.await_connect.remove(&player_id).is_some()
    }
}

impl SampPlugin for CefPlugin {
    fn on_load(&mut self) {
        info!("CEF plugin is successful loaded.");
    }

    fn on_amx_load(&mut self, amx: &Amx) {
        self.amx_list.push(amx.ident());
    }

    fn on_amx_unload(&mut self, amx: &Amx) {
        let ident = amx.ident();

        if let Some(position) = self.amx_list.iter().position(|&id| id == ident) {
            self.amx_list.remove(position);
        }
    }

    fn process_tick(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                Event::EmitEvent {
                    player_id,
                    event,
                    arguments,
                } => {
                    if let Some((ident, cb)) = self.events.get(&event) {
                        samp::amx::get(*ident)
                            .map(|amx| exec_public!(amx, &cb, player_id, &arguments => string));
                    }
                }

                Event::PlayerConnected(player) => {
                    if self.remove_from_await_list(player) {
                        self.notify_connect(player, true);
                    }
                }

                Event::BrowserCreated {
                    player_id,
                    browser_id,
                    code,
                } => {
                    self.notify_browser_created(player_id, browser_id, code);
                }

                _ => (),
            }
        }

        self.notify_timeout();
    }
}

initialize_plugin!(
    natives: [
        CefPlugin::on_player_connect,
        CefPlugin::on_player_disconnect,
        CefPlugin::create_browser,
        CefPlugin::destroy_browser,
        CefPlugin::emit_event,
        CefPlugin::subscribe,
        CefPlugin::block_input,
        CefPlugin::hide_browser,
        CefPlugin::browser_listen_events,
        CefPlugin::is_player_has_plugin,
        CefPlugin::create_external_browser,
        CefPlugin::append_to_object,
        CefPlugin::remove_from_object,
        CefPlugin::toggle_dev_tools,
        CefPlugin::set_audio_settings,
        CefPlugin::load_url,
    ],
    {
        samp::plugin::enable_process_tick();
        samp::encoding::set_default_encoding(samp::encoding::WINDOWS_1251);
        let _ = samp::plugin::logger(); // fuck logger

        let plugin = CefPlugin::new();
        return plugin;
    }
);
