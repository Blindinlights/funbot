#![allow(unused)]
use log::info;

use crate::client::api::*;
use crate::event::events::*;
#[async_trait::async_trait]
pub trait Reply {
    async fn reply(&self, msg: &str)->Result<(),Box<dyn std::error::Error>>;
}
#[async_trait::async_trait]
impl Reply for PrivateMessage {
    async fn reply(&self, msg: &str)->Result<(), Box<dyn std::error::Error>> {
        let api = SendPrivateMessage::new(self.user_id, msg.to_string());
        info!("发送给{}的消息:{}",self.user_id,msg);
        api.post().await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl Reply for GroupMessage {
    async fn reply(&self, msg: &str)->Result<(), Box<dyn std::error::Error>> {
        let api = SendGroupMessage::new(self.group_id, msg.to_string());
        info!("发送给群{}的消息:{}",self.group_id,msg);
        api.post().await?;
        Ok(())
    }
}
