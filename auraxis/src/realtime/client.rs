use super::Message as CensusMessage;
use crate::realtime::{Action, Event, SubscriptionSettings, REALTIME_URL};
use crate::AuraxisError;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use std::time::Duration;

use futures::TryFutureExt;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Future, SinkExt, StreamExt};
use metrics::{describe_counter, increment_counter};
use stream_reconnect::{ReconnectStream, UnderlyingStream};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::tungstenite::handshake::client::Response;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct RealtimeClientConfig {
    pub environment: String,
    pub service_id: String,
    pub realtime_url: Option<String>,
}

impl Default for RealtimeClientConfig {
    fn default() -> Self {
        Self {
            environment: String::from("ps2"),
            service_id: String::new(),
            realtime_url: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RealtimeClient {
    config: Arc<RealtimeClientConfig>,
    ws_send: Option<Sender<Message>>,
    subscription_config: Arc<SubscriptionSettings>,
}

struct WebSocket(WebSocketStream<MaybeTlsStream<TcpStream>>, Response);

type ReconnectingWS = ReconnectStream<WebSocket, String, Result<Message, Error>, Error>;

impl RealtimeClient {
    #[must_use]
    pub fn new(config: RealtimeClientConfig) -> Self {
        describe_counter!(
            "realtime_messages_total_sent",
            "Total number of messages sent to Census stream"
        );
        describe_counter!(
            "realtime_messages_received_total",
            "Total number of messages received from Census stream"
        );
        describe_counter!(
            "realtime_messages_received_total_errored",
            "Total number of messages received from Census stream that errored"
        );
        describe_counter!(
            "realtime_total_closed_connections",
            "Total number of closed connections to Census stream"
        );
        describe_counter!(
            "realtime_total_connections",
            "Total number of connections to Census stream"
        );
        describe_counter!(
            "realtime_messages_received_heartbeat",
            "Total number of heartbeat messages received from Census stream"
        );
        describe_counter!(
            "realtime_total_pings",
            "Total number of ping messages sent to Census stream, may include errors"
        );
        describe_counter!(
            "realtime_total_ping_errors",
            "Total number of ping messages that failed to receive a response from Census stream"
        );
        describe_counter!(
            "realtime_total_resubscriptions",
            "Total number of resubscriptions to Census stream"
        );

        Self {
            config: Arc::new(config),
            ws_send: None,
            subscription_config: Arc::new(SubscriptionSettings::default()),
        }
    }

    /// Send a message to the websocket connection.
    ///
    /// This function will be spawned as a task and will run concurrently to the
    /// rest of the application. It will continually check for messages on the
    /// receiver end of the channel. When a message is received, it will be sent to
    /// the websocket connection. If sending the message fails, the error is logged
    /// and the connection is closed.
    ///
    /// # Arguments
    ///
    /// * `websocket` - The websocket connection to send messages to.
    /// * `receiver` - The channel receiving messages to send.
    ///
    /// # Errors
    ///
    /// This function will return an error if the websocket connection cannot be created.
    pub async fn connect(&mut self) -> Result<Receiver<Event>, AuraxisError> {
        let census_addr = format!(
            "{}?environment={}&service-id=s:{}",
            self.config.realtime_url.as_deref().unwrap_or(REALTIME_URL),
            self.config.environment,
            self.config.service_id
        );

        let (websocket, _res) = connect_async(census_addr).await?;

        let (ws_send, ws_recv) = websocket.split();
        let (ws_send_tx, ws_send_rx) = tokio::sync::mpsc::channel::<Message>(1000);
        let (event_stream_tx, event_stream_rx) = tokio::sync::mpsc::channel::<Event>(1000);

        self.ws_send = Some(ws_send_tx.clone());

        tokio::spawn(Self::send_ws(ws_send, ws_send_rx));
        tokio::spawn(Self::ping_ws(ws_send_tx.clone()));
        tokio::spawn(Self::resubscribe(self.clone(), ws_send_tx.clone()));
        tokio::spawn(Self::read_ws(
            self.clone(),
            ws_send_tx,
            ws_recv,
            event_stream_tx,
        ));

        Ok(event_stream_rx)
    }

    pub fn subscribe(&mut self, subscription: SubscriptionSettings) {
        self.subscription_config = Arc::new(subscription);
    }

    async fn resubscribe(self, ws_send: Sender<Message>) -> Result<(), AuraxisError> {
        loop {
            let res = ws_send
                .send(Message::Text(serde_json::to_string(&Action::Subscribe(
                    (*self.subscription_config).clone(),
                ))?))
                .await;

            match res {
                Ok(_) => {
                    increment_counter!("realtime_total_resubscriptions");
                    tokio::time::sleep(Duration::from_secs(60 * 30)).await;
                }
                Err(err) => {
                    error!(
                        "Failed to send subscription message: {}. Retrying in 5 seconds",
                        err
                    );
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn ping_ws(ping_send: Sender<Message>) -> Result<(), AuraxisError> {
        let max_ping_fails = 5;
        let mut ping_fails = 0;
        loop {
            let send_result = ping_send.send(Message::Ping(b"Hello".to_vec())).await;

            match send_result {
                Ok(_) => {
                    ping_fails -= 1;
                    increment_counter!("realtime_total_pings");
                }
                Err(err) => {
                    error!("Failed to send ping message: {}", err);
                    increment_counter!("realtime_total_ping_errors");
                    ping_fails += 1;
                    if ping_fails > max_ping_fails {
                        panic!(
                            "Failed to send ping message: {max_ping_fails} times in a row. Exiting"
                        );
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn send_ws(
        mut ws_send: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        mut ws_send_rx: Receiver<Message>,
    ) -> Result<(), AuraxisError> {
        while let Some(msg) = ws_send_rx.recv().await {
            // debug!("Sent: {:?}", msg.to_string());
            ws_send.send(msg).await?;
            increment_counter!("realtime_messages_total_sent");
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
            increment_counter!("realtime_messages_received_total");
            match message {
                Ok(msg) => {
                    // debug!("Received: {:?}", msg.to_string());
                    tokio::spawn(
                        Self::handle_ws_msg(
                            self.clone(),
                            ws_send.clone(),
                            event_stream_tx.clone(),
                            msg,
                        )
                        .map_err(|err| {
                            increment_counter!("realtime_messages_received_total_errored");
                            error!("{:?}", err);
                        }),
                    );
                }
                Err(err) => {
                    //println!("{:?}", &err);
                    increment_counter!("realtime_messages_received_total_errored");

                    match err {
                        Error::ConnectionClosed => {
                            error!("Connection closed");
                            increment_counter!("realtime_total_closed_connections");
                            // TODO: Reconnect
                        }
                        Error::AlreadyClosed
                        | Error::Io(_)
                        | Error::Tls(_)
                        | Error::Capacity(_)
                        | Error::Protocol(_)
                        | Error::SendQueueFull(_)
                        | Error::Utf8
                        | Error::Url(_)
                        | Error::Http(_)
                        | Error::HttpFormat(_) => {}
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
                // info!("Received: {}", text);
                let message: CensusMessage = serde_json::from_str(&text)?;

                match message {
                    CensusMessage::ConnectionStateChanged { connected } => {
                        if connected {
                            info!("Connected to Census!");

                            increment_counter!("realtime_total_connections");

                            debug!(
                                "Subscribing with {:?}",
                                serde_json::to_string(&Action::Subscribe(
                                    (*self.subscription_config).clone()
                                ))?
                            );

                            ws_send
                                .send(Message::Text(serde_json::to_string(&Action::Subscribe(
                                    (*self.subscription_config).clone(),
                                ))?))
                                .await
                                .expect("WS send channel closed");
                        }
                    }
                    CensusMessage::Heartbeat { .. } => {
                        increment_counter!("realtime_messages_received_heartbeat");
                    }
                    CensusMessage::ServiceStateChanged { .. } => {}
                    CensusMessage::ServiceMessage { payload } => {
                        events
                            .send(payload)
                            .await
                            .expect("events send channel closed");
                    }
                    CensusMessage::Subscription { subscription } => {
                        debug!("Subscribed: {:?}", subscription);
                    }
                }
            }
            Message::Binary(_) | Message::Pong(_) | Message::Frame(_) => {}
            Message::Ping(ping) => {
                ws_send
                    .send(Message::Pong(ping))
                    .await
                    .expect("WS send channel closed");
            }
            Message::Close(close) => {
                increment_counter!("realtime.total_closed_connections");
                if let Some(close_frame) = close {
                    error!(
                        "Websocket closed. Code: {}, Reason: {}",
                        close_frame.code, close_frame.reason
                    );
                }
            }
        }

        Ok(())
    }
}

impl UnderlyingStream<String, Result<Message, Error>, Error> for WebSocket {
    // Establishes connection.
    // Additionally, this will be used when reconnect tries are attempted.
    fn establish(addr: String) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async move {
            // In this case, we are trying to connect to the WebSocket endpoint
            let (websocket, res) = connect_async(addr).await?;
            Ok(WebSocket(websocket, res))
        })
    }

    // The following errors are considered disconnect errors.
    fn is_write_disconnect_error(&self, err: &Error) -> bool {
        matches!(
            err,
            Error::ConnectionClosed
                | Error::AlreadyClosed
                | Error::Io(_)
                | Error::Tls(_)
                | Error::Protocol(_)
        )
    }

    // If an `Err` is read, then there might be an disconnection.
    fn is_read_disconnect_error(&self, item: &Result<Message, Error>) -> bool {
        if let Err(e) = item {
            self.is_write_disconnect_error(e)
        } else {
            false
        }
    }

    // Return "Exhausted" if all retry attempts are failed.
    fn exhaust_err() -> Error {
        Error::Io(io::Error::new(io::ErrorKind::Other, "Exhausted"))
    }
}
