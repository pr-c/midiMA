mod connection;
pub mod objects;
mod requests;
pub mod responses;

use crate::ma_connection::requests::{LoginRequest, PlaybacksRequest, PlaybacksUserInputRequest, SessionIdRequest};
use crate::ma_connection::responses::{LoginRequestResponse, SessionIdResponse};
use connection::Connection;
use futures_util::StreamExt;
use requests::RequestType;
use responses::ResponseWithExplicitType;
use serde::Serialize;
use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

use self::responses::PlaybacksResponse;

pub struct LoginCredentials {
    pub username: String,
    pub password_hash: String,
}

struct ResponseSenders {
    pub playbacks: UnboundedSender<PlaybacksResponse>,
    pub session_id: UnboundedSender<SessionIdResponse>,
    pub login: UnboundedSender<LoginRequestResponse>,
}

struct ResponseReceivers {
    pub playbacks: UnboundedReceiver<PlaybacksResponse>,
    pub session_id: UnboundedReceiver<SessionIdResponse>,
    pub login: UnboundedReceiver<LoginRequestResponse>,
}

fn create_response_receiver_sender_pair() -> (ResponseSenders, ResponseReceivers) {
    let (playbacks_tx, playbacks_rx) = tokio::sync::mpsc::unbounded_channel();
    let (session_id_tx, session_id_rx) = tokio::sync::mpsc::unbounded_channel();
    let (login_tx, login_rx) = tokio::sync::mpsc::unbounded_channel();
    (
        ResponseSenders {
            playbacks: playbacks_tx,
            session_id: session_id_tx,
            login: login_tx,
        },
        ResponseReceivers {
            playbacks: playbacks_rx,
            session_id: session_id_rx,
            login: login_rx,
        },
    )
}

pub struct MaInterface {
    receiver_thread: JoinHandle<()>,
    keep_alive_thread: JoinHandle<()>,
    websocket_sender: UnboundedSender<Message>,
    response_receivers: ResponseReceivers,
    session_id: i32,
}

impl MaInterface {
    pub async fn new(url: &Url, login_credentials: &LoginCredentials) -> Result<MaInterface, Box<dyn Error>> {
        let connection = Connection::new(url).await?;

        let keep_alive_tx = connection.tx.clone();
        let websocket_sender = connection.tx.clone();

        let (response_senders, mut response_receivers) = create_response_receiver_sender_pair();

        let receiver_thread = tokio::spawn(MaInterface::receive_loop(connection, response_senders));
        let session_id = MaInterface::get_session_id(&websocket_sender, &mut response_receivers).await?;
        let keep_alive_thread = tokio::spawn(MaInterface::keep_alive_loop(keep_alive_tx, session_id));

        MaInterface::login(&websocket_sender, &mut response_receivers, login_credentials, &session_id).await?;
        let interface = MaInterface {
            receiver_thread,
            keep_alive_thread,
            websocket_sender,
            response_receivers,
            session_id,
        };
        Ok(interface)
    }

    pub async fn poll_fader_values(&mut self) -> Result<Vec<f32>, Box<dyn Error>> {
        let request = PlaybacksRequest {
            request_type: RequestType::Playbacks.to_string(),
            start_index: Vec::from([000]),
            items_count: Vec::from([10]),
            page_index: 0,
            items_type: Vec::from([2]),
            view: 2,
            exec_button_view_mode: 2,
            buttons_view_mode: 0,
            session: self.session_id,
        };
        self.send_request(request)?;
        let next = self.response_receivers.playbacks.recv().await;
        if let Some(response) = next {
            let mut v: Vec<f32> = Vec::new();
            for group in response.itemGroups {
                for group_of_five in group.items {
                    for executor in group_of_five {
                        for executor_block in executor.executor_blocks {
                            v.push(executor_block.fader.value);
                        }
                    }
                }
            }
            Ok(v)
        } else {
            Err("get_fader_values EOS".into())
        }
    }

    pub fn send_fader_value(&mut self, exec_index: u32, page_index: u32, fader_value: f32) -> Result<(), Box<dyn Error>> {
        let request = PlaybacksUserInputRequest::new(self.session_id, exec_index, page_index, fader_value);
        self.send_request(request)?;
        Ok(())
    }

    async fn keep_alive_loop(tx: UnboundedSender<Message>, session_id: i32) {
        let request = SessionIdRequest::new(&session_id);
        let request_string: String = serde_json::to_string(&request).unwrap();
        let mut interval = interval(Duration::from_millis(4000));
        loop {
            interval.tick().await;
            let send_result = tx.send(Message::text(&request_string));
            if let Err(e) = send_result {
                println!("Keep alive thread exited with error: {:?}", e);
                break;
            }
        }
    }

    async fn get_session_id(tx: &UnboundedSender<Message>, rx: &mut ResponseReceivers) -> Result<i32, Box<dyn Error>> {
        let request = SessionIdRequest::new_unknown_session();
        MaInterface::send_request_to_channel(tx, request)?;
        let next = rx.session_id.recv().await;
        if let Some(response) = next {
            Ok(response.session)
        } else {
            Err("session_id_request EOS".into())
        }
    }

    async fn login(tx: &UnboundedSender<Message>, rx: &mut ResponseReceivers, credentials: &LoginCredentials, session_id: &i32) -> Result<(), Box<dyn Error>> {
        let request = LoginRequest::new(credentials, session_id);
        MaInterface::send_request_to_channel(tx, request)?;
        let next = rx.login.recv().await;
        if let Some(response) = next {
            return if response.result { Ok(()) } else { Err("login invalid credentials".into()) };
        }
        Err("login EOS".into())
    }

    async fn receive_loop(mut connection: Connection, response_senders: ResponseSenders) {
        loop {
            let next = connection.rx.next().await;
            if let Some(result) = next {
                match result {
                    Ok(message) => {
                        if MaInterface::receive_message(message, &response_senders).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tokio::io::stdout().write_all(format!("receive loop error: {:?}", e).as_bytes()).await;
                    }
                }
            } else {
                break;
            }
        }
    }

    async fn receive_message(message: Message, response_senders: &ResponseSenders) -> Result<(), Box<dyn Error>> {
        let response_with_explicit_type = serde_json::from_str::<ResponseWithExplicitType>(&message.to_string());
        match response_with_explicit_type {
            Ok(response) => {
                if let Ok(request_type) = RequestType::from_str(&response.response_type) {
                    MaInterface::receive_message_with_type(message, request_type, response_senders)?;
                }
            }
            Err(_) => {
                if let Ok(session_id_response) = serde_json::from_str::<SessionIdResponse>(&message.to_string()) {
                    let send_result = response_senders.session_id.send(session_id_response);
                    if send_result.is_err() {
                        return Err("session id response channel closed".into());
                    }
                } else if !message.to_string().is_empty() {}
            }
        }
        Ok(())
    }

    fn receive_message_with_type(message: Message, request_type: RequestType, response_senders: &ResponseSenders) -> Result<(), Box<dyn Error>> {
        match request_type {
            RequestType::Login => {
                let login_response = serde_json::from_str::<LoginRequestResponse>(&message.to_string())?;
                let send_result = response_senders.login.send(login_response);
                if send_result.is_err() {
                    return Err("login response channel closed".into());
                }
                Ok(())
            }
            RequestType::Playbacks => {
                let playbacks_response = serde_json::from_str::<PlaybacksResponse>(&message.to_string())?;
                let send_result = response_senders.playbacks.send(playbacks_response);
                if send_result.is_err() {
                    return Err("playbacks response channel closed".into());
                }
                Ok(())
            }
            _ => {
                Err(format!("Request Type unknown '{}'", request_type.to_string()).into())
            }
        }
    }

    fn send_request<T: Serialize>(&self, request: T) -> Result<(), Box<dyn Error>> {
        MaInterface::send_request_to_channel(&self.websocket_sender, request)
    }

    fn send_request_to_channel<T: Serialize>(tx: &UnboundedSender<Message>, request: T) -> Result<(), Box<dyn Error>> {
        let json_string = serde_json::to_string(&request)?;
        let message = Message::text(json_string);
        tx.send(message)?;
        Ok(())
    }
}

impl Drop for MaInterface {
    fn drop(&mut self) {
        self.keep_alive_thread.abort();
        self.receiver_thread.abort();
        println!("MaConnection closed");
    }
}
