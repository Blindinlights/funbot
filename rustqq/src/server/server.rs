//web server
#![allow(dead_code, unused)]
use std::io::Error;
use std::rc::Rc;
use std::sync::Arc;

use crate::app;
use crate::event::events::*;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::de::value;
use serde_json;
use std::sync::atomic::AtomicPtr;
use tokio::sync::Mutex;
#[post("/")]
async fn index(data: String, handler: web::Data<app::App>) -> impl Responder {
    println!("data:{}", data);
    let value: serde_json::Value = serde_json::from_str(&data).unwrap();
    if let Ok(event) = get_event(&value) {
        println!("get event");
        match event {
            Event::GroupMessage(_) => {
                println!("group message");
                //echo(event).await;
            }
            Event::Unknown => {
                println!("unknown event");
                return actix_web::HttpResponse::Ok().body("Unknow event type");
            }
            _ => {}
        }
        println!("event: {:?}", event);
        let res=(*handler).handle_event(&event).await;
        if let Err(err)=res{
            println!("err:{}",err);
        }
    } else {
        println!("error");
    }
    HttpResponse::Ok().body("Hello world!")
}
pub async fn build_server(app: app::App) -> Result<(), Box<dyn std::error::Error>> {
    let app = app.clone();
    //let web_app=
    let ip = ("127.0.0.1", 8755);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app.clone()))
            .service(index)
    })
    .bind(ip)?
    .run()
    .await?;
    Ok(())
}
pub fn get_event(event: &serde_json::Value) -> Result<Event, Error> {
    let post_type = event["post_type"].as_str().unwrap();
    match post_type {
        "message" => {
            let message_type = event["message_type"].as_str().unwrap();
            match message_type {
                "private" => Ok(Event::PrivateMessage(serde_json::from_value(
                    event.clone(),
                )?)),
                "group" => Ok(Event::GroupMessage(serde_json::from_value(event.clone())?)),
                _ => Ok(Event::Unknown),
            }
        }
        "notice" => {
            let notice_type = event["sub_type"].as_str().unwrap();
            match notice_type {
                "group_upload" => Ok(Event::GroupFileUpload(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "group_admin" => Ok(Event::GroupAdminChange(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "group_decrease" => Ok(Event::GroupMemberReduce(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "group_increase" => Ok(Event::GroupMemberIncrease(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "group_ban" => Ok(Event::GroupMute(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "friend_add" => Ok(Event::FriendAdd(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "group_recall" => Ok(Event::GroupMessageRecall(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "friend_recall" => Ok(Event::FriendMessageRecall(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "poke" => Ok(Event::GroupPoke(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                _ => Ok(Event::Unknown),
            }
        }

        "request" => {
            let request_type = event["request_type"].as_str().unwrap();
            match request_type {
                "friend" => Ok(Event::FriendRequest(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                "group" => Ok(Event::GroupRequest(
                    serde_json::from_value(event.clone()).unwrap(),
                )),
                _ => Ok(Event::Unknown),
            }
        }
        "meta_event" => Ok(Event::MetaEvent(
            serde_json::from_value(event.clone()).unwrap(),
        )),

        _ => Ok(Event::Unknown),
    }
}
