use crate::event::events::*;
use crate::client::api::*;
#[async_trait::async_trait]
pub trait Reply {
    async fn reply(&self, msg: String) -> serde_json::Value;
}
#[async_trait::async_trait]
impl Reply for PrivateMessage {
    async fn reply(&self, msg: String) -> serde_json::Value {
        let api = SendPrivateMessage::new(self.user_id, msg);
        api.post()
    }
}
#[async_trait::async_trait]
impl Reply for GroupMessage {
    async fn reply(&self, msg: String) -> serde_json::Value {
        let api = SendGroupMessage::new(self.group_id, msg);
        api.post()
    }
}
    
