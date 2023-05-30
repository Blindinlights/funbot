#![allow(dead_code, unused)]
use async_trait;
use codegen::ApiName;
use log::{debug, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio;
pub async fn post_reqwest<T: Serialize + ApiName>(
    api: &T,
) -> Result<Value, Box<dyn std::error::Error>> {
    let ip = "127.0.0.1:8080";
    let api_url = format!("http://{}/{}", ip, api.name());
    let client = Client::new();

    let res = client.post(&api_url).json(api).send().await?;
    let res = res.text().await.unwrap();
    let res: serde_json::Value = serde_json::from_str(&res)?;
    log::debug!("res: {res:?}");
    Ok(res)
}
pub trait ApiName {
    fn name(&self) -> String;
}
pub async fn send_private_message(user_id: i64, message: String) -> serde_json::Value {
    let api = SendPrivateMessage::new(user_id, message);
    todo!()
}
pub async fn send_group_message(group_id: i64, message: String) -> serde_json::Value {
    let api = SendGroupMessage::new(group_id, message);
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
    pub async fn post(&self) -> Result<Value, Box<dyn std::error::Error>> {
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
    pub async fn post(&self) -> Result<Value, Box<dyn std::error::Error>> {
        post_reqwest(self).await
    }
}
impl GetMessage {
    pub fn new(message_id: i64) -> Self {
        Self { message_id }
    }
    pub async fn post(&self) -> Result<Value, Box<dyn std::error::Error>> {
        post_reqwest(self).await
    }
}
use std::net::SocketAddr;
pub struct CqHttpApi {
    socket_addr: SocketAddr,
    client: Client,
    url: String,
}

type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;
impl CqHttpApi {
    pub fn new() -> Self {
        let addr = std::env::var("CQHTTP_API").unwrap();
        let socket_addr = addr.parse().unwrap();
        let client = Client::new();
        let url = format!("http://{}/", addr);
        Self {
            socket_addr,
            client,
            url,
        }
    }
    async fn invoke(&self, endpoint: &str, data: serde_json::Value) -> BoxResult<String> {
        let res = self
            .client
            .post(endpoint)
            .json(&data)
            .send()
            .await?
            .text()
            .await?;

        debug!("{}: {}", endpoint, res);
        let res: serde_json::Value = serde_json::from_str(&res)?;

        let msg = res["message"].to_string();
        info!("{}: {}", endpoint, msg);
        let res = res["data"].to_string();
        Ok(res)
    }
    pub async fn get_group_list(&self) -> BoxResult<Vec<GroupInfo>> {
        let endpoint = format!("{}{}", self.url, "get_group_list");
        let res = self.invoke(&endpoint, serde_json::json!({})).await?;

        let res: Vec<GroupInfo> = serde_json::from_str(&res)?;
        Ok(res)
    }

    pub async fn get_group_member_info(
        &self,
        group_id: i64,
        user_id: i64,
    ) -> BoxResult<GroupMemberInfo> {
        let endpoint = format!("{}{}", self.url, "get_group_member_info");
        let res = self
            .invoke(
                &endpoint,
                serde_json::json!({ "group_id": group_id, "user_id": user_id }),
            )
            .await?;
        let res: GroupMemberInfo = serde_json::from_str(&res)?;

        Ok(res)
    }
    pub async fn get_group_member_list(&self, group_id: i64) -> BoxResult<Vec<GroupMemberInfo>> {
        let endpoint = format!("{}{}", self.url, "get_group_member_list");
        let res = self
            .invoke(&endpoint, serde_json::json!({ "group_id": group_id }))
            .await?;
        let res: Vec<GroupMemberInfo> = serde_json::from_str(&res)?;
        Ok(res)
    }
    pub async fn send_private_message(&self, user_id: i64, message: &str) -> BoxResult<Value> {
        let endpoint = format!("{}{}", self.url, "send_private_msg");
        let res = self
            .invoke(
                &endpoint,
                serde_json::json!({ "user_id": user_id, "message": message ,"autospace": false,"group_id":0}),
            )
            .await?;
        let res: serde_json::Value = serde_json::from_str(&res)?;
        Ok(res)
    }

    pub async fn send_group_message(&self, group_id: i64, message: String) -> BoxResult<Value> {
        let endpoint = format!("{}{}", self.url, "send_group_msg");
        let res = self
            .invoke(
                &endpoint,
                serde_json::json!({ "group_id": group_id, "message": message ,"autospace": false}),
            )
            .await?;
        let res: serde_json::Value = serde_json::from_str(&res)?;
        Ok(res)
    }
    pub async fn get_msg(&self, message_id: i64) -> BoxResult<Value> {
        let endpoint = format!("{}{}", self.url, "get_msg");
        let res = self
            .invoke(&endpoint, serde_json::json!({ "message_id": message_id }))
            .await?;
        let res: serde_json::Value = serde_json::from_str(&res)?;
        Ok(res)
    }
    pub async fn get_friend_list(&self) -> BoxResult<Vec<FriendInfo>> {
        let endpoint = format!("{}{}", self.url, "get_friend_list");
        let res = self.invoke(&endpoint, serde_json::json!({})).await?;
        let res: Vec<FriendInfo> = serde_json::from_str(&res)?;
        Ok(res)
    }
}
#[derive(Deserialize)]
pub struct GroupInfo {
    pub group_id: i64,
    pub group_name: String,
    pub member_count: i64,
    pub max_member_count: i64,
    pub group_memo: Option<String>,
    pub group_create_time: u32,
    pub group_level: u32,
}
#[derive(Deserialize)]
pub struct GroupMemberInfo {
    pub group_id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub card: Option<String>,
    pub sex: Option<String>,
    pub age: Option<i64>,
    pub area: Option<String>,
    pub join_time: Option<u32>,
    pub last_sent_time: Option<u32>,
    pub level: Option<String>,
    pub role: Option<String>,
    pub unfriendly: Option<bool>,
    pub title: Option<String>,
    pub title_expire_time: Option<u32>,
    pub card_changeable: Option<bool>,
    pub shut_up_timestamp: Option<u32>,
}
#[derive(Deserialize)]
pub struct FriendInfo {
    pub user_id: i64,
    pub nickname: String,
    pub remark: String,
}
