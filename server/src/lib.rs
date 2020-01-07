use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use log::info;

use samp::amx::AmxIdent;
use samp::args::Args;
use samp::prelude::*;
use samp::{exec_public, initialize_plugin, native};

mod client;
mod server;
mod utils;

use messages::packets::EventValue;

use crate::server::Server;
use crate::utils::{handle_result, IdPool};

use crossbeam_channel::Receiver;

const INIT_TIMEOUT: Duration = Duration::from_secs(10);

pub enum Event {
    EmitEvent(i32, String, String),
    Connected(i32),
    BrowserCreated(i32, u32, i32),
}

struct CefPlugin {
    server: Arc<Mutex<Server>>,
    events: HashMap<String, (AmxIdent, String)>,
    event_rx: Receiver<Event>,
    amx_list: Vec<AmxIdent>,
    await_connect: Vec<(i32, Instant)>,
}

impl CefPlugin {
    fn new() -> Self {
        // открывает UDP сокет на 7779 порту для cef

        let ip: IpAddr = std::fs::read_to_string("./server.cfg")
            .ok()
            .and_then(|inner| {
                inner
                    .lines()
                    .find(|line| line.starts_with("bind"))
                    .map(|borrow| borrow.to_string())
                    .and_then(|bind| {
                        bind.split(" ")
                            .skip(1)
                            .next()
                            .map(|borrow| borrow.to_string())
                    })
            })
            .and_then(|addr| addr.parse().ok())
            .unwrap_or_else(|| "0.0.0.0".parse().unwrap());

        let server = Server::new(SocketAddr::from((ip, 7779)));

        let event_rx = {
            let s = server.lock().unwrap();
            s.receiver()
        };

        CefPlugin {
            server,
            event_rx,
            events: HashMap::new(),
            amx_list: Vec::new(),
            await_connect: Vec::new(),
        }
    }

    #[native(name = "cef_on_player_connect")]
    fn on_player_connect(
        &mut self, _amx: &Amx, player_id: i32, player_ip: AmxString,
    ) -> AmxResult<bool> {
        let player_ip = player_ip.to_string();

        if let Ok(addr) = player_ip.parse() {
            let mut server = self.server.lock().unwrap();
            server.allow_connection(player_id, addr);
            println!("Cef::on_player_connect({}, {})", player_id, player_ip);
            self.await_connect.push((player_id, Instant::now()));
        }

        Ok(true)
    }

    #[native(name = "cef_on_player_disconnect")]
    fn on_player_disconnect(&mut self, _: &Amx, player_id: i32) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.remove_connection(player_id);

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

    #[native(name = "cef_listen_client_events")]
    fn browser_listen_events(
        &mut self, _: &Amx, player_id: i32, browser_id: i32, listen: bool,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.browser_listen_events(player_id, browser_id, listen);

        Ok(true)
    }

    #[native(name = "cef_emit_event", raw)]
    fn emit_event(&mut self, _: &Amx, mut args: Args) -> AmxResult<bool> {
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

    #[native(name = "cef_block_input")]
    fn block_input(&mut self, _: &Amx, player_id: i32, block: bool) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.block_input(player_id, block);

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

    // utils
    fn notify_timeout(&mut self) {
        let mut i = 0;
        while i < self.await_connect.len() {
            if self.await_connect[i].1.elapsed() >= INIT_TIMEOUT {
                let (player, instant) = self.await_connect.remove(i);
                println!(
                    "notify_timeout({}, id:{} elapsed:{:?})",
                    i,
                    player,
                    instant.elapsed()
                );
                self.notify_connect(player, false);
            } else {
                i += 1;
            }
        }
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
                Event::EmitEvent(player, event, args) => {
                    if let Some((ident, cb)) = self.events.get(&event) {
                        samp::amx::get(*ident)
                            .map(|amx| exec_public!(amx, &cb, player, &args => string));
                    }
                }

                Event::Connected(player) => {
                    println!("Event::Connected({})", player);
                    self.notify_connect(player, true);

                    self.await_connect
                        .iter()
                        .position(|(player_id, _)| *player_id == player)
                        .map(|idx| {
                            println!("Remove {}", idx);
                            self.await_connect.remove(idx);
                        });
                }

                Event::BrowserCreated(player, browser, code) => {
                    self.notify_browser_created(player, browser, code);
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
    ],
    {
        samp::plugin::enable_process_tick();
        samp::encoding::set_default_encoding(samp::encoding::WINDOWS_1251);

        let plugin = CefPlugin::new();
        return plugin;
    }
);
