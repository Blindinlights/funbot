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
#[actix_web::main]
async fn main() {
    setup_logger().unwrap();
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
    app.run().await.unwrap();
}
fn setup_logger() -> Result<(), fern::InitError> {
    let log_file = fern::log_file("log.txt")?;

    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Info)
                .filter(|metadata| metadata.target() != "sqlx::query")
                .chain(std::io::stdout())
                .format(|out, msg, record| {
                    let colors = fern::colors::ColoredLevelConfig::new()
                        .info(fern::colors::Color::Green)
                        .debug(fern::colors::Color::Magenta)
                        .error(fern::colors::Color::Red)
                        .warn(fern::colors::Color::Yellow);
                    if msg.to_string().chars().count() < 60 {
                        out.finish(format_args!(
                            "{} {} {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            colors.color(record.level()),
                            msg.to_string().replace('\n', "")
                        ))
                    } else {
                        let msg = msg
                            .to_string()
                            .chars()
                            .take(60)
                            .collect::<String>()
                            .replace('\n', "");
                        out.finish(format_args!(
                            "{} {} {}...",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            colors.color(record.level()),
                            msg
                        ))
                    }
                }),
        )
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Info)
                .filter(|metedata| metedata.target() != "sqlx::query")
                .chain(log_file)
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{} [{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        message
                    ))
                }),
        )
        .apply()?;
    Ok(())
}
