use rustqq::event::reply_trait::Reply;
use rustqq::event::events::Meassages;
use rustqq::event::events::Event;
use rustqq::handler;


#[handler]
async fn echo_msg(event: Event) ->Result<(),Box<dyn std::error::Error>>{
    println!("echo mod");
    if let Event::PrivateMessage(ref msg) = event.clone() {
        if msg.start_with("echo ") {
            //println!("echo: {:?}", msg);
            msg.reply(msg.message.clone().replace("echo ", "").as_str()).await?;
        }
    }
    if let Event::GroupMessage(ref msg) = event.clone() {
        if msg.start_with("echo ") {
            //println!("echo: {:?}", msg);
            msg.reply(msg.message.clone().replace("echo ", "").as_str()).await?;
        }

        
    }
    Ok(())
}