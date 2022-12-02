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
mod make_it_quote;
use echo::{echo_msg,github_url_preview};
use quote::{one_quote,bing_pic,copy_paste,crazy_thu};
use weather::{weather_query, weather_report};
use make_it_quote::quote_it;
#[actix_web::main]
async fn main() {
    app::App::new()
    .event(Box::new(echo_msg))
    .event(Box::new(weather_report))
    .event(Box::new(weather_query))
    .event(Box::new(one_quote))
    .event(Box::new(bing_pic))
    .event(Box::new(copy_paste))
    .event(Box::new(github_url_preview))
    .event(Box::new(quote_it))
    .run()
    .await;
    
    
        
}
