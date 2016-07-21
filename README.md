# Alewife

Alewife is an asynchronous message bus with a publish-subscribe interface.

# Example usage

```rust
extern crate alewife;

fn main() {
    // Begin network setup. Our messages will be (u8, String) pairs.
    let mut bus = alewife::Publisher::<u8, String>::new();

    // The first type parameter (u8) identifies which subscribers to send a
    // message to. This subscriber will receive messages whose topic is 0.
    let sub0 = bus.add_subscriber(&[0]);

    // This one only receives messages with a topic of 1.
    let sub1 = bus.add_subscriber(&[1]);

    // A subscriber can request multiple topics.
    let sub_evens = bus.add_subscriber(&[0, 2, 4, 6, 8]);
    let sub3and5 = bus.add_subscriber(&[3, 5]);

    // Finish network setup. No more subscribers can be created after this.
    let publisher = bus.build();

    // This message will be received by sub0 and sub_evens, but not sub1.
    publisher.publish(0, "test 0".to_owned());

    // This message will be received only by sub1.
    publisher.publish(1, "test 1".to_owned());

    // Fetch all messages received so far. This method is non-blocking.
    println!("{:?}", sub0.fetch()); // [(0, "test 0")]
    println!("{:?}", sub1.fetch()); // [(1, "test 1")]
    println!("{:?}", sub_evens.fetch()); // [(0, "test 0")]

    // You can publish the same message body with multiple topics...
    for i in 0 .. 9 { publisher.publish(i, "test".to_owned()); }

    // ...but they'll count as separate messages.
    println!("{:?}", sub_evens.fetch()); // [(0, "test"), (2, "test"), ...]

    // If you need multiple publishers, just clone them:
    let publisher2 = publisher.clone();
    publisher2.publish(3, "test 3".to_owned());

    // However, note that you can't clone subscribers.
    /* let sub_odds = sub3and5.clone(); */ // Compile error

    // All of this uses async channels under the hood, so it's safe to use
    // alewife to communicate between different threads.
    std::thread::spawn(move || loop {
        let messages = sub3and5.fetch();
        if !messages.is_empty() { println!("{:?}", messages); }
    });

    publisher2.publish(3, "Hello from sub3and5".to_owned());

    // That's all for now.
    std::thread::sleep(std::time::Duration::from_secs(1));
}
```
