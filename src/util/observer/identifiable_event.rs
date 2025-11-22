use std::hash::Hash;

pub trait IdentifiableEvent: Clone + Send + Sync {
    type Key: Eq + Hash + Clone + Send + Sync;
    
    fn event_type(&self) -> Self::Key;
}