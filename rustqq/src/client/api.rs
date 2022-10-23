#![allow(dead_code)]
use reqwest::Client;
use serde::{Serialize,Deserialize};
async fn post_reqwest<T: Serialize>(api: &T)->serde_json::Value{
    let client = Client::new();
    let res = client.post("127.0.0.1").json(&api).send().await.unwrap();
    let ret =res.text().await.unwrap();

    serde_json::json!(ret)

}
#[derive(Serialize,Deserialize)]
pub struct SendPrivateMessage{
    user_id:i64,
    group_id:i64,
    message:String,
    auto_space:bool
}impl SendPrivateMessage {
    pub async fn post(&self)->serde_json::Value{
        post_reqwest(&self).await
    }
    
}
#[derive(Serialize,Deserialize)]
pub struct SendGroupMessage{
    group_id:i64,
    message:String,
    auto_space:bool
}
impl  SendGroupMessage{
    pub async fn post(&self)->serde_json::Value{
        post_reqwest(&self).await
    }
}