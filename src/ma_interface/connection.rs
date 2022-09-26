use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

pub struct Connection {
    pub tx: UnboundedSender<Message>,
    pub rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,

    forward_thread: JoinHandle<()>,
}

impl Connection {
    pub async fn new(url: &Url) -> Result<Connection, Box<dyn Error>> {
        let (tx_pipe_in, tx_pipe_out) = tokio::sync::mpsc::unbounded_channel();
        let (ws_stream, _) = connect_async(url).await?;
        let (socket_tx, socket_rx) = ws_stream.split();
        let forward_thread = tokio::spawn(forward_loop(socket_tx, tx_pipe_out));

        Ok(Connection {
            tx: tx_pipe_in,
            rx: socket_rx,
            forward_thread,
        })
    }
}

async fn forward_loop(mut socket_tx: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, mut tx_pipe_out: UnboundedReceiver<Message>) {
    loop {
        let message = tx_pipe_out.recv().await.unwrap();
        socket_tx.send(message).await.unwrap();
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.forward_thread.abort();
    }
}
