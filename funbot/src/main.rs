use rustqq::app;
mod echo;
mod festival;
mod make_it_quote;
mod openai;
mod quote;
mod weather;
mod blive;
use echo::{echo_msg, url_preview};
use make_it_quote::quote_it;
use openai::{open_image, open_journey};
use quote::{bing_pic, copy_paste, one_quote};
use rustqq::app::AsyncJobScheduler;
use weather::{weather_query, weather_report};
use blive::{add_live, delete_live};
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
        .event(Box::new(add_live))
        .event(Box::new(delete_live))
        .run()
        .await
        .unwrap();
}
