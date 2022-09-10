use std::fmt::Pointer;
use std::sync::Arc;
use super::Message as CensusMessage;
use crate::realtime::{
    Action, CharacterSubscription, Event, EventSubscription, Service, SubscriptionSettings,
    WorldSubscription, REALTIME_URL,
};
use crate::AuraxisError;

use std::time::Duration;

use futures::TryFutureExt;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

#[derive(Clone)]
pub struct RealtimeClientConfig {
    pub environment: String,
    pub service_id: String,
    pub realtime_url: Option<String>,
}

impl Default for RealtimeClientConfig {
    fn default() -> Self {
        Self {
            environment: String::from("ps2"),
            service_id: String::from(""),
            realtime_url: None
        }
    }
}

#[derive(Clone)]
pub struct RealtimeClient {
    config: Arc<RealtimeClientConfig>,
    ws_send: Option<Sender<Message>>,
    subscription_config: Arc<SubscriptionSettings>,
}

impl RealtimeClient {
    pub fn new(config: RealtimeClientConfig) -> Self {
        Self {
            config: Arc::new(config),
            ws_send: None,
            subscription_config: Arc::new(SubscriptionSettings::default()),
        }
    }

    pub async fn connect(&mut self) -> Result<Receiver<Event>, AuraxisError> {
        let census_addr = format!(
            "{}?environment={}&service-id=s:{}",
            self.config.realtime_url.as_deref().unwrap_or(REALTIME_URL), self.config.environment, self.config.service_id
        );

        let (websocket, _res) = connect_async(census_addr).await?;

        let (ws_send, ws_recv) = websocket.split();
        let (ws_send_tx, ws_send_rx) = tokio::sync::mpsc::channel::<Message>(1000);
        let (event_stream_tx, event_stream_rx) = tokio::sync::mpsc::channel::<Event>(1000);

        self.ws_send = Some(ws_send_tx.clone());

        tokio::spawn(Self::send_ws(ws_send, ws_send_rx));
        tokio::spawn(Self::ping_ws(ws_send_tx.clone()));
        tokio::spawn(Self::resubscribe(self.clone(), ws_send_tx.clone()));
        tokio::spawn(Self::read_ws( self.clone(), ws_send_tx, ws_recv, event_stream_tx));

        Ok(event_stream_rx)
    }

    pub fn subscribe(
        &mut self,
        events: Option<EventSubscription>,
        characters: Option<CharacterSubscription>,
        worlds: Option<WorldSubscription>,
    ) {
        self.subscription_config = Arc::new(SubscriptionSettings {
            event_names: events,
            characters,
            worlds,
            ..Default::default()
        })
    }

    async fn resubscribe(self, ws_send: Sender<Message>) -> Result<(), AuraxisError> {
        loop {
            ws_send
                .send(Message::Text(serde_json::to_string(&Action::Subscribe((*self.subscription_config).clone()))?))
                .await
                .expect("WS send channel closed");

            tokio::time::sleep(Duration::from_secs(60 * 30)).await;
        }
    }

    async fn ping_ws(ping_send: Sender<Message>) -> Result<(), AuraxisError> {
        loop {
            ping_send
                .send(Message::Ping("Hello".to_string().into_bytes()))
                .await
                .expect("WS send channel closed");

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn send_ws(
        mut ws_send: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        mut ws_send_rx: Receiver<Message>,
    ) -> Result<(), AuraxisError> {
        while let Some(msg) = ws_send_rx.recv().await {
            ws_send.send(msg).await?;
        }

        Ok(())
    }

    async fn read_ws(
        self,
        ws_send: Sender<Message>,
        mut ws_recv: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        event_stream_tx: Sender<Event>,
    ) -> Result<(), AuraxisError> {
        while let Some(message) = ws_recv.next().await {
            match message {
                Ok(msg) => {
                    tokio::spawn(
                        Self::handle_ws_msg(self.clone(), ws_send.clone(), event_stream_tx.clone(), msg).map_err(
                            |err| {
                                error!("{:?}", err);
                            },
                        ),
                    );
                }
                Err(err) => {
                    //println!("{:?}", &err);

                    match err {
                        Error::ConnectionClosed => {
                            error!("Connection closed");
                        }
                        Error::AlreadyClosed => {}
                        Error::Io(_) => {}
                        Error::Tls(_) => {}
                        Error::Capacity(_) => {}
                        Error::Protocol(_) => {}
                        Error::SendQueueFull(_) => {}
                        Error::Utf8 => {}
                        Error::Url(_) => {}
                        Error::Http(_) => {}
                        Error::HttpFormat(_) => {}
                    }
                }
            }
        }

        // Connection closed

        Ok(())
    }

    async fn handle_ws_msg(
        self,
        ws_send: Sender<Message>,
        events: Sender<Event>,
        msg: Message,
    ) -> Result<(), AuraxisError> {
        match msg {
            Message::Text(text) => {
                let message: CensusMessage = serde_json::from_str(&text)?;

                match message {
                    CensusMessage::ConnectionStateChanged { connected } => {
                        if connected {
                            info!("Connected to Census!");

                            ws_send
                                .send(Message::Text(serde_json::to_string(&Action::Subscribe((*self.subscription_config).clone()))?))
                                .await
                                .expect("WS send channel closed");
                        }
                    }
                    CensusMessage::Heartbeat { .. } => {}
                    CensusMessage::ServiceMessage { payload } => {
                        events
                            .send(payload)
                            .await
                            .expect("events send channel closed");
                    }
                    CensusMessage::ServiceStateChanged { .. } => {}
                    CensusMessage::Subscription { subscription } => {
                        print!("Subscribed: {:?}", subscription);
                    }
                }
            }
            Message::Binary(_) => {}
            Message::Ping(ping) => {
                ws_send
                    .send(Message::Pong(ping))
                    .await
                    .expect("WS send channel closed");
            }
            Message::Pong(_) => {}
            Message::Close(close) => {
                if let Some(close_frame) = close {
                    println!(
                        "Websocket closed. Code: {}, Reason: {}",
                        close_frame.code, close_frame.reason
                    );
                }
            }
            Message::Frame(_) => {}
        }

        Ok(())
    }
}
