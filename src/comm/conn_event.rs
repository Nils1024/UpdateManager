use crate::comm::conn::Conn;
use crate::util;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ConnEventType {
    MsgReceived,
}

#[derive(Clone)]
pub struct ConnEvent {
    pub event_type: ConnEventType,
    pub source: Conn,
    pub timestamp: u64,
    pub payload: String,
}

impl util::observer::identifiable_event::IdentifiableEvent for ConnEvent {
    type Key = ConnEventType;

    fn event_type(&self) -> Self::Key {
        self.event_type.clone()
    }
}