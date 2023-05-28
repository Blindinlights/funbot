#![allow(unused)]
extern crate fern;
#[macro_use]
extern crate log;


use rustqq::app;
mod echo;
mod make_it_quote;
mod openai;
mod quote;
use echo::{emoji_mix, url_preview};
use make_it_quote::quote_it;
use openai::{audio_gpt, gpt4, gpt_group, gpt_private, open_image, open_journey};
use quote::bing_pic;
#[tokio::main]
async fn main() {
    info!("Bot start");
    setup_logger().unwrap();
    let mut app = app::App::new()
        .bind("127.0.0.1:8755".parse().unwrap())
        .event(bing_pic)
        .event(url_preview)
        .event(quote_it)
        .event(open_journey)
        .event(emoji_mix)
        .event(gpt_private)
        .event(gpt_group)
        .event(gpt4)
        .event(open_image)
        .event(audio_gpt);
    app.config();

    app.run().await.unwrap();
    
}
fn setup_logger() -> Result<(), fern::InitError> {
    let today=chrono::Local::now().format("%Y-%m-%d").to_string();
    let date=fern::DateBased::new("log/",today);
    let log_file = fern::Dispatch::new().chain(date);

    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .filter(|metedata| metedata.target() != "sqlx::query")
        .chain(log_file)
        .chain(std::io::stdout())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} [{}] {}",
                record.target(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .apply()?;
    Ok(())
}
