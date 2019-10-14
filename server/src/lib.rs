use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};

use log::info;
use samp::args::Args;
use samp::prelude::*;
use samp::{initialize_plugin, native};

mod client;
mod server;
mod utils;

use messages::packets::EventValue;

use crate::server::Server;
use crate::utils::{handle_result, IdPool};

struct CefPlugin {
    server: Arc<Mutex<Server>>,
}

impl CefPlugin {
    fn new() -> Self {
        let pool = IdPool::new(10000);
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

        CefPlugin { server }
    }

    #[native(name = "cef_on_player_connect")]
    fn on_player_connect(
        &mut self, _amx: &Amx, player_id: i32, player_ip: AmxString,
    ) -> AmxResult<bool> {
        let player_ip = player_ip.to_string();

        if let Ok(addr) = player_ip.parse() {
            let mut server = self.server.lock().unwrap();
            server.allow_connection(player_id, addr);
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
        &mut self, _: &Amx, player_id: i32, browser_id: i32, url: AmxString,
    ) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.create_browser(player_id, browser_id, url.to_string());

        Ok(true)
    }

    #[native(name = "cef_destroy_browser")]
    fn destroy_browser(&mut self, _: &Amx, player_id: i32, browser_id: i32) -> AmxResult<bool> {
        let mut server = self.server.lock().unwrap();
        server.destroy_browser(player_id, browser_id);

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

    #[native(name = "cef_player_has_plugin")]
    fn is_player_has_plugin(&mut self, _: &Amx, player_id: i32) -> AmxResult<bool> {
        let server = self.server.lock().unwrap();
        let has_plugin = server.has_plugin(player_id);
        Ok(has_plugin)
    }
}

impl SampPlugin for CefPlugin {
    fn on_load(&mut self) {
        info!("Voice chat plugin is successful loaded.");
    }

    fn process_tick(&mut self) {}
}

initialize_plugin!(
    natives: [
        CefPlugin::on_player_connect,
        CefPlugin::on_player_disconnect,
        CefPlugin::create_browser,
        CefPlugin::destroy_browser,
        CefPlugin::emit_event,
        CefPlugin::is_player_has_plugin,
    ],
    {
        samp::plugin::enable_process_tick();

        let plugin = CefPlugin::new();
        return plugin;
    }
);
