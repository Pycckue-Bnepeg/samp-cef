use std::net::SocketAddr;

use network::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum State {
    Connecting,
    Connected,
}

#[derive(Debug)]
pub struct Client {
    id: i32, // SA:MP player id
    state: State,
    addr: SocketAddr,
    peer: PeerId,
}

impl Client {
    pub fn new(id: i32, peer: PeerId, addr: SocketAddr) -> Client {
        Client {
            id,
            addr,
            peer,
            state: State::Connecting,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.state == State::Connected
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr.clone()
    }

    pub fn peer(&self) -> PeerId {
        self.peer
    }
}
