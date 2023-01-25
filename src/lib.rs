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
pub mod docker;
pub mod runner;

use crate::config::CONFIG;
use anyhow::Result;
use common::websocket::{OpCodes, WebsocketMessage};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    sync::Mutex,
    task::JoinHandle,
    time::Duration,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// Only used for the actual websocket connection. Otherwise, this struct is useless.
pub struct WebsocketInner {
    pub reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    pub writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,

    pub outer_writer: UnboundedSender<WebsocketMessage>,
    pub outer_reader: Arc<Mutex<UnboundedReceiver<WebsocketMessage>>>,
}

impl WebsocketInner {
    pub async fn start_listener(&self) -> JoinHandle<()> {
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

    pub async fn start_writer(&self) -> JoinHandle<()> {
        let outer_reader = self.outer_reader.clone();
        let writer = self.writer.clone();
        tokio::spawn(async move {
            let mut writer = writer.lock().await;
            let mut locked_reader = outer_reader.lock().await;

            while let Some(to_write) = locked_reader.recv().await {
                writer
                    .send(Message::Text(serde_json::to_string(&to_write).unwrap()))
                    .await
                    .unwrap();
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
            writer: Arc::new(Mutex::new(writer)),
            outer_reader: Arc::new(Mutex::new(reader_reader)),
            outer_writer: writer_writer,
        };
        Ok(Self {
            _inner: inner_websocket_struct,
            reader: Arc::new(Mutex::new(writer_reader)),
            writer: reader_writer,
        })
    }

    pub async fn start_heartbeating(&self) -> JoinHandle<()> {
        let writer = self.writer.clone();
        tokio::spawn(async move {
            loop {
                writer
                    .send(WebsocketMessage {
                        op: OpCodes::HeartBeatAck,
                        event: None,
                    })
                    .unwrap();
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        })
    }

    pub fn send(&self, message: WebsocketMessage) -> Result<()> {
        Ok(self.writer.send(message)?)
    }

    pub async fn start(&self) -> (JoinHandle<()>, JoinHandle<()>, JoinHandle<()>) {
        (
            self._inner.start_listener().await,
            self._inner.start_writer().await,
            self.start_heartbeating().await,
        )
    }
}
