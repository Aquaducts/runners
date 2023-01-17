#![allow(improper_ctypes)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::unreadable_literal)]

pub mod lxc {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub mod backends;
pub mod config;

use crate::config::CONFIG;
use anyhow::Result;
use common::websocket::WebsocketMessage;
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    sync::Mutex,
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// Only used for the actual websocket connection. Otherwise, this struct is useless.
pub struct WebsocketInner {
    pub reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    pub writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,

    pub outer_writer: UnboundedSender<WebsocketMessage>,
    pub outer_reader: Arc<Mutex<UnboundedReceiver<WebsocketMessage>>>,
}

impl WebsocketInner {
    pub async fn listen(&self) -> JoinHandle<()> {
        let outer_writer = self.outer_writer.clone();
        let reader = self.reader.clone();
        tokio::spawn(async move {
            let mut locked_reader = reader.lock().await;
            while let Some(msg) = locked_reader.next().await {
                let msg = msg.unwrap();
                if msg.is_text() || msg.is_binary() {
                    if let Ok(message) =
                        serde_json::from_str::<WebsocketMessage>(&msg.into_text().unwrap())
                    {
                        outer_writer.send(message).unwrap();
                    }
                }
            }
        })
    }
}

pub struct Websocket {
    _inner: WebsocketInner,
    pub reader: Arc<Mutex<UnboundedReceiver<WebsocketMessage>>>,
    pub writer: UnboundedSender<WebsocketMessage>,
}

impl Websocket {
    pub async fn new() -> Result<Self> {
        let (stream, _) = connect_async(format!(
            "ws://{}:{}/api/ws?name=runner1&password=runner1234",
            CONFIG.server.host, CONFIG.server.port
        ))
        .await?;

        let (writer, reader) = stream.split();

        let (writer_writer, writer_reader) = unbounded_channel::<WebsocketMessage>();
        let (reader_writer, reader_reader) = unbounded_channel::<WebsocketMessage>();

        let inner_websocket_struct = WebsocketInner {
            reader: Arc::new(Mutex::new(reader)),
            writer,
            outer_reader: Arc::new(Mutex::new(reader_reader)),
            outer_writer: writer_writer,
        };
        Ok(Self {
            _inner: inner_websocket_struct,
            reader: Arc::new(Mutex::new(writer_reader)),
            writer: reader_writer,
        })
    }
}
