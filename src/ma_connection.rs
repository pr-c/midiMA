mod connection;
mod objects;
mod requests;
mod responses;
use connection::Connection;
use futures_util::StreamExt;
use requests::{LoginRequest, PlaybacksRequst, SessionIdRequest};
use responses::{ConnectResponse, LoginRequestResponse, SessionIdResponse};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::fs;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

use crate::ma_connection::responses::PlaybacksResponse;

pub struct MaInterface {
    server_username: String,
    server_password: String,
    session_id: i32,
    connection: Connection,
}

impl MaInterface {
    pub async fn new(
        server_ip: String,
        server_username: String,
        server_password: String,
    ) -> Result<MaInterface, Box<dyn Error>> {
        let url = Url::parse(&format!("ws://{}", server_ip))?;
        let connection = Connection::new(url).await?;
        Ok(MaInterface {
            server_username,
            server_password,
            session_id: 0,
            connection,
        })
    }

    pub async fn start_session(&mut self) -> Result<(), Box<dyn Error>> {
        self.receive_response::<ConnectResponse>().await?;
        self.send_session_id_requst()?;
        let session_id_response = self.receive_response::<SessionIdResponse>().await?;
        self.session_id = session_id_response.session;
        println!("Session ID: {}", self.session_id);
        self.send_login_request()?;
        let login_response = self.receive_response::<LoginRequestResponse>().await?;
        match login_response.result {
            true => println!("Success"),
            false => println!("Failure"),
        }
        Ok(())
    }

    pub async fn request_playbacks(&mut self) -> Result<(), Box<dyn Error>> {
        let request = PlaybacksRequst {
            requestType: String::from("playbacks"),
            startIndex: Vec::from([000]),
            itemsCount: Vec::from([10]),
            pageIndex: 0,
            itemsType: Vec::from([2]),
            view: 2,
            execButtonViewMode: 2,
            buttonsViewMode: 0,
            session: self.session_id,
        };
        self.send_request(&request)?;
        let response = self.receive_response::<PlaybacksResponse>().await?;
        for group in response.itemGroups {
            for group_of_five in group.items {
                for executor in group_of_five {
                   for executor_block in executor.executor_blocks {
                       println!("{} {} {} {}", executor_block.button1.pressed, executor_block.button2.pressed, executor_block.fader.value, executor_block.button3.pressed);
                   } 
                }
            }
        }
        Ok(())
    }

    fn send_session_id_requst(&self) -> Result<(), Box<dyn Error>> {
        let request = SessionIdRequest { session: 0 };
        self.send_request(&request)
    }

    fn send_login_request(&self) -> Result<(), Box<dyn Error>> {
        let request = LoginRequest {
            requestType: String::from("login"),
            username: self.server_username.clone(),
            password: self.server_password.clone(),
            session: self.session_id,
            maxRequests: 10,
        };
        self.send_request(&request)
    }

    async fn receive_response<T: DeserializeOwned>(&mut self) -> Result<T, Box<dyn Error>> {
        let message = self.receive().await?;
        let s = message.to_string();
        fs::write("log.txt", String::from(&s) + "\n\n\n\n").unwrap();
        let deserialized: T = serde_json::from_str(&s)?;
        Ok(deserialized)
    }

    async fn receive(&mut self) -> Result<Message, Box<dyn Error>> {
        loop {
            let next = self.connection.rx.next().await;
            if let Some(Ok(message)) = next {
                return Ok(message);
            }
        }
    }

    fn send_request<T: Serialize>(&self, t: &T) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(&t)?;
        let data = json.as_bytes();
        self.connection.tx.unbounded_send(Message::binary(data))?;
        Ok(())
    }
}
