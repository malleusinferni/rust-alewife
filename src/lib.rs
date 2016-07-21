//! Alewife is an asynchronous message bus with a publish-subscribe interface.
//!
//! Each message contains one Topic value and one Content value. Clients are
//! registered as subscribers during initial network setup, providing a list of
//! desired Topics. Subscribers will only receive messages containing the Topic
//! values they specify. After setup, new publishers can continue to be added to
//! the network, and can publish messages with any Topic.
//!
//! The following features are not presently supported:
//!
//!  - Adding publishers during initial network setup
//!  - Adding new subscribers after initial network setup
//!  - Removing subscribers from the network at any time
//!  - Detecting or handling the disappearance of parts of the network

#![warn(missing_docs)]

use std::hash::Hash;
use std::sync::mpsc::{self, Sender, Receiver};
use std::collections::HashMap;

#[cfg(test)]
mod test {
    #[test]
    fn simple_network() {
        use super::*;

        let mut builder = Publisher::new();
        let subscriber = builder.add_subscriber(&["widgets"]);
        let publisher = builder.build();

        publisher.publish("widgets", "sprocket");

        for (topic, content) in subscriber.fetch() {
            println!("{}: {}", topic, content);
        }
    }
}

/// Interface for receiving messages from the network. Created by calling
/// `Builder::add_subscriber()` during network setup.
pub struct Subscriber<Topic, Content> {
    inbox: Receiver<(Topic, Content)>,
}

impl<Topic, Content> Subscriber<Topic, Content> {
    /// Consumes all pending messages in the subscriber's inbox.
    pub fn fetch(&self) -> Vec<(Topic, Content)> {
        // TODO: Instead of a Vec, use some kind of iterator
        let mut messages = vec![];
        while let Ok(message) = self.inbox.try_recv() {
            messages.push(message);
        }
        messages
    }
}

/// Interface for sending messages to the network. To add more publishers, just
/// clone this object and distribute the clones to your clients.
#[derive(Clone)]
pub struct Publisher<Topic: Hash + Eq + Clone, Content: Clone> {
    subscribers: HashMap<Topic, Vec<Sender<(Topic, Content)>>>,
}

impl<Topic: Hash + Eq + Clone, Content: Clone> Publisher<Topic, Content> {
    /// Called to initialize a network.
    pub fn new() -> Builder<Topic, Content> {
        Builder {
            publisher: Publisher {
                subscribers: HashMap::new()
            }
        }
    }

    /// Sends a message to the network. All topic filtering is done in the
    /// calling thread.
    pub fn publish(&self, topic: Topic, content: Content) {
        let outbox = match self.subscribers.get(&topic) {
            Some(o) => o,
            None => return,
        };

        for subscriber in outbox {
            subscriber.send((topic.clone(), content.clone())).unwrap_or(());
        }
    }
}

/// Helper for building networks. Call `build()` to complete initialization.
pub struct Builder<Topic: Hash + Eq + Clone, Content: Clone> {
    publisher: Publisher<Topic, Content>,
}

impl<Topic: Hash + Eq + Clone, Content: Clone> Builder<Topic, Content> {
    /// Adds a subscriber to the network, with a complete list of the Topics it
    /// expects to receive. This list cannot be modified later.
    pub fn add_subscriber(&mut self, topics: &[Topic]) -> Subscriber<Topic, Content> {
        let (tx, rx) = mpsc::channel();
        for topic in topics {
            let topic = topic.clone();
            let subscriber_list = self.publisher.subscribers.entry(topic);
            subscriber_list.or_insert_with(|| Vec::new()).push(tx.clone());
        }

        Subscriber { inbox: rx }
    }

    /// Finishes network setup. No more subscribers can be added after this.
    pub fn build(self) -> Publisher<Topic, Content> {
        self.publisher
    }
}
