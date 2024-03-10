use std::net::{SocketAddr};

use anyhow::Context;
use bytes::BytesMut;
use futures::prelude::*;
use tokio::{net::TcpStream, runtime::Builder, sync::mpsc};
use tokio_websockets::{MaybeTlsStream, WebSocketStream, Message};

type Conn = WebSocketStream<TcpStream>;

pub enum NetCmdMessage {
    Connect(SocketAddr),
    MigrateConnect(SocketAddr),
    Disconnect,
    SendMsg(bytes::Bytes),
}

pub enum NetRxMessage {
    Connected,
    Disconnected,
    MsgReceived(tokio_websockets::Message),
}

pub struct NetClientHandle {
    tx: mpsc::Sender<NetCmdMessage>,
    rx: mpsc::Receiver<NetRxMessage>,
    buf: BytesMut,
}

impl NetClientHandle {
    pub fn new(tx: mpsc::Sender<NetCmdMessage>, rx: mpsc::Receiver<NetRxMessage>) -> Self {
        Self {
            tx,
            rx,
            buf: BytesMut::new(),
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) {
        self.tx.try_send(NetCmdMessage::Connect(addr)).unwrap();
    }

    pub fn disconnect(&mut self) {
        self.tx.try_send(NetCmdMessage::Disconnect).unwrap();
    }

    pub fn send_msg(&mut self, msg: bytes::Bytes) {
        self.tx.try_send(NetCmdMessage::SendMsg(msg)).unwrap();
    }

    pub fn poll(&mut self) -> Option<NetRxMessage> {
        self.rx.try_recv().ok()
    }
}

pub struct NetClient {
    rx: mpsc::Receiver<NetCmdMessage>,
    tx: mpsc::Sender<NetRxMessage>,
}

impl NetClient {
    pub fn new() -> (Self, NetClientHandle) {
        let (tx, rx) = mpsc::channel(32);
        let (tx2, rx2) = mpsc::channel(32);
        let handle = NetClientHandle::new(tx, rx2);
        let client = Self { rx, tx: tx2 };
        (client, handle)
    }

    async fn connect(&mut self, addr: SocketAddr) -> anyhow::Result<Conn> {
        let stream = TcpStream::connect(addr).await?;
        let (stream, resp) = tokio_websockets::ClientBuilder::new()
            .uri("http://127.0.0.1").unwrap()
            .connect_on(stream)
            .await?;


        Ok(stream)
    }

    async fn handle_cmd(&mut self, cmd: NetCmdMessage) -> anyhow::Result<()> {
        Ok(())
    }

    async fn run_socket(
        &mut self,
        mut conn: Conn,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                cmd = self.rx.recv() => {
                    let cmd = cmd.context("closed")?;
                    match cmd {
                        NetCmdMessage::Connect(addr) => {
                            panic!("already connected");
                        }
                        NetCmdMessage::MigrateConnect(addr) => {
                            conn = self.connect(addr).await?;
                        }
                        NetCmdMessage::Disconnect => {
                            conn.close().await?;
                            break;
                        }
                        NetCmdMessage::SendMsg(msg) => {
                            socket.send(Message::Binary(msg)).await?;
                        }
                    }
                }
                msg = socket.next() => {
                    let msg = msg.context("closed")??;
                    match msg {
                        Message::
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let mut socket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>> = None;

        loop {
            if let Some(ref mut socket) = socket.take() {
                loop {
                    tokio::select! {
                        // This one should be never closed
                        cmd = self.rx.recv() => self.handle_cmd(cmd.expect("NetClient rx closed")).await?,
                        msg = socket.next() => {
                            if let Some(msg) = msg {
                                self.handle_msg(msg?).await?;
                            } else {
                                // TODO post disconnected
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn net_executor() {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    rt.block_on(async move {})
}
