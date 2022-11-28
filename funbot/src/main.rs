#![allow(unused)]
use rustqq::client::message::RowMessage;
use reqwest;
use rustqq::app;
use rustqq::event::events::*;
use rustqq::event::reply_trait::*;
use rustqq::server::get_event;
use serde_json::Value;
mod echo;
mod weather;
use echo::echo_msg;
use weather::{weather_query, weather_report};




#[actix_web::post("/")]
async fn index(data: String) -> impl actix_web::Responder {
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
        let app = app::app::App::new();
        app.service(Box::new(echo_msg))
            .service(Box::new(weather_query))
            .service(Box::new(weather_report))
            .run(&event)
            .await;
    } else {
        println!("error");
    }
    actix_web::HttpResponse::Ok().body(data)
}
#[actix_web::main]
async fn main() {
    actix_web::HttpServer::new(|| actix_web::App::new().service(index))
        .bind(("127.0.0.1", 8755))
        .unwrap()
        .run()
        .await;
}
