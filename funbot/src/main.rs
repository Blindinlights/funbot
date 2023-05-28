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

use crate::openai::chat_set;
#[tokio::main]
async fn main() {
    info!("Bot start");
    setup_logger().unwrap();
    let app = app::App::new()
        .bind("127.0.0.1:8755".parse().unwrap())
        .service(bing_pic)
        .service(url_preview)
        .service(quote_it)
        .service(open_journey)
        .service(emoji_mix)
        .service(gpt_private)
        .service(gpt_group)
        .service(chat_set)
        .service(gpt4)
        .service(open_image)
        .service(audio_gpt);
    app.run().await.unwrap();
}
fn setup_logger() -> Result<(), fern::InitError> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let date = fern::DateBased::new("log/", today);
    let log_file = fern::Dispatch::new().chain(date);
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .filter(|metedata| {
            metedata.target() != "sqlx::query" && metedata.level() <= log::LevelFilter::Info
        })
        .chain(log_file)
        .chain(std::io::stdout())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .apply()?;
    Ok(())
}
