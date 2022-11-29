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
mod quote;
use echo::echo_msg;
use quote::{one_quote,bing_pic};
use weather::{weather_query, weather_report};
#[actix_web::main]
async fn main() {
    app::App::new()
    .event(Box::new(echo_msg))
    .event(Box::new(weather_report))
    .event(Box::new(weather_query))
    .event(Box::new(one_quote))
    .event(Box::new(bing_pic))
    .run()
    .await;
    
    
        
}