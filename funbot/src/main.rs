extern crate fern;
#[macro_use]
extern crate log;
use rustqq::app;
mod blive;
mod echo;
mod festival;
mod make_it_quote;
mod openai;
mod quote;
mod weather;
use echo::{emoji_mix, url_preview};
use make_it_quote::quote_it;
use openai::{gpt_private,gpt_group, gpt4, open_journey};
use quote::{bing_pic, one_quote};
use rustqq::app::AsyncJobScheduler;
use weather::{weather_query, weather_report};
#[actix_web::main]
async fn main() {
    //pretty_env_logger::init();
    setup_logger().unwrap();
    let mut scheduler = AsyncJobScheduler::new();
    scheduler.add_job(festival::get_job());
    scheduler.add_job(blive::blive_job());
    // tokio::spawn(async move {
    //     loop {
    //         scheduler.run_pending().await;
    //     }

    // });

    let mut app = app::App::new()
        .event(Box::new(weather_report))
        .event(Box::new(weather_query))
        .event(Box::new(one_quote))
        .event(Box::new(bing_pic))
        .event(Box::new(url_preview))
        .event(Box::new(quote_it))
        .event(Box::new(open_journey))
        .event(Box::new(emoji_mix))
        .event(Box::new(gpt_private))
        .event(Box::new(gpt_group))
        .event(Box::new(gpt4));
    app.config();
    app.run().await.unwrap();
}
fn setup_logger() -> Result<(), fern::InitError> {
    let log_file = fern::log_file("log.txt")?;
    
    // 过滤器来只显示来自特定目标的日志
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Info)
                .filter(|metadata| metadata.target() !="sqlx::query")
                .chain(std::io::stdout()),
        )
        .chain(log_file)
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
