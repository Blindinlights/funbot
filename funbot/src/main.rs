
use rustqq::app;
mod echo;
mod weather;
mod quote;
mod make_it_quote;
mod openai;
use echo::{echo_msg,url_preview};
use quote::{one_quote,bing_pic,copy_paste};
use weather::{weather_query, weather_report};
use make_it_quote::quote_it;
use openai::{open_image,open_journey};
#[actix_web::main]
async fn main(){
    app::App::new()
    .event(Box::new(echo_msg))
    .event(Box::new(weather_report))
    .event(Box::new(weather_query))
    .event(Box::new(one_quote))
    .event(Box::new(bing_pic))
    .event(Box::new(copy_paste))
    .event(Box::new(url_preview))
    .event(Box::new(quote_it))
    .event(Box::new(open_image))
    .event(Box::new(open_journey))
    .run()
    .await.unwrap();
}