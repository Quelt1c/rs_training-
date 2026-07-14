use crate::channel::*;
use std::thread;
use std::time::Duration;
#[test]
fn test_basic_send_and_recv() {
    let (tx, rx) = unbounded();
    tx.send("hello").unwrap();
    assert_eq!(rx.recv().unwrap(), "hello");
}

#[test]
fn test_sender_fails_when_all_receivers_die() {
    let (tx, rx) = unbounded::<i32>();

    drop(rx);

    let result = tx.send(42);
    assert!(
        result.is_err(),
        "The upload was supposed to end with an error"
    );

    if let Err(SendError(msg)) = result {
        assert_eq!(msg, 42);
    }
}

#[test]
fn test_receiver_reads_remaining_then_fails_when_senders_die() {
    let (tx, rx) = unbounded();

    tx.send(1).unwrap();
    tx.send(2).unwrap();

    drop(tx);

    assert_eq!(rx.recv().unwrap(), 1);
    assert_eq!(rx.recv().unwrap(), 2);
    assert!(rx.recv().is_err(), "A RecvError was expected");
}

#[test]
fn test_receiver_wakes_up_when_senders_die() {
    let (tx, rx) = unbounded::<i32>();

    let handle = thread::spawn(move || rx.recv());
    thread::sleep(Duration::from_millis(50));

    drop(tx);

    let result = handle.join().unwrap();
    assert!(
        result.is_err(),
        "The receiver was supposed to wake up and return an error"
    );
}

#[test]
fn test_10_concurrent_senders() {
    let (tx, rx) = unbounded();
    let mut handles = vec![];

    let senders_count = 10;
    let messages_per_sender = 100;

    for _ in 0..senders_count {
        let tx_clone = tx.clone();

        let handle = thread::spawn(move || {
            for i in 0..messages_per_sender {
                tx_clone.send(i).unwrap();
            }
        });
        handles.push(handle);
    }

    drop(tx);

    let mut received_count = 0;
    while let Ok(_msg) = rx.recv() {
        received_count += 1;
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(
        received_count,
        senders_count * messages_per_sender,
        "We were supposed to receive exactly 100 messages"
    );
}
