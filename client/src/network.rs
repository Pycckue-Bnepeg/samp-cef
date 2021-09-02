use client_api::utils::handle_result;
use crossbeam_channel::{Receiver, Sender};
use laminar::{Config, Packet, Socket, SocketEvent};
use messages::packets;
use quick_protobuf::deserialize_from_slice;

use crate::app::{Event, ExternalBrowser};

use std::net::SocketAddr;
use std::time::{Duration, Instant};

pub struct NetworkClient {
    event_tx: Sender<Event>,
}

impl NetworkClient {
    pub fn new(net_tx: Sender<Event>) -> NetworkClient {
        let (client_tx, client_rx) = crossbeam_channel::unbounded();

        std::thread::spawn(move || {
            if let Some(network) = Network::new(net_tx.clone(), client_rx) {
                network.run();
            } else {
                log::trace!("network error ...");
                std::thread::sleep(Duration::from_secs(2));
                handle_result(net_tx.send(Event::NetworkError));
            }
        });

        NetworkClient {
            event_tx: client_tx,
        }
    }

    pub fn send(&self, message: Event) {
        handle_result(self.event_tx.send(message));
    }
}

impl Drop for NetworkClient {
    fn drop(&mut self) {
        self.send(Event::Terminate);
    }
}

#[derive(Debug, Clone, Copy)]
enum ConnectionState {
    Auth(SocketAddr, Instant),
    Connected(SocketAddr),
    Disconnected,
}

impl ConnectionState {
    fn addr(&self) -> Option<SocketAddr> {
        match self {
            ConnectionState::Auth(addr, _) => Some(addr.clone()),
            ConnectionState::Connected(addr) => Some(addr.clone()),
            _ => None,
        }
    }
}

struct Network {
    socket: Socket,
    connection_state: ConnectionState,

    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,

    timings: Instant,
}

impl Network {
    fn new(event_tx: Sender<Event>, event_rx: Receiver<Event>) -> Option<Network> {
        let cfg = Config {
            heartbeat_interval: Some(Duration::from_secs(2)),
            ..Default::default()
        };

        let socket = handle_result(Socket::bind_with_config("0.0.0.0:0", cfg))?;

        Some(Network {
            connection_state: ConnectionState::Disconnected,
            timings: Instant::now(),
            socket,
            event_tx,
            event_rx,
        })
    }

    fn handle_packet(&mut self, packet: packets::Packet) {
        use packets::PacketId::*;

        match packet.packet_id {
            OPEN_CONNECTION => {
                if let ConnectionState::Auth(addr, _) = &self.connection_state {
                    self.net_connect(*addr);
                } else {
                    log::trace!(
                        "CEF Network: Got OpenConnection from server, but connection_state is {:?}",
                        self.connection_state
                    );
                }
            }

            JOIN_RESPONSE => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_join_response(packet))
                    .ok();
            }

            CREATE_BROWSER => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_create_browser(packet))
                    .ok();
            }

            DESTROY_BROWSER => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_destroy_browser(packet))
                    .ok();
            }

            HIDE_BROWSER => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_hide_browser(packet))
                    .ok();
            }

            ALWAYS_LISTEN_KEYS => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_always_listen_keys(packet))
                    .ok();
            }

            FOCUS_BROWSER => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_focus_browser(packet))
                    .ok();
            }

            EMIT_EVENT => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_emit_event(packet))
                    .ok();
            }

            CREATE_EXTERNAL_BROWSER => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_create_external_browser(packet))
                    .ok();
            }

            APPEND_TO_OBJECT => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_append_to_object(packet))
                    .ok();
            }

            REMOVE_FROM_OBJECT => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_remove_from_object(packet))
                    .ok();
            }

            TOGGLE_DEV_TOOLS => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_toggle_dev_tools(packet))
                    .ok();
            }

            SET_AUDIO_SETTINGS => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_set_audio_settings(packet))
                    .ok();
            }

            _ => (),
        }
    }

    fn handle_join_response(&mut self, packet: packets::JoinResponse) {
        if let ConnectionState::Auth(addr, _) = self.connection_state {
            if packet.success {
                self.connection_state = ConnectionState::Connected(addr);
            } else {
                self.connection_state = ConnectionState::Disconnected;
                handle_result(self.event_tx.send(Event::BadVersion));
            }

            log::trace!("CEF Network: JoinResponse OK. {:?}", self.connection_state);

            handle_result(self.event_tx.send(Event::NetworkJoined));
        } else {
            log::trace!(
                "CEF Network: JoinResponse error. {:?}",
                self.connection_state
            );
        }
    }

    fn handle_create_browser(&mut self, packet: packets::CreateBrowser) {
        let event = Event::CreateBrowser {
            id: packet.browser_id,
            url: packet.url.to_string(),
            hidden: packet.hidden,
            focused: packet.focused,
        };

        handle_result(self.event_tx.send(event));
    }

    fn handle_destroy_browser(&mut self, packet: packets::DestroyBrowser) {
        handle_result(self.event_tx.send(Event::DestroyBrowser(packet.browser_id)));
    }

    fn handle_emit_event(&mut self, packet: packets::EmitEvent) {
        let list = cef::types::list::List::new();

        for (idx, arg) in packet.arguments.iter().enumerate() {
            if let Some(f) = arg.float_value {
                list.set_double(idx, f as f64);
            }

            if let Some(i) = arg.integer_value {
                list.set_integer(idx, i);
            }

            if let Some(s) = &arg.string_value {
                let cef_str = cef::types::string::CefString::new(&s);
                list.set_string(idx, &cef_str);
            }
        }

        handle_result(
            self.event_tx
                .send(Event::EmitEvent(packet.event_name.to_string(), list)),
        );
    }

    fn handle_hide_browser(&mut self, packet: packets::HideBrowser) {
        let event = Event::HideBrowser(packet.browser_id, packet.hide);
        handle_result(self.event_tx.send(event));
    }

    fn handle_always_listen_keys(&mut self, packet: packets::AlwaysListenKeys) {
        let event = Event::AlwaysListenKeys(packet.browser_id, packet.listen);
        handle_result(self.event_tx.send(event));
    }

    fn handle_focus_browser(&mut self, packet: packets::FocusBrowser) {
        let event = Event::FocusBrowser(packet.browser_id, packet.focused);
        handle_result(self.event_tx.send(event));
    }

    fn handle_create_external_browser(&mut self, packet: packets::CreateExternalBrowser) {
        let ext = ExternalBrowser {
            id: packet.browser_id,
            texture: packet.texture.to_string(),
            scale: packet.scale,
            url: packet.url.to_string(),
        };

        let event = Event::CreateExternBrowser(ext);
        handle_result(self.event_tx.send(event));
    }

    fn handle_append_to_object(&mut self, packet: packets::AppendToObject) {
        let event = Event::AppendToObject(packet.browser_id, packet.object_id);
        handle_result(self.event_tx.send(event));
    }

    fn handle_remove_from_object(&mut self, packet: packets::RemoveFromObject) {
        let event = Event::RemoveFromObject(packet.browser_id, packet.object_id);
        handle_result(self.event_tx.send(event));
    }

    fn handle_toggle_dev_tools(&mut self, packet: packets::ToggleDevTools) {
        let event = Event::ToggleDevTools(packet.browser_id, packet.enabled);
        handle_result(self.event_tx.send(event));
    }

    fn handle_set_audio_settings(&mut self, packet: packets::SetAudioSettings) {
        let event = Event::SetAudioSettings(
            packet.browser_id,
            crate::audio::BrowserAudioSettings {
                max_distance: packet.max_distance,
                reference_distance: packet.reference_distance,
            },
        );

        handle_result(self.event_tx.send(event));
    }

    fn net_open_connection(&mut self, address: SocketAddr) {
        self.connection_state = ConnectionState::Auth(address, Instant::now());

        let auth = packets::OpenConnection {};

        let packet = messages::try_into_packet(auth).unwrap();
        let packet = Packet::unreliable_sequenced(address, packet, Some(1));

        log::trace!("CEF Network: OpenConnection ({})", address);
        log::trace!(
            "CEF Network: Elapsed since Network module created {:?}",
            self.timings.elapsed()
        );

        handle_result(self.socket.send(packet));
    }

    fn net_connect(&mut self, address: SocketAddr) {
        let auth = packets::RequestJoin {
            plugin_version: crate::app::CEF_PLUGIN_VERSION,
        };

        let packet = messages::try_into_packet(auth).unwrap();
        let packet = Packet::unreliable_sequenced(address, packet, Some(1));

        log::trace!("CEF Network: RequestJoin ({})", address);

        handle_result(self.socket.send(packet));
    }

    fn net_emit_event(&mut self, event: String, args: String) {
        if let ConnectionState::Connected(address) = self.connection_state {
            let emit = packets::EmitEvent {
                event_name: event.into(),
                args: Some(args.into()),
                arguments: Vec::new(),
            };

            let packet = messages::try_into_packet(emit).unwrap();
            let packet = Packet::unreliable_sequenced(address, packet, Some(1));

            handle_result(self.socket.send(packet));
        }
    }

    fn net_browser_created(&mut self, browser_id: u32, status_code: i32) {
        if let ConnectionState::Connected(address) = self.connection_state {
            let created = packets::BrowserCreated {
                browser_id,
                status_code,
            };

            let packet = messages::try_into_packet(created).unwrap();
            let packet = Packet::unreliable_sequenced(address, packet, Some(1));

            handle_result(self.socket.send(packet));
        }
    }

    fn process_network(&mut self) {
        if let Some(addr) = self.connection_state.addr() {
            while let Some(event) = self.socket.recv() {
                match event {
                    SocketEvent::Packet(packet) => {
                        if packet.addr() == addr {
                            if let Err(e) = deserialize_from_slice(packet.payload())
                                .map(|packet| self.handle_packet(packet))
                            {
                                log::trace!("malformed packet from the server: {}", e);
                            }
                        }
                    }

                    SocketEvent::Connect(addr) => log::trace!("connect? {}", addr), // what?

                    SocketEvent::Timeout(timeout_addr) => {
                        if timeout_addr == addr {
                            log::trace!("CEF Network: Timeout");
                            handle_result(self.event_tx.send(Event::Timeout));

                            self.net_open_connection(timeout_addr);
                        }
                    }
                }
            }
        }

        if let ConnectionState::Auth(addr, time) = &self.connection_state {
            if time.elapsed() >= Duration::from_millis(2500) {
                log::trace!("CEF Network: CEF didn't connect. Retrying ...");
                self.net_open_connection(addr.clone());
            }
        }

        self.socket.manual_poll(Instant::now());
    }

    fn process_event(&mut self, event: Event) {
        match event {
            Event::Connect(addr) => self.net_open_connection(addr),
            Event::EmitEventOnServer(event, arguments) => self.net_emit_event(event, arguments),
            Event::BrowserCreated(id, code) => self.net_browser_created(id, code),
            _ => (),
        }
    }

    fn run(mut self) {
        'mainloop: loop {
            self.process_network();

            while let Ok(event) = self.event_rx.try_recv() {
                match event {
                    Event::Terminate => {
                        break 'mainloop;
                    }

                    event => self.process_event(event),
                }
            }

            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
