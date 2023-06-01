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
use openai::{
    audio_gpt,
    daily::{daily, daily_cmd},
    gpt4, gpt_group, gpt_private, open_image, open_journey,
};
use quote::bing_pic;

use crate::openai::chat_set;
#[actix_web::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    info!("Bot start");
    setup_logger().unwrap();
    let mut jobs = rustqq::app::AsyncJobScheduler::new();
    jobs.add_job(daily());
    actix_web::rt::spawn(async move {
        jobs.run().await;
    });

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
        .service(audio_gpt)
        .service(daily_cmd);
    app.run().await.unwrap();
}
fn setup_logger() -> Result<(), fern::InitError> {
    let date = fern::DateBased::new("log/", "%Y-%m-%d-funbot.log");
    let level = log_filter();

    fern::Dispatch::new()
        .level(level)
        .level_for("sqlx", log::LevelFilter::Warn)
        .chain(date)
        .chain(std::io::stdout())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {} {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.line().unwrap(),
                message
            ))
        })
        .apply()?;
    Ok(())
}
fn log_filter() -> log::LevelFilter {
    let env = std::env::var("RUST_LOG").unwrap_or("info".to_string());
    match env.as_str() {
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    }
}
