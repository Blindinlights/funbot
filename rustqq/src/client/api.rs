#![allow(dead_code)]
use reqwest::Client;
use serde::{Serialize,Deserialize};
use codegen::PostApi;
async fn post_reqwest<T: serde::Serialize>(api: &T)->serde_json::Value{
    let client = Client::new();
    let res = client.post("127.0.0.1").json(&api).send().await.unwrap();
    let ret =res.text().await.unwrap();

    serde_json::json!(ret)

}

pub trait PostApi{
    fn post(&self)->serde_json::Value;
}
  

#[derive(Serialize,Deserialize,PostApi)]
pub struct SendPrivateMessage{
    user_id:i64,
    group_id:i64,
    message:String,
    auto_space:bool
}
#[derive(Serialize,Deserialize,PostApi)]
pub struct SendGroupMessage{
    group_id:i64,
    message:String,
    auto_space:bool
}