use std::sync::Arc;
use crate::comm::conn::Conn;
use crate::comm::conn_state::ConnState;

pub struct Session {
    pub(crate) conn: Arc<Conn>,
    pub(crate) state: ConnState,
    pub(crate) nonce: u32,
    pub(crate) buffer: Vec<u8>,
}

impl Session {
    pub fn change_state(&mut self, new_state: ConnState) {
        self.state = new_state;
    }
}