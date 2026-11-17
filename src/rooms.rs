use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::protocol::ServerMessage;

const MAX_CHANNEL_MESSAGES: usize = 10;

#[derive(Clone)]
struct Registry {
    shared_table: Arc<Mutex<HashMap<String, Sender<ServerMessage>>>>,
}

struct RoomGuard {
    shared_table: Arc<Mutex<HashMap<String, Sender<ServerMessage>>>>,
    receiver: Option<Receiver<ServerMessage>>,
    room: String,
}

#[derive(Debug)]
enum PublishError {
    RoomNotFound,
    NoSubscribers,
}

impl Drop for RoomGuard {
    fn drop(&mut self) {
        let mut lock_st = match self.shared_table.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let take_receiver = self.receiver.take();
        drop(take_receiver);

        let count_sender = match lock_st.get(&self.room) {
            Some(s) => s.receiver_count(),

            None => 0,
        };

        if count_sender == 0 {
            lock_st.remove(&self.room);
        }
    }
}

impl RoomGuard {
    async fn recv(&mut self) -> Result<ServerMessage, broadcast::error::RecvError> {
        self.receiver
            .as_mut()
            .expect("RoomGuard receiver missing")
            .recv()
            .await
    }

    fn try_recv(&mut self) -> Result<ServerMessage, broadcast::error::TryRecvError> {
        self.receiver
            .as_mut()
            .expect("RoomGuard receiver failed")
            .try_recv()
    }
}

impl Registry {
    fn new() -> Registry {
        Registry {
            shared_table: (Arc::new(Mutex::new(HashMap::new()))),
        }
    }

    fn join(&self, room: &str) -> RoomGuard {
        let mut lock_st = self.shared_table.lock().unwrap();

        let room_connect = lock_st.entry(room.to_string()).or_insert_with(|| {
            let (tx, _rx) = broadcast::channel::<ServerMessage>(MAX_CHANNEL_MESSAGES);
            tx
        });

        RoomGuard {
            shared_table: Arc::clone(&self.shared_table),
            receiver: Some(room_connect.subscribe()),
            room: room.to_string(),
        }
    }

    fn publish(&self, room: &str, mess: ServerMessage) -> Result<(), PublishError> {
        let lock_st = self.shared_table.lock().unwrap();

        match lock_st.get(room) {
            Some(s) => {
                let clone_sender = s.clone();

                drop(lock_st);

                match clone_sender.send(mess) {
                    Ok(_) => Ok(()),

                    Err(_) => Err(PublishError::NoSubscribers),
                }
            }
            None => Err(PublishError::RoomNotFound),
        }
    }

    #[cfg(test)]
    fn room_count(&self) -> usize {
        let lock_st = self.shared_table.lock().unwrap();

        lock_st.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Barrier;

    #[test]
    fn join_non_existant_room_creates_one() {
        let registry = Registry::new();

        let _recv1 = registry.join("room1");

        let room_count = registry.room_count();

        assert_eq!(room_count, 1);

        let _recv2 = registry.join("room1");

        let room_count = registry.room_count();

        assert_eq!(room_count, 1);
    }

    #[tokio::test]
    async fn two_subscriber_recv_same_mess() {
        let registry = Registry::new();

        let mut recv1 = registry.join("room1");
        let mut recv2 = registry.join("room1");

        let mess = ServerMessage::Message {
            room: "room1".into(),
            text: "hi room1".into(),
        };

        registry.publish("room1", mess.clone()).unwrap();

        let recv_mess1 = recv1.recv().await.unwrap();
        let recv_mess2 = recv2.recv().await.unwrap();

        assert_eq!(recv_mess1, mess);
        assert_eq!(recv_mess2, mess);
    }

    #[tokio::test]
    async fn room2_member_shouldnt_recv_room1_mess() {
        let registry = Registry::new();

        let mut recv1 = registry.join("room1");
        let mut recv2 = registry.join("room2");

        let mess = ServerMessage::Message {
            room: "room1".into(),
            text: "hi room1".into(),
        };

        registry.publish("room1", mess.clone()).unwrap();

        let recv_mess1 = recv1.recv().await.unwrap();
        let recv_mess2 = recv2.try_recv();

        assert_eq!(recv_mess1, mess);

        assert!(matches!(
            recv_mess2,
            Err(broadcast::error::TryRecvError::Empty)
        ));
    }

    #[test]
    fn member_leave_room_shouldnt_delete_it() {
        let registry = Registry::new();

        let recv1 = registry.join("room1");
        let _recv2 = registry.join("room1");

        drop(recv1);

        let room_count = registry.room_count();

        assert_eq!(room_count, 1);
    }

    #[test]
    fn last_member_leave_delete_room() {
        let registry = Registry::new();

        let recv1 = registry.join("room1");

        drop(recv1);

        let room_count = registry.room_count();

        assert_eq!(room_count, 0);
    }

    #[test]
    fn join_room_after_delete_recreate_it() {
        let registry = Registry::new();

        let recv1 = registry.join("room1");

        drop(recv1);

        let room_count = registry.room_count();

        assert_eq!(room_count, 0);

        let _recv2 = registry.join("room1");

        let room_count = registry.room_count();

        assert_eq!(room_count, 1);
    }

    #[test]
    fn publish_in_non_existant_room_return_err() {
        let registry = Registry::new();

        let mess = ServerMessage::Message {
            room: "room1".into(),
            text: "hi room1".into(),
        };

        let result = registry.publish("room1", mess);

        assert!(matches!(result, Err(PublishError::RoomNotFound)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_join_leave() {
        const N: usize = 100;

        let registry = Registry::new();
        let mut task_list = Vec::new();

        let barrier1 = Arc::new(Barrier::new(N + 1));
        let barrier2 = Arc::new(Barrier::new(N + 1));

        for _ in 0..N {
            let registry_clone = registry.clone();
            let barrier1_clone = Arc::clone(&barrier1);
            let barrier2_clone = Arc::clone(&barrier2);

            let t = tokio::spawn(async move {
                let _recv1 = registry_clone.join("room1");

                barrier1_clone.wait().await;
                barrier2_clone.wait().await;
            });
            task_list.push(t);
        }

        barrier1.wait().await;
        assert_eq!(registry.room_count(), 1);

        barrier2.wait().await;

        for task in task_list {
            task.await.unwrap();
        }
        assert_eq!(registry.room_count(), 0);
    }
}
