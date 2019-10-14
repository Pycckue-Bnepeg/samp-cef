use crossbeam_channel::Sender;
use laminar::{Config, Packet, Socket, SocketEvent};
use log::info;
use messages::{packets, try_into_packet};
use quick_protobuf::deserialize_from_slice;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::client::Client;

pub struct Server {
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

        let server = Server {
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
                deserialize_from_slice(&packet.bytes).map(|packet| self.handle_auth(addr, packet));
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

        try_into_packet(response).map(|bytes| {
            let packet = Packet::reliable_ordered(addr, bytes, None);
            self.sender.send(packet);
        });
    }

    /// выпинываем игрока из списка клиентов
    fn handle_timeout(&mut self, addr: SocketAddr) {
        self.clients.remove(&addr);
    }

    /// обрабатывает новое входящее соединение
    fn handle_new_connection(&mut self, addr: SocketAddr) {
        if !self.clients.contains_key(&addr) && self.allowed.contains_key(&addr.ip()) {
            let player_id = self.allowed.get(&addr.ip()).unwrap();
            let client = Client::new(player_id.clone(), addr);

            self.clients.insert(addr, client);

            let request = packets::RequestJoin { plugin_version: 0 }; // kind of shit

            try_into_packet(request).map(|bytes| {
                let packet = Packet::reliable_ordered(addr, bytes, None);
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

    pub fn create_browser(&mut self, player_id: i32, browser_id: i32, url: String) {
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
                };

                let bytes = try_into_packet(packet).unwrap();
                let packet = Packet::reliable_ordered(client.addr(), bytes.clone(), None);
                sender.send(packet);
            });
        }
    }

    pub fn block_input(&mut self, player_id: i32, block: bool) {}

    pub fn has_plugin(&self, player_id: i32) -> bool {
        self.addr_by_id(player_id)
            .map(|addr| self.clients.contains_key(&addr))
            .unwrap_or(false)
    }

    // utils

    fn addr_by_id(&self, player_id: i32) -> Option<SocketAddr> {
        self.clients
            .iter()
            .find(|(_, client)| client.id() == player_id)
            .map(|(&addr, _)| addr.clone())
    }
}
