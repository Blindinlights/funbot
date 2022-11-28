//web server
#![allow(dead_code,unused)]
use std::io::Error;

use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use crate::event::events::*;
use serde_json;
#[post("/")]
async fn index(data:String)-> impl Responder {
    let event:serde_json::Value = serde_json::from_str(&data).unwrap();
    
    HttpResponse::Ok().body(data)
}
pub async fn build_server(ip:&str,port:u16) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
    })
    .bind((ip,port))?
    .run()
    .await
}
pub fn get_event(event:&serde_json::Value)->Result<Event,Error>{
    let post_type = event["post_type"].as_str().unwrap();
    match post_type{
        "message"=>{
            let message_type = event["message_type"].as_str().unwrap();
            match message_type{
                "private"=>Ok(Event::PrivateMessage(serde_json::from_value(event.clone())?)),
                "group"=>Ok(Event::GroupMessage(serde_json::from_value(event.clone())?)),
                _=>Ok(Event::Unknown)
            }
        },
        "notice"=>{
            let notice_type = event["notice_type"].as_str().unwrap();
            match notice_type{
                "group_upload"=>Ok(Event::GroupFileUpload(serde_json::from_value(event.clone()).unwrap())),
                "group_admin"=>Ok(Event::GroupAdminChange(serde_json::from_value(event.clone()).unwrap())),
                "group_decrease"=>Ok(Event::GroupMemberReduce(serde_json::from_value(event.clone()).unwrap())),
                "group_increase"=>Ok(Event::GroupMemberIncrease(serde_json::from_value(event.clone()).unwrap())),
                "group_ban"=>Ok(Event::GroupMute(serde_json::from_value(event.clone()).unwrap())),
                "friend_add"=>Ok(Event::FriendAdd(serde_json::from_value(event.clone()).unwrap())),
                "group_recall"=>Ok(Event::GroupMessageRecall(serde_json::from_value(event.clone()).unwrap())),
                "friend_recall"=>Ok(Event::FriendMessageRecall(serde_json::from_value(event.clone()).unwrap())),
                "poke"=>{
                    let sub_type = event["sub_type"].as_str().unwrap();
                    match sub_type{
                        "friend"=>Ok(Event::FriendPoke(serde_json::from_value(event.clone()).unwrap())),
                        "group"=>Ok(Event::GroupPoke(serde_json::from_value(event.clone()).unwrap())),
                        _=>Ok(Event::Unknown)
                    }
                },
                _=>Ok(Event::Unknown)
            }
        },
    
        "request"=>{
            let request_type = event["request_type"].as_str().unwrap();
            match request_type{
                "friend"=>Ok(Event::FriendRequest(serde_json::from_value(event.clone()).unwrap())),
                "group"=>Ok(Event::GroupRequest(serde_json::from_value(event.clone()).unwrap())),
                _=>Ok(Event::Unknown)
            }
        },
        "meta_event"=>Ok(Event::MetaEvent(serde_json::from_value(event.clone()).unwrap())),
    
        _=>Ok(Event::Unknown)
    }
}  

