use futures_channel::mpsc::UnboundedSender;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use std::error::Error;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

pub struct Connection {
    pub tx: UnboundedSender<Message>,
    pub rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,

    forward_thread: JoinHandle<Result<(), tokio_tungstenite::tungstenite::error::Error>>,
}

impl Connection {
    pub async fn new(url: &Url) -> Result<Connection, Box<dyn Error>> {
        let (tx_pipe_in, tx_pipe_out) = futures_channel::mpsc::unbounded::<Message>();
        let (ws_stream, _) = connect_async(url).await?;
        let (socket_tx, socket_rx) = ws_stream.split();
        let forward_thread = tokio::spawn(tx_pipe_out.map(Ok).forward(socket_tx));

        Ok(Connection {
            tx: tx_pipe_in,
            rx: socket_rx,
            forward_thread,
        })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.forward_thread.abort();
    }
}
