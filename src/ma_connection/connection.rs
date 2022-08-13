use futures_channel::mpsc::UnboundedSender;
use futures_util::stream::SplitStream;
use tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream, connect_async};
use tokio::net::TcpStream;
use url::Url;
use std::error::Error;
use futures_util::StreamExt;


pub struct Connection {
    pub tx: UnboundedSender<Message>,
    pub rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl Connection {
    pub async fn new(url: Url) -> Result<Connection, Box<dyn Error>> {
        println!("Connecting to {} ...", url);
        let (tx_pipe_in, tx_pipe_out) = futures_channel::mpsc::unbounded::<Message>();

        let (ws_stream, _) = connect_async(url).await?;
        println!("Websocket handshake has been successfully completed");

        let (socket_tx, socket_rx) = ws_stream.split();

        tokio::spawn(tx_pipe_out.map(Ok).forward(socket_tx));

        Ok(Connection {
            tx: tx_pipe_in,
            rx: socket_rx,
        })
    }
}
