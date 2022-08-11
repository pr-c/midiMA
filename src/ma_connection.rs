mod ma_requests;
mod ma_responses;
use futures_channel::mpsc::UnboundedSender;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use ma_requests::LoginRequest;
use ma_requests::SessionIdRequest;
use ma_responses::{ConnectResponse, LoginRequestResponse, SessionIdResponse};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

pub struct MaInterface {
    url: Url,
    server_username: String,
    server_password: String,
    session_id: i32,
}

impl MaInterface {
    pub fn new(
        server_ip: String,
        server_username: String,
        server_password: String,
    ) -> Result<MaInterface, Box<dyn Error>> {
        Ok(MaInterface {
            url: Url::parse(&format!("ws://{}", server_ip))?,
            server_username,
            server_password,
            session_id: 0,
        })
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Connecting to {} ...", self.url);
        let (tx_pipe_in, tx_pipe_out) = futures_channel::mpsc::unbounded::<Message>();

        let (ws_stream, _) = connect_async(&self.url).await?;
        println!("Websocket handshake has been successfully completed");

        let (socket_tx, mut socket_rx) = ws_stream.split();

        tokio::spawn(tx_pipe_out.map(Ok).forward(socket_tx));

        self.receive_response::<ConnectResponse>(&mut socket_rx)
            .await?;
        self.send_session_id_requst(&tx_pipe_in)?;
        let session_id_response = self
            .receive_response::<SessionIdResponse>(&mut socket_rx)
            .await?;
        self.session_id = session_id_response.session;
        println!("Session ID: {}", self.session_id);
        self.send_login_request(&tx_pipe_in)?;
        let login_response = self
            .receive_response::<LoginRequestResponse>(&mut socket_rx)
            .await?;
        match login_response.result {
            true => println!("Success"),
            false => println!("Failure"),
        }
        Ok(())
    }

    fn send_session_id_requst(&self, tx: &UnboundedSender<Message>) -> Result<(), Box<dyn Error>> {
        let request = SessionIdRequest { session: 0 };
        self.send_request(tx, &request)
    }

    fn send_login_request(&self, tx: &UnboundedSender<Message>) -> Result<(), Box<dyn Error>> {
        let request = LoginRequest {
            requestType: String::from("login"),
            username: self.server_username.clone(),
            password: self.server_password.clone(),
            session: self.session_id,
            maxRequests: 10,
        };
        self.send_request(tx, &request)
    }

    async fn receive_response<T: DeserializeOwned>(
        &self,
        rx: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) -> Result<T, Box<dyn Error>> {
        let message = self.receive(rx).await?;
        let s = message.to_string();
        let deserialized: T = serde_json::from_str(&s)?;
        Ok(deserialized)
    }

    async fn receive(
        &self,
        rx: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) -> Result<Message, Box<dyn Error>> {
        loop {
            let next = rx.next().await;
            if let Some(Ok(message)) = next {
                return Ok(message);
            }
        }
    }

    fn send_request<T: Serialize>(
        &self,
        tx: &UnboundedSender<Message>,
        t: &T,
    ) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(&t)?;
        let data = json.as_bytes();
        tx.unbounded_send(Message::binary(data))?;
        Ok(())
    }
}
