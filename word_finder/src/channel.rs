use parking_lot::Mutex;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

#[derive(Debug, PartialEq, Eq)]
pub struct SendError<T>(pub T);

#[derive(Debug, PartialEq, Eq)]
pub struct RecvError;

struct ChannelState<T> {
    queue: VecDeque<T>,
    sender_count: usize,
    receiver_count: usize,
    wakers: VecDeque<(usize, Waker)>,
    next_waker_id: usize,
}

struct Shared<T> {
    state: Mutex<ChannelState<T>>,
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        let mut state = self.shared.state.lock();

        if state.receiver_count == 0 {
            return Err(SendError(msg));
        }

        state.queue.push_back(msg);

        if let Some((_, w)) = state.wakers.pop_front() {
            w.wake_by_ref();
        }

        Ok(())
    }

    pub async fn send_async(&self, msg: T) -> Result<(), SendError<T>> {
        self.send(msg)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.shared.state.lock().sender_count += 1;

        Sender {
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut state = self.shared.state.lock();
        state.sender_count -= 1;

        if state.sender_count == 0 {
            for (_, w) in state.wakers.drain(..) {
                w.wake_by_ref();
            }
        }
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> RecvFuture<T> {
        let mut state = self.shared.state.lock();
        let id = state.next_waker_id;
        state.next_waker_id = state.next_waker_id.wrapping_add(1);

        RecvFuture {
            shared: self.shared.clone(),
            id,
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let mut state = self.shared.state.lock();
        state.receiver_count += 1;
        Receiver {
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.state.lock().receiver_count -= 1;
    }
}

pub struct RecvFuture<T> {
    shared: Arc<Shared<T>>,
    id: usize,
}

impl<T> Future for RecvFuture<T> {
    type Output = Result<T, RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.shared.state.lock();

        if let Some(msg) = state.queue.pop_front() {
            if !state.queue.is_empty() {
                if let Some((_, w)) = state.wakers.pop_front() {
                    w.wake();
                }
            }
            return Poll::Ready(Ok(msg));
        }

        if state.sender_count == 0 {
            return Poll::Ready(Err(RecvError));
        }

        let new_waker = cx.waker();

        state.wakers.retain(|(id, _)| *id != self.id);

        state.wakers.push_back((self.id, new_waker.clone()));

        Poll::Pending
    }
}

impl<T> Drop for RecvFuture<T> {
    fn drop(&mut self) {
        let mut state = self.shared.state.lock();
        state.wakers.retain(|(id, _)| *id != self.id);
    }
}

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        state: Mutex::new(ChannelState {
            queue: VecDeque::new(),
            sender_count: 1,
            receiver_count: 1,
            wakers: VecDeque::new(),
            next_waker_id: 0,
        }),
    });

    (
        Sender {
            shared: shared.clone(),
        },
        Receiver { shared },
    )
}

#[cfg(test)]
mod async_tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_basic_async_flow() {
        let (tx, rx) = unbounded::<i32>();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            assert!(tx.send_async(42).await.is_ok(), "Failed to send message");
        });

        assert!(
            matches!(rx.recv().await, Ok(42)),
            "Receive failed or returned incorrect value"
        );
    }

    #[tokio::test]
    async fn test_stress_mpmc_concurrent() {
        let (tx, rx) = unbounded::<i32>();
        let mut producer_handles = vec![];
        let mut consumer_handles = vec![];

        let num_producers = 10;
        let num_consumers = 10;
        let msgs_per_producer = 1000;

        for i in 0..num_producers {
            let tx_clone = tx.clone();
            producer_handles.push(tokio::spawn(async move {
                for j in 0..msgs_per_producer {
                    let msg = i * 10_000 + j;
                    assert!(
                        tx_clone.send_async(msg).await.is_ok(),
                        "Producer failed to send"
                    );
                }
            }));
        }

        drop(tx);

        let expected_total = num_producers * msgs_per_producer;
        let counter = Arc::new(parking_lot::Mutex::new(0));

        for _ in 0..num_consumers {
            let rx_clone = rx.clone();
            let counter_clone = counter.clone();
            consumer_handles.push(tokio::spawn(async move {
                while let Ok(_) = rx_clone.recv().await {
                    let mut lock = counter_clone.lock();
                    *lock += 1;
                    if *lock == expected_total {
                        break;
                    }
                }
            }));
        }

        for handle in producer_handles {
            assert!(handle.await.is_ok(), "Producer task panicked");
        }

        let timeout_res = tokio::time::timeout(Duration::from_secs(5), async {
            for handle in consumer_handles {
                assert!(handle.await.is_ok(), "Consumer task panicked");
            }
        })
        .await;

        assert!(
            timeout_res.is_ok(),
            "Test timed out: Consumers did not process all messages"
        );

        let final_count = *counter.lock();
        assert_eq!(
            final_count, expected_total,
            "Not all messages were received"
        );
    }

    #[tokio::test]
    async fn test_cancellation_safety_with_select() {
        let (tx, rx) = unbounded::<i32>();

        let res = tokio::select! {
            _ = rx.recv() => false,
            _ = tokio::time::sleep(Duration::from_millis(50)) => true,
        };

        assert!(res, "Timeout should have won");

        let state = rx.shared.state.lock();
        assert_eq!(
            state.wakers.len(),
            0,
            "Dangling waker detected after future was dropped by select!"
        );

        drop(state);

        assert!(
            tx.send_async(42).await.is_ok(),
            "Send failed after cancellation"
        );
        assert!(
            matches!(rx.recv().await, Ok(42)),
            "Failed to receive message after cancellation"
        );
    }

    #[tokio::test]
    async fn test_disconnect_wakes_pending_receivers() {
        let (tx, rx) = unbounded::<i32>();

        let rx_clone1 = rx.clone();
        let handle1 = tokio::spawn(async move { rx_clone1.recv().await });

        let rx_clone2 = rx.clone();
        let handle2 = tokio::spawn(async move { rx_clone2.recv().await });

        tokio::time::sleep(Duration::from_millis(50)).await;

        drop(tx);

        assert!(
            matches!(
                tokio::time::timeout(Duration::from_secs(1), handle1).await,
                Ok(Ok(Err(_)))
            ),
            "Task 1 should have returned RecvError without deadlocking"
        );

        assert!(
            matches!(
                tokio::time::timeout(Duration::from_secs(1), handle2).await,
                Ok(Ok(Err(_)))
            ),
            "Task 2 should have returned RecvError without deadlocking"
        );
    }
    #[tokio::test]
    async fn test_buffered_messages_after_sender_drops() {
        let (tx, rx) = unbounded::<i32>();

        assert!(
            tx.send_async(1).await.is_ok(),
            "Failed to send first message"
        );
        assert!(
            tx.send_async(2).await.is_ok(),
            "Failed to send second message"
        );

        drop(tx);

        assert!(
            matches!(rx.recv().await, Ok(1)),
            "Failed to read first buffered message"
        );
        assert!(
            matches!(rx.recv().await, Ok(2)),
            "Failed to read second buffered message"
        );

        assert!(
            matches!(rx.recv().await, Err(RecvError)),
            "Channel should be closed after buffer is drained"
        );
    }
    #[tokio::test]
    async fn test_send_fails_when_all_receivers_dropped() {
        let (tx, rx) = unbounded::<i32>();

        assert!(
            tx.send_async(1).await.is_ok(),
            "Should send successfully when receiver exists"
        );

        drop(rx);

        assert!(
            matches!(tx.send_async(2).await, Err(SendError(2))),
            "Should return SendError with the message when no receivers are alive"
        );
    }
    #[tokio::test]
    async fn test_multiple_receivers_wake_cascade() {
        let (tx, rx) = unbounded::<i32>();

        let mut handles = vec![];
        for _ in 0..3 {
            let rx_clone = rx.clone();
            handles.push(tokio::spawn(async move { rx_clone.recv().await }));
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(tx.send_async(10).await.is_ok(), "Failed to send 10");
        assert!(tx.send_async(20).await.is_ok(), "Failed to send 20");
        assert!(tx.send_async(30).await.is_ok(), "Failed to send 30");

        let mut received_values = vec![];

        for handle in handles {
            let timeout_res = tokio::time::timeout(Duration::from_secs(1), handle).await;

            assert!(timeout_res.is_ok(), "Receiver task timed out");

            if let Ok(Ok(Ok(val))) = timeout_res {
                received_values.push(val);
            } else {
                assert!(false, "Receiver task failed or returned an error");
            }
        }

        received_values.sort();
        assert_eq!(
            received_values,
            vec![10, 20, 30],
            "Messages were not correctly distributed among receivers"
        );
    }

    #[tokio::test]
    async fn test_cloned_senders_keep_channel_open() {
        let (tx1, rx) = unbounded::<i32>();
        let tx2 = tx1.clone();

        drop(tx1);

        let handle = tokio::spawn(async move { rx.recv().await });

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(
            tx2.send_async(99).await.is_ok(),
            "Send should succeed because tx2 is still alive"
        );

        let timeout_res = tokio::time::timeout(Duration::from_secs(1), handle).await;
        assert!(
            matches!(timeout_res, Ok(Ok(Ok(99)))),
            "Receiver failed to get the message from cloned sender"
        );
    }
}
