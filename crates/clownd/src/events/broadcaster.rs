//! Event broadcaster using tokio broadcast channels.

use clown_core::Event;
use tokio::sync::broadcast;
use tracing::debug;

/// Broadcasts events to all subscribed WebSocket clients.
#[derive(Debug)]
pub struct EventBroadcaster {
    sender: broadcast::Sender<Event>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to receive events.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Broadcast an event to all subscribers.
    /// Returns the number of receivers that received the event.
    pub fn broadcast(&self, event: Event) -> usize {
        debug!("Broadcasting event: {:?}", event);
        // send() returns error if there are no receivers, which is fine
        self.sender.send(event).unwrap_or(0)
    }

    /// Get the current number of active subscribers.
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new(256) // Default capacity
    }
}

impl Clone for EventBroadcaster {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}
