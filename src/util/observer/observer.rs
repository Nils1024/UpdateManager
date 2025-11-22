use std::collections::HashMap;
use crate::util::observer::identifiable_event::IdentifiableEvent;

pub type SubscriptionId = u64;

pub struct Publisher<E> where E: IdentifiableEvent {
    listeners: HashMap<E::Key, Vec<(SubscriptionId, Box<dyn Fn(E) + Send + Sync>)>>,
    next_id: SubscriptionId,
}

impl<E> Default for Publisher<E> where E: IdentifiableEvent {
    fn default() -> Self {
        Self { listeners: HashMap::new(), next_id: 0 }
    }
}

impl<E> Publisher<E> where E: IdentifiableEvent {
    pub fn subscribe<F>(&mut self, key: E::Key, callback: F) -> SubscriptionId where F: Fn(E) + Send + Sync + 'static {
        let id = self.next_id;
        self.next_id += 1;

        self.listeners.entry(key).or_default().push((id, Box::new(callback)));
        id
    }

    pub fn unsubscribe(&mut self, key: E::Key, id: SubscriptionId) {
        if let Some(list) = self.listeners.get_mut(&key) {
            list.retain(|(sub_id, _)| *sub_id != id);
        }
    }

    pub fn notify(&self, event: E) {
        let key = event.event_type();

        if let Some(list) = self.listeners.get(&key) {
            for (_, callback) in list {
                callback(event.clone());
            }
        }
    }
}