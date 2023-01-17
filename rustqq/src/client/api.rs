#![allow(dead_code, unused)]
use async_trait;
use codegen::ApiName;
use reqwest::Client;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use tokio;
pub async fn post_reqwest<T: Serialize + ApiName>(api: &T) -> Result<Value,Box<dyn std::error::Error>> {
    let ip = "127.0.0.1:8080";
    let api_url = format!("http://{}/{}", ip, api.name());
    let client = Client::new();
    let res = client.post(&api_url).json(api).send().await?;
    let res = res.text().await.unwrap();
    let res: serde_json::Value = serde_json::from_str(&res)?;
    println!("res: {:?}", res);
    Ok(res)
}
pub trait ApiName {
    fn name(&self) -> String;
}
pub async fn send_private_message(user_id: i64, message: String) -> serde_json::Value {
    let api = SendPrivateMessage::new(user_id, message);
    // api.post()
    todo!()
}
pub async fn send_group_message(group_id: i64, message: String) -> serde_json::Value {
    let api = SendGroupMessage::new(group_id, message);
    //api.post()
    todo!()
}

#[derive(Serialize, Deserialize, ApiName)]
pub struct SendPrivateMessage {
    user_id: i64,
    group_id: i64,
    message: String,
    auto_space: bool,
}
#[derive(Serialize, Deserialize, ApiName)]
pub struct GetMessage {
    message_id: i64,
}

#[derive(Serialize, Deserialize, ApiName)]
pub struct SendGroupMessage {
    group_id: i64,
    message: String,
    auto_space: bool,
}
impl SendPrivateMessage {
    pub fn new(user_id: i64, message: String) -> Self {
        Self {
            user_id,
            group_id: 0,
            message,
            auto_space: false,
        }
    }
    pub async fn post(&self)->Result<Value,Box<dyn std::error::Error>> {
        post_reqwest(self).await
    }
}
impl SendGroupMessage {
    pub fn new(group_id: i64, message: String) -> Self {
        Self {
            group_id,
            message,
            auto_space: false,
        }
    }
    pub async fn post(&self)->Result<Value, Box<dyn std::error::Error>> {
        post_reqwest(self).await
        
    }
}
impl GetMessage {
    pub fn new(message_id: i64) -> Self {
        Self { message_id }
    }
    pub async fn post(&self)->Result<Value, Box<dyn std::error::Error>> {
        post_reqwest(self).await
    }
}
    