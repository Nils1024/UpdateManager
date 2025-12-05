pub enum ConnState {
    Connected,
    HandshakeCompleted,
    Update,
    Finished
}

impl PartialEq for ConnState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ConnState::Connected, ConnState::Connected) => true,
            (ConnState::HandshakeCompleted, ConnState::HandshakeCompleted) => true,
            (ConnState::Update, ConnState::Update) => true,
            (ConnState::Finished, ConnState::Finished) => true,
            _ => false
        }
    }
}