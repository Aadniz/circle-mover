use anyhow::Result;
use cm_protos::{create_goodbye, create_hello, create_message, serialize_message};
use quinn::{RecvStream, SendStream};
use tokio::sync::mpsc;

use crate::util::network::connect;

enum NetworkActorMessage {
    SendHello,
    SendGoodbye,
}

struct NetworkActor {
    receiver: mpsc::Receiver<NetworkActorMessage>,
    connection: (SendStream, RecvStream),
}

impl NetworkActor {
    async fn init(receiver: mpsc::Receiver<NetworkActorMessage>) -> Result<Self> {
        let connection = connect().await?;
        Ok(NetworkActor {
            receiver,
            connection,
        })
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    async fn handle_message(&mut self, msg: NetworkActorMessage) {
        match msg {
            NetworkActorMessage::SendHello => self.send_hello().await,
            NetworkActorMessage::SendGoodbye => self.send_goodbye().await,
        }
    }

    async fn send_hello(&mut self) {
        let (ref mut send, _) = self.connection;
        let msg = create_message(
            cm_protos::cm_proto::messages::circle_mover_message::Value::Hello(create_hello(
                "world".to_string(),
            )),
        );
        let bytes = serialize_message(msg);

        send.write_all(&bytes).await.expect("Failed to write");
    }

    async fn send_goodbye(&mut self) {
        let (ref mut send, _) = self.connection;
        let msg = create_message(
            cm_protos::cm_proto::messages::circle_mover_message::Value::Goodbye(create_goodbye(
                "world".to_string(),
            )),
        );
        let bytes = serialize_message(msg);

        send.write_all(&bytes).await.expect("Failed to write");
    }
}

#[derive(Clone)]
pub struct NetworkActorHandle {
    sender: mpsc::Sender<NetworkActorMessage>,
}

impl NetworkActorHandle {
    pub fn new() -> Self {
        // Arbitrary channel size, look into this, handling back pressure etc
        let (sender, receiver) = mpsc::channel(256);
        tokio::spawn(async move {
            let mut actor = NetworkActor::init(receiver)
                .await
                .expect("NetworkHandle failed to init");
            actor.run().await;
        });

        Self { sender }
    }

    pub fn send_hello(&self) {
        let msg = NetworkActorMessage::SendHello;
        self.sender.try_send(msg).expect("Failed to send hello");
    }

    pub fn send_goodbye(&self) {
        let msg = NetworkActorMessage::SendGoodbye;
        self.sender.try_send(msg).expect("Failed to send goodbye");
    }
}