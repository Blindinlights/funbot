//web server
#![allow(dead_code,unused)]
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
pub fn get_event(event:&serde_json::Value)->Event{
    let post_type = event["post_type"].as_str().unwrap();
    match post_type{
        "message"=>{
            let message_type = event["message_type"].as_str().unwrap();
            match message_type{
                "private"=>Event::PrivateMessage(serde_json::from_value(event.clone()).unwrap()),
                "group"=>Event::GroupMessage(serde_json::from_value(event.clone()).unwrap()),
                _=>Event::Unknown
            }
        },
        "notice"=>{
            let notice_type = event["notice_type"].as_str().unwrap();
            match notice_type{
                "group_upload"=>Event::GroupFileUpload(serde_json::from_value(event.clone()).unwrap()),
                "group_admin"=>Event::GroupAdminChange(serde_json::from_value(event.clone()).unwrap()),
                "group_decrease"=>Event::GroupMemberReduce(serde_json::from_value(event.clone()).unwrap()),
                "group_increase"=>Event::GroupMemberIncrease(serde_json::from_value(event.clone()).unwrap()),
                "group_ban"=>Event::GroupMute(serde_json::from_value(event.clone()).unwrap()),
                "friend_add"=>Event::FriendAdd(serde_json::from_value(event.clone()).unwrap()),
                "group_recall"=>Event::GroupMessageRecall(serde_json::from_value(event.clone()).unwrap()),
                "friend_recall"=>Event::FriendMessageRecall(serde_json::from_value(event.clone()).unwrap()),
                "poke"=>{
                    let sub_type = event["sub_type"].as_str().unwrap();
                    match sub_type{
                        "friend"=>Event::FriendPoke(serde_json::from_value(event.clone()).unwrap()),
                        "group"=>Event::GroupPoke(serde_json::from_value(event.clone()).unwrap()),
                        _=>Event::Unknown
                    }
                },
                _=>Event::Unknown
            }
        },
    
        "request"=>{
            let request_type = event["request_type"].as_str().unwrap();
            match request_type{
                "friend"=>Event::FriendRequest(serde_json::from_value(event.clone()).unwrap()),
                "group"=>Event::GroupRequest(serde_json::from_value(event.clone()).unwrap()),
                _=>Event::Unknown
            }
        },
        "meta_event"=>Event::MetaEvent(serde_json::from_value(event.clone()).unwrap()),
    
        _=>Event::Unknown
    }
}  

