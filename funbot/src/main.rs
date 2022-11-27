#![allow(unused)]
use rustqq::event::events::*;
use rustqq::server;
use rustqq::event::reply_trait::*;
use rustqq::app;
use rustqq::server::server::get_event;
use serde_json;
#[rustqq::handler]
async fn echo(event:Event){
    if let Event::PrivateMessage(ref msg) = event.clone(){
        //msg.
       let api = rustqq::client::api::SendPrivateMessage::new(msg.user_id,msg.message.clone());
       rustqq::client::api::post_reqwest(&api).await;
    }

}
#[actix_web::post("/")]
async fn index(data:String)-> impl actix_web::Responder {
    let e:PrivateMessage = serde_json::from_str(&data).unwrap();
    let app = app::app::App::new();
    app.service(Box::new(echo)).run(&Event::PrivateMessage(e)).await;
    
    //let e=get_event(&event);
    
    actix_web::HttpResponse::Ok().body(data)
}
#[actix_web::main]
async fn main(){

    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .service(index)
    }).bind(("127.0.0.1",8755)).unwrap().run().await;


}