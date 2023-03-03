use rustqq::app;
mod blive;
mod echo;
mod festival;
mod make_it_quote;
mod openai;
mod quote;
mod weather;

use echo::{echo_msg, emoji_mix, say, url_preview};
use make_it_quote::quote_it;
use openai::{open_journey};
use quote::{bing_pic, copy_paste, one_quote};
use rustqq::app::AsyncJobScheduler;
use weather::{weather_query, weather_report};
#[actix_web::main]
async fn main() {
    let mut scheduler = AsyncJobScheduler::new();
    scheduler.add_job(festival::get_job());
    scheduler.add_job(blive::blive_job());
    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
        }
    });

    let mut app=app::App::new()
        .event(Box::new(echo_msg))
        .event(Box::new(weather_report))
        .event(Box::new(weather_query))
        .event(Box::new(one_quote))
        .event(Box::new(bing_pic))
        .event(Box::new(copy_paste))
        .event(Box::new(url_preview))
        .event(Box::new(quote_it))
        .event(Box::new(open_journey))
        .event(Box::new(emoji_mix))
        .event(Box::new(say));
    app.config();
    app.run().await.unwrap();
}
