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
use openai::daily::daily;
use quote::bing_pic;
#[actix_web::main]
async fn main() {
    info!("Bot start");
    setup_logger().unwrap();
    let _jobs=rustqq::app::AsyncJobScheduler::new()
        .add_job(daily());
    
    let mut app = app::App::new()
        .event(Box::new(bing_pic))
        .event(Box::new(url_preview))
        .event(Box::new(quote_it))
        .event(Box::new(open_journey))
        .event(Box::new(emoji_mix))
        .event(Box::new(gpt_private))
        .event(Box::new(gpt_group))
        .event(Box::new(gpt4))
        .event(Box::new(open_image))
        .event(Box::new(audio_gpt));
    app.config();
    //jobs.run().await;
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
