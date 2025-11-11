use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Event {
    MsgReceived
}
pub type Subscriber = fn(msg: String);

#[derive(Default)]
pub struct Publisher {
    events: HashMap<Event, Vec<Subscriber>>
}

impl Publisher {
    pub fn subscribe(&mut self, event_type: Event, listener: Subscriber) {
        self.events.entry(event_type.clone()).or_default();
        self.events.get_mut(&event_type).unwrap().push(listener);
    }

    pub fn unsubscribe(&mut self, event_type: Event, listener: Subscriber) {
        self.events
            .get_mut(&event_type).unwrap().retain(|&x| x != listener);
    }

    pub fn notify(&self, event_type: Event, msg: String) {
        let listeners = self.events.get(&event_type).unwrap();
        for listener in listeners {
            listener(msg.clone());
        }
    }
}