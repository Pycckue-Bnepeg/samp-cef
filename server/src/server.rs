use crossbeam_channel::{Receiver, Sender};
use messages::{packets, try_into_packet};
use network::{CertStrategy, Event as SocketEvent, PeerId, Socket};
use quick_protobuf::deserialize_from_slice;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::client::Client;
use crate::Event;

struct Packet {
    peer: PeerId,
    bytes: Vec<u8>,
}

impl Packet {
    fn new(peer: PeerId, bytes: Vec<u8>) -> Packet {
        Packet { peer, bytes }
    }
}

pub struct Server {
    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,
    sender: Sender<Packet>,
    allowed: HashMap<IpAddr, i32>,
    clients: HashMap<PeerId, Client>,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Arc<Mutex<Server>> {
        let mut socket = Socket::new_server(addr, CertStrategy::SelfSigned).unwrap();

        let (sender, receiver) = crossbeam_channel::unbounded();
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
                    SocketEvent::Message(peer, bytes) => {
                        if let Ok(proto) = deserialize_from_slice::<packets::Packet>(&bytes) {
                            let mut server = server.lock().unwrap();
                            server.handle_client_packet(peer, proto);
                        }
                    }

                    // обработка пакетов соединения
                    SocketEvent::Connected(peer, addr) => {
                        let mut server = server.lock().unwrap();
                        server.handle_new_connection(peer, addr);
                    }

                    // таймауты
                    SocketEvent::Disconnect(peer, _) => {
                        let mut server = server.lock().unwrap();
                        server.handle_timeout(peer);
                    }

                    _ => (),
                }
            }

            for packet in receiver.try_iter() {
                socket.send_message(packet.peer, packet.bytes);
            }

            std::thread::sleep(Duration::from_millis(5));
        });

        server_clone
    }

    // voice server side

    /// обработка пакетов от клиентов
    fn handle_client_packet(&mut self, peer: PeerId, packet: packets::Packet) {
        use messages::packets::PacketId;

        // клиента нет пшел нахрен
        if !self.clients.contains_key(&peer) {
            return;
        }

        match packet.packet_id {
            PacketId::REQUEST_JOIN => {
                deserialize_from_slice(&packet.bytes).map(|packet| self.handle_auth(peer, packet));
            }

            PacketId::EMIT_EVENT => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_emit_event(peer, packet));
            }

            PacketId::BROWSER_CREATED => {
                deserialize_from_slice(&packet.bytes)
                    .map(|packet| self.handle_browser_created(peer, packet));
            }

            _ => (),
        }
    }

    /// обработка пакета авторизации
    fn handle_auth(&mut self, peer: PeerId, packet: packets::RequestJoin) {
        let client = self.clients.get_mut(&peer).unwrap(); // safe

        let response = packets::JoinResponse {
            success: true,
            current_version: None,
        };

        client.set_state(crate::client::State::Connected);

        self.event_tx.send(Event::Connected(client.id()));

        try_into_packet(response).map(|bytes| {
            let packet = Packet::new(peer, bytes);
            self.sender.send(packet);
        });
    }

    fn handle_emit_event(&mut self, peer: PeerId, packet: packets::EmitEvent) {
        let client = self.clients.get_mut(&peer).unwrap(); // safe
        let player_id = client.id();

        if let Some(args) = &packet.args {
            let event_name = packet.event_name.to_string();
            let args = args.to_string();
            let event = Event::EmitEvent(player_id, event_name, args);

            self.event_tx.send(event);
        }
    }

    fn handle_browser_created(&mut self, peer: PeerId, packet: packets::BrowserCreated) {
        let client = self.clients.get_mut(&peer).unwrap(); // safe
        let player_id = client.id();

        let event = Event::BrowserCreated(player_id, packet.browser_id, packet.status_code);
        self.event_tx.send(event);
    }

    /// выпинываем игрока из списка клиентов
    fn handle_timeout(&mut self, addr: PeerId) {
        self.clients.remove(&addr);
    }

    /// обрабатывает новое входящее соединение
    fn handle_new_connection(&mut self, peer: PeerId, addr: SocketAddr) {
        if !self.clients.contains_key(&peer) && self.allowed.contains_key(&addr.ip()) {
            let player_id = self.allowed.get(&addr.ip()).unwrap();
            let client = Client::new(player_id.clone(), peer, addr);

            self.clients.insert(peer, client);

            let request = packets::OpenConnection {};

            try_into_packet(request).map(|bytes| {
                let packet = Packet::new(peer, bytes);
                self.sender.send(packet);
            });
        }
    }

    // samp server side

    pub fn allow_connection(&mut self, player_id: i32, addr: IpAddr) {
        self.allowed.insert(addr, player_id);
    }

    pub fn remove_connection(&mut self, player_id: i32) {
        let addr = self.peer_by_id(player_id);

        if let Some(addr) = addr {
            if let Some(client) = self.clients.remove(&addr) {
                self.allowed.remove(&client.addr().ip());
            }
        }
    }

    pub fn create_browser(
        &mut self, player_id: i32, browser_id: i32, url: String, hidden: bool, focused: bool,
    ) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn destroy_browser(&mut self, player_id: i32, browser_id: i32) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn hide_browser(&mut self, player_id: i32, browser_id: i32, hide: bool) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn focus_browser(&mut self, player_id: i32, browser_id: i32, focused: bool) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn emit_event(&mut self, player_id: i32, event: &str, arguments: Vec<packets::EventValue>) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn always_listen_keys(&mut self, player_id: i32, browser_id: i32, listen: bool) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn has_plugin(&self, player_id: i32) -> bool {
        self.peer_by_id(player_id)
            .map(|addr| self.clients.contains_key(&addr))
            .unwrap_or(false)
    }

    pub fn create_external_browser(
        &mut self, player_id: i32, browser_id: i32, texture: String, url: String, scale: i32,
    ) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn append_to_object(&mut self, player_id: i32, browser_id: i32, object_id: i32) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn remove_from_object(&mut self, player_id: i32, browser_id: i32, object_id: i32) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn toggle_dev_tools(&mut self, player_id: i32, browser_id: i32, enabled: bool) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn set_audio_settings(
        &mut self, player_id: i32, browser_id: u32, max_distance: f32, reference_distance: f32,
    ) {
        if let Some(addr) = self.peer_by_id(player_id) {
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
                let packet = Packet::new(client.peer(), bytes);
                sender.send(packet);
            });
        }
    }

    pub fn receiver(&self) -> Receiver<Event> {
        self.event_rx.clone()
    }

    // utils

    fn peer_by_id(&self, player_id: i32) -> Option<PeerId> {
        self.clients
            .iter()
            .find(|(_, client)| client.id() == player_id)
            .map(|(&peer, _)| peer.clone())
    }
}
