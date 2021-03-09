use crossbeam_channel::{Receiver, Sender};
use laminar::{Config, Packet, Socket, SocketEvent};
use log::info;
use messages::{packets, try_into_packet};
use quick_protobuf::deserialize_from_slice;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::client::Client;
use crate::Event;

pub struct Server {
    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,
    sender: Sender<Packet>,
    allowed: HashMap<IpAddr, i32>,
    clients: HashMap<SocketAddr, Client>,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Arc<Mutex<Server>> {
        let cfg = Config {
            heartbeat_interval: Some(Duration::from_secs(2)),
            ..Default::default()
        };

        let mut socket = Socket::bind_with_config(addr, cfg).unwrap();

        let sender = socket.get_packet_sender();

        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let server = Server {
            event_tx,
            event_rx,
            sender,
            allowed: HashMap::new(),
            clients: HashMap::new(),
        };

        let server = Arc::new(Mutex::new(server));
        let server_clone = server.clone();

        std::thread::spawn(move || loop {
            while let Some(event) = socket.recv() {
                match event {
                    // если послали новый пакет
                    SocketEvent::Packet(packet) => {
                        if let Ok(proto) =
                            deserialize_from_slice::<packets::Packet>(packet.payload())
                        {
                            let mut server = server.lock().unwrap();
                            server.handle_client_packet(packet.addr(), proto);
                        }
                    }

                    // обработка пакетов соединения
                    SocketEvent::Connect(addr) => {
                        let mut server = server.lock().unwrap();
                        server.handle_new_connection(addr);
                    }

                    // таймауты
                    SocketEvent::Timeout(addr) => {
                        let mut server = server.lock().unwrap();
                        server.handle_timeout(addr);
                    }
                }
            }

            socket.manual_poll(Instant::now());
            std::thread::sleep(Duration::from_millis(5));
        });

        server_clone
    }

    // voice server side

    /// обработка пакетов от клиентов
    fn handle_client_packet(&mut self, addr: SocketAddr, packet: packets::Packet) {
        use messages::packets::PacketId;

        // клиента нет пшел нахрен
        if !self.clients.contains_key(&addr) {
            return;
        }

        match packet.packet_id {
            PacketId::REQUEST_JOIN => {
                deserialize_from_slice(&packet.bytes).map(|packet| self.handle_auth(addr, packet));
            }

            PacketId::EMIT_EVENT => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_emit_event(addr, packet));
            }

            PacketId::BROWSER_CREATED => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_browser_created(addr, packet));
            }

            _ => (),
        }
    }

    /// обработка пакета авторизации
    fn handle_auth(&mut self, addr: SocketAddr, packet: packets::RequestJoin) {
        let client = self.clients.get_mut(&addr).unwrap(); // safe

        let response = packets::JoinResponse {
            success: true,
            current_version: None,
        };

        client.set_state(crate::client::State::Connected);

        self.event_tx.send(Event::Connected(client.id()));

        try_into_packet(response).map(|bytes| {
            let packet = Packet::unreliable_sequenced(addr, bytes, Some(1));
            self.sender.send(packet);
        });
    }

    fn handle_emit_event(&mut self, addr: SocketAddr, packet: packets::EmitEvent) {
        let client = self.clients.get_mut(&addr).unwrap(); // safe
        let player_id = client.id();

        if let Some(args) = &packet.args {
            let event_name = packet.event_name.to_string();
            let args = args.to_string();
            let event = Event::EmitEvent(player_id, event_name, args);

            self.event_tx.send(event);
        }
    }

    fn handle_browser_created(&mut self, addr: SocketAddr, packet: packets::BrowserCreated) {
        let client = self.clients.get_mut(&addr).unwrap(); // safe
        let player_id = client.id();

        let event = Event::BrowserCreated(player_id, packet.browser_id, packet.status_code);
        self.event_tx.send(event);
    }

    /// выпинываем игрока из списка клиентов
    fn handle_timeout(&mut self, addr: SocketAddr) {
        self.allowed.remove(&addr.ip());
        self.clients.remove(&addr);
    }

    /// обрабатывает новое входящее соединение
    fn handle_new_connection(&mut self, addr: SocketAddr) {
        if !self.clients.contains_key(&addr) && self.allowed.contains_key(&addr.ip()) {
            let player_id = self.allowed.get(&addr.ip()).unwrap();
            let client = Client::new(player_id.clone(), addr);

            self.clients.insert(addr, client);

            let request = packets::OpenConnection {};

            try_into_packet(request).map(|bytes| {
                let packet = Packet::unreliable_sequenced(addr, bytes, Some(1));
                self.sender.send(packet);
            });
        }
    }

    // samp server side

    pub fn allow_connection(&mut self, player_id: i32, addr: IpAddr) {
        self.allowed.insert(addr, player_id);
    }

    pub fn remove_connection(&mut self, player_id: i32) {
        let addr = self.addr_by_id(player_id);

        if let Some(addr) = addr {
            self.allowed.remove(&addr.ip());
            self.clients.remove(&addr);
        }
    }

    pub fn create_browser(
        &mut self, player_id: i32, browser_id: i32, url: String, hidden: bool, focused: bool,
    ) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::CreateBrowser {
                    browser_id: browser_id as u32,
                    url: url.into(),
                    hidden,
                    focused,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn destroy_browser(&mut self, player_id: i32, browser_id: i32) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::DestroyBrowser {
                    browser_id: browser_id as u32,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn hide_browser(&mut self, player_id: i32, browser_id: i32, hide: bool) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::HideBrowser {
                    browser_id: browser_id as u32,
                    hide,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn focus_browser(&mut self, player_id: i32, browser_id: i32, focused: bool) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::FocusBrowser {
                    browser_id: browser_id as u32,
                    focused,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn emit_event(&mut self, player_id: i32, event: &str, arguments: Vec<packets::EventValue>) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::EmitEvent {
                    event_name: event.into(),
                    args: None,
                    arguments,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::reliable_ordered(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn always_listen_keys(&mut self, player_id: i32, browser_id: i32, listen: bool) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::AlwaysListenKeys {
                    browser_id: browser_id as u32,
                    listen,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn has_plugin(&self, player_id: i32) -> bool {
        self.addr_by_id(player_id)
            .map(|addr| self.clients.contains_key(&addr))
            .unwrap_or(false)
    }

    pub fn create_external_browser(
        &mut self, player_id: i32, browser_id: i32, texture: String, url: String, scale: i32,
    ) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::CreateExternalBrowser {
                    browser_id: browser_id as u32,
                    url: url.into(),
                    texture: texture.into(),
                    scale,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn append_to_object(&mut self, player_id: i32, browser_id: i32, object_id: i32) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::AppendToObject {
                    browser_id: browser_id as u32,
                    object_id,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn remove_from_object(&mut self, player_id: i32, browser_id: i32, object_id: i32) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::RemoveFromObject {
                    browser_id: browser_id as u32,
                    object_id,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn toggle_dev_tools(&mut self, player_id: i32, browser_id: i32, enabled: bool) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::ToggleDevTools {
                    browser_id: browser_id as u32,
                    enabled,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn set_audio_settings(
        &mut self, player_id: i32, browser_id: u32, max_distance: f32, reference_distance: f32,
    ) {
        if let Some(addr) = self.addr_by_id(player_id) {
            let Server {
                ref mut clients,
                ref mut sender,
                ..
            } = self;

            clients.get_mut(&addr).map(|client| {
                let packet = packets::SetAudioSettings {
                    browser_id,
                    max_distance,
                    reference_distance,
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::unreliable_sequenced(client.addr(), bytes.clone(), Some(1));
                sender.send(packet);
            });
        }
    }

    pub fn receiver(&self) -> Receiver<Event> {
        self.event_rx.clone()
    }

    // utils

    fn addr_by_id(&self, player_id: i32) -> Option<SocketAddr> {
        self.clients
            .iter()
            .find(|(_, client)| client.id() == player_id)
            .map(|(&addr, _)| addr.clone())
    }
}
