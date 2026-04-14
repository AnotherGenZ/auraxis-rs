use super::Message as CensusMessage;
use crate::AuraxisError;
use crate::realtime::{Action, Event, REALTIME_URL, SubscriptionSettings};
use std::io;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Future, Sink, SinkExt, Stream, StreamExt};
use metrics::{counter, describe_counter};
use stream_reconnect::{ReconnectStream, UnderlyingStream};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender};
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::error::Error as WsError;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, error, info, warn};

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
    state: Arc<RwLock<RealtimeClientState>>,
}

#[derive(Debug, Clone)]
struct RealtimeClientState {
    subscription_config: SubscriptionSettings,
    ws_send: Option<UnboundedSender<Message>>,
}

struct WebSocket(WebSocketStream<MaybeTlsStream<TcpStream>>);

type ReconnectWs = ReconnectStream<WebSocket, String, Result<Message, WsError>, WsError>;

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
            state: Arc::new(RwLock::new(RealtimeClientState {
                subscription_config: SubscriptionSettings::empty(),
                ws_send: None,
            })),
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
        if self.current_ws_sender().is_some() {
            return Err(anyhow::anyhow!("RealtimeClient is already connected").into());
        }

        let census_addr = format!(
            "{}?environment={}&service-id=s:{}",
            self.config.realtime_url.as_deref().unwrap_or(REALTIME_URL),
            self.config.environment,
            self.config.service_id
        );

        let websocket = ReconnectWs::connect(census_addr).await?;

        let (ws_send, ws_recv) = websocket.split();
        let (ws_send_tx, ws_send_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
        let (event_stream_tx, event_stream_rx) = tokio::sync::mpsc::channel::<Event>(1000);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        self.set_ws_sender(Some(ws_send_tx.clone()));

        tokio::spawn(Self::send_ws(ws_send, ws_send_rx, shutdown_rx.clone()));
        tokio::spawn(Self::ping_ws(ws_send_tx.clone(), shutdown_rx.clone()));
        tokio::spawn(Self::resubscribe(
            self.clone(),
            ws_send_tx.clone(),
            shutdown_rx.clone(),
        ));
        tokio::spawn(Self::read_ws(
            self.clone(),
            ws_send_tx,
            ws_recv,
            event_stream_tx,
            shutdown_tx,
            shutdown_rx,
        ));

        Ok(event_stream_rx)
    }

    pub fn subscribe(&mut self, subscription: SubscriptionSettings) {
        let ws_send = {
            let mut state = self.state.write().expect("realtime client state poisoned");
            state.subscription_config.merge(subscription);
            state.ws_send.clone()
        };

        let subscribe_message = match self.subscribe_message() {
            Ok(Some(message)) => message,
            Ok(None) => return,
            Err(err) => {
                error!("Failed to serialize subscription update: {err}");
                return;
            }
        };

        if let Some(ws_send) = ws_send
            && let Err(err) = ws_send.send(subscribe_message)
        {
            warn!("Failed to enqueue live subscription update: {err}");
            self.set_ws_sender(None);
        }
    }

    pub fn clear_subscribe(&mut self, subscription: SubscriptionSettings) {
        let (ws_send, current_subscription) = {
            let mut state = self.state.write().expect("realtime client state poisoned");
            state.subscription_config.clear(&subscription);
            (state.ws_send.clone(), state.subscription_config.clone())
        };

        let clear_message = match Self::clear_subscribe_message(&subscription) {
            Ok(message) => message,
            Err(err) => {
                error!("Failed to serialize clear subscription update: {err}");
                return;
            }
        };

        if let Some(ws_send) = ws_send {
            if let Err(err) = ws_send.send(clear_message) {
                warn!("Failed to enqueue clear subscription update: {err}");
                self.set_ws_sender(None);
                return;
            }

            if subscription.logical_and_characters_with_worlds.is_some()
                && !current_subscription.is_empty()
            {
                match serde_json::to_string(&Action::Subscribe(current_subscription))
                    .map(|message| Message::Text(message.into()))
                {
                    Ok(message) => {
                        if let Err(err) = ws_send.send(message) {
                            warn!("Failed to enqueue resubscribe after logical-and update: {err}");
                            self.set_ws_sender(None);
                        }
                    }
                    Err(err) => {
                        error!("Failed to serialize resubscribe after logical-and update: {err}");
                    }
                }
            }
        }
    }

    pub fn clear_all_subscriptions(&mut self) {
        let ws_send = {
            let mut state = self.state.write().expect("realtime client state poisoned");
            state.subscription_config = SubscriptionSettings::empty();
            state.ws_send.clone()
        };

        if let Some(ws_send) = ws_send {
            match Self::clear_all_subscribe_message() {
                Ok(message) => {
                    if let Err(err) = ws_send.send(message) {
                        warn!("Failed to enqueue clear-all subscription update: {err}");
                        self.set_ws_sender(None);
                    }
                }
                Err(err) => {
                    error!("Failed to serialize clear-all subscription update: {err}");
                }
            }
        }
    }

    async fn resubscribe(
        self,
        ws_send: UnboundedSender<Message>,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<(), AuraxisError> {
        loop {
            if *shutdown.borrow() {
                return Ok(());
            }

            let Some(message) = self.subscribe_message()? else {
                tokio::select! {
                    _ = shutdown.changed() => return Ok(()),
                    _ = tokio::time::sleep(Duration::from_secs(60 * 30)) => {}
                }
                continue;
            };

            let res = ws_send.send(message);

            match res {
                Ok(_) => {
                    counter!("realtime_total_resubscriptions").increment(1);
                    tokio::select! {
                        _ = shutdown.changed() => return Ok(()),
                        _ = tokio::time::sleep(Duration::from_secs(60 * 30)) => {}
                    }
                }
                Err(err) => {
                    warn!("Subscription loop shutting down: {}", err);
                    return Ok(());
                }
            }
        }
    }

    async fn ping_ws(
        ping_send: UnboundedSender<Message>,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<(), AuraxisError> {
        loop {
            match ping_send.send(Message::Ping(b"Hello".to_vec().into())) {
                Ok(_) => {
                    counter!("realtime_total_pings").increment(1);
                }
                Err(err) => {
                    warn!("Ping loop shutting down: {}", err);
                    counter!("realtime_total_ping_errors").increment(1);
                    return Ok(());
                }
            }

            tokio::select! {
                _ = shutdown.changed() => return Ok(()),
                _ = tokio::time::sleep(Duration::from_secs(1)) => {}
            }
        }
    }

    async fn send_ws(
        mut ws_send: SplitSink<ReconnectWs, Message>,
        mut ws_send_rx: UnboundedReceiver<Message>,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<(), AuraxisError> {
        loop {
            let message = tokio::select! {
                _ = shutdown.changed() => break,
                message = ws_send_rx.recv() => message,
            };

            let Some(msg) = message else {
                break;
            };

            // debug!("Sent: {:?}", msg.to_string());
            if let Err(err) = ws_send.send(msg).await {
                warn!("Send loop shutting down: {err}");
                return Err(err.into());
            }
            counter!("realtime_messages_total_sent").increment(1);
        }

        Ok(())
    }

    async fn read_ws(
        self,
        ws_send: UnboundedSender<Message>,
        mut ws_recv: SplitStream<ReconnectWs>,
        event_stream_tx: Sender<Event>,
        shutdown_tx: watch::Sender<bool>,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<(), AuraxisError> {
        loop {
            let message = tokio::select! {
                _ = shutdown.changed() => break,
                message = ws_recv.next() => message,
            };

            let Some(message) = message else {
                break;
            };

            counter!("realtime_messages_received_total").increment(1);
            match message {
                Ok(msg) => {
                    // debug!("Received: {:?}", msg.to_string());
                    if let Err(err) = Self::handle_ws_msg(
                        self.clone(),
                        ws_send.clone(),
                        event_stream_tx.clone(),
                        shutdown_tx.clone(),
                        msg,
                    )
                    .await
                    {
                        counter!("realtime_messages_received_total_errored").increment(1);
                        error!("{:?}", err);
                    }
                }
                Err(err) => {
                    //println!("{:?}", &err);
                    counter!("realtime_messages_received_total_errored").increment(1);

                    match err {
                        WsError::ConnectionClosed => {
                            error!("Connection closed");
                            counter!("realtime_total_closed_connections").increment(1);
                            break;
                        }
                        WsError::AlreadyClosed
                        | WsError::Io(_)
                        | WsError::Tls(_)
                        | WsError::Capacity(_)
                        | WsError::Protocol(_)
                        | WsError::WriteBufferFull(_)
                        | WsError::Utf8(_)
                        | WsError::Url(_)
                        | WsError::Http(_)
                        | WsError::HttpFormat(_)
                        | WsError::AttackAttempt => {}
                    }
                }
            }
        }

        self.set_ws_sender(None);
        signal_shutdown(&shutdown_tx);

        Ok(())
    }

    async fn handle_ws_msg(
        self,
        ws_send: UnboundedSender<Message>,
        events: Sender<Event>,
        shutdown: watch::Sender<bool>,
        msg: Message,
    ) -> Result<(), AuraxisError> {
        match msg {
            Message::Text(text) => {
                let message: CensusMessage = serde_json::from_str(&text)?;

                match message {
                    CensusMessage::ConnectionStateChanged { connected } => {
                        if connected {
                            info!("Connected to Census!");

                            counter!("realtime_total_connections").increment(1);

                            let Some(subscription_message) = self.subscribe_message()? else {
                                return Ok(());
                            };
                            debug!("Subscribing with {:?}", subscription_message);

                            if let Err(err) = ws_send.send(subscription_message) {
                                signal_shutdown(&shutdown);
                                debug!(
                                    "Subscription send aborted because ws channel closed: {err}"
                                );
                            }
                        }
                    }
                    CensusMessage::Heartbeat { .. } => {
                        counter!("realtime_messages_received_heartbeat").increment(1);
                    }
                    CensusMessage::ServiceStateChanged { .. } => {}
                    CensusMessage::ServiceMessage { payload } => {
                        if events.send(payload).await.is_err() {
                            debug!("Dropping realtime event because consumer channel is closed");
                            signal_shutdown(&shutdown);
                            return Ok(());
                        }
                    }
                    CensusMessage::Subscription { subscription } => {
                        debug!("Subscribed: {:?}", subscription);
                    }
                }
            }
            Message::Binary(_) | Message::Pong(_) | Message::Frame(_) => {}
            Message::Ping(ping) => {
                if let Err(err) = ws_send.send(Message::Pong(ping)) {
                    signal_shutdown(&shutdown);
                    debug!("Pong send aborted because ws channel closed: {err}");
                }
            }
            Message::Close(close) => {
                counter!("realtime_total_closed_connections").increment(1);
                if let Some(close_frame) = close {
                    error!(
                        "Websocket closed. Code: {}, Reason: {}",
                        close_frame.code, close_frame.reason
                    );
                }
                warn!("Websocket close frame received; waiting for reconnect");
            }
        }

        Ok(())
    }

    fn subscribe_message(&self) -> Result<Option<Message>, AuraxisError> {
        let subscription = self.current_subscription();
        if subscription.is_empty() {
            return Ok(None);
        }

        Ok(Some(Message::Text(
            serde_json::to_string(&Action::Subscribe(subscription))?.into(),
        )))
    }

    fn clear_subscribe_message(
        subscription: &SubscriptionSettings,
    ) -> Result<Message, AuraxisError> {
        Ok(Message::Text(
            serde_json::to_string(&Action::ClearSubscribe {
                all: None,
                event_names: subscription.event_names.clone(),
                characters: subscription.characters.clone(),
                worlds: subscription.worlds.clone(),
                service: subscription.service.clone(),
            })?
            .into(),
        ))
    }

    fn clear_all_subscribe_message() -> Result<Message, AuraxisError> {
        Ok(Message::Text(
            serde_json::to_string(&Action::ClearSubscribe {
                all: Some(true),
                event_names: None,
                characters: None,
                worlds: None,
                service: crate::realtime::Service::Event,
            })?
            .into(),
        ))
    }

    fn current_subscription(&self) -> SubscriptionSettings {
        self.state
            .read()
            .expect("realtime client state poisoned")
            .subscription_config
            .clone()
    }

    fn current_ws_sender(&self) -> Option<UnboundedSender<Message>> {
        self.state
            .read()
            .expect("realtime client state poisoned")
            .ws_send
            .clone()
    }

    fn set_ws_sender(&self, ws_send: Option<UnboundedSender<Message>>) {
        self.state
            .write()
            .expect("realtime client state poisoned")
            .ws_send = ws_send;
    }
}

fn signal_shutdown(shutdown: &watch::Sender<bool>) {
    let _ = shutdown.send(true);
}

impl Stream for WebSocket {
    type Item = Result<Message, WsError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}

impl Sink<Message> for WebSocket {
    type Error = WsError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        Pin::new(&mut self.0).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

impl UnderlyingStream<String, Result<Message, WsError>, WsError> for WebSocket {
    // Establishes connection.
    // Additionally, this will be used when reconnect tries are attempted.
    fn establish(addr: String) -> Pin<Box<dyn Future<Output = Result<Self, WsError>> + Send>> {
        Box::pin(async move {
            // In this case, we are trying to connect to the WebSocket endpoint
            let (websocket, _) = connect_async(addr).await?;
            Ok(WebSocket(websocket))
        })
    }

    // The following errors are considered disconnect errors.
    fn is_write_disconnect_error(&self, err: &WsError) -> bool {
        matches!(
            err,
            WsError::ConnectionClosed
                | WsError::AlreadyClosed
                | WsError::Io(_)
                | WsError::Tls(_)
                | WsError::Protocol(_)
        )
    }

    // If an `Err` is read, then there might be an disconnection.
    fn is_read_disconnect_error(&self, item: &Result<Message, WsError>) -> bool {
        if let Err(e) = item {
            self.is_write_disconnect_error(e)
        } else {
            false
        }
    }

    // Return "Exhausted" if all retry attempts are failed.
    fn exhaust_err() -> WsError {
        WsError::Io(io::Error::other("Exhausted"))
    }
}
