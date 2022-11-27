#![allow(dead_code,unused)]
use reqwest::Client;
use serde::{Serialize,Deserialize};
use codegen::{
    ApiName,
    PostApi,
};
use tokio;
use async_trait;
pub async fn post_reqwest<T:Serialize>(api: &T)->serde_json::Value{
    let ip="127.0.0.1:8080";
    println!("post_reqwest");
    let res=Client::new().post(format!("http://{}/send_private_msg",ip))
        .json(api)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("post_reqwest res:{}",res);
    serde_json::from_str("{}").unwrap()
    
    
}
pub async  fn send_private_message(user_id:i64,message:String)->serde_json::Value{
    let api = SendPrivateMessage::new(user_id,message);
    api.post()
}
pub async fn send_group_message(group_id:i64,message:String)->serde_json::Value{
    let api = SendGroupMessage::new(group_id,message);
    api.post()
}

pub trait PostApi{
    fn post(&self)->serde_json::Value;
}
trait ApiName{
    fn name(&self)->String;
} 


#[derive(Serialize,Deserialize,ApiName)]
pub struct SendPrivateMessage{
    user_id:i64,
    group_id:i64,
    message:String,
    auto_space:bool
}
impl PostApi for SendPrivateMessage{
    fn post(&self)->serde_json::Value{
        tokio::runtime::Runtime::new().unwrap().block_on(post_reqwest(&self))
        //todo!()
    }
}
#[derive(Serialize,Deserialize,PostApi,ApiName)]
pub struct SendGroupMessage{
    group_id:i64,
    message:String,
    auto_space:bool
}
 
 

impl SendPrivateMessage{
    pub fn new(user_id:i64,message:String)->Self{
        Self{
            user_id,
            group_id:0,
            message,
            auto_space:false
        }
    }
    pub fn get_api_name(&self)->String{
        "send_private_msg".to_string()
    }
}
impl SendGroupMessage{
    pub fn new(group_id:i64,message:String)->Self{
        Self{
            group_id,
            message,
            auto_space:false
        }
    }        
}