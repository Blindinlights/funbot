use reqwest::Client;
use reqwest::ClientBuilder;
use rustqq::client::message::RowMessage;
use rustqq::event::events::Event;
use rustqq::event::events::Meassages;
use rustqq::event::reply_trait::Reply;
use rustqq::handler;
use serde_json::json;

#[handler]
async fn echo_msg(event: Event) -> Result<(), Box<dyn std::error::Error>> {
    println!("echo mod");
    if let Event::PrivateMessage(ref msg) = event.clone() {
        if msg.start_with("echo ") {
            //println!("echo: {:?}", msg);
            msg.reply(msg.message.clone().replace("echo ", "").as_str())
                .await?;
        }
    }
    if let Event::GroupMessage(ref msg) = event.clone() {
        if msg.start_with("echo ") {
            //println!("echo: {:?}", msg);
            msg.reply(msg.message.clone().replace("echo ", "").as_str())
                .await?;
        }
    }
    Ok(())
}
#[handler]
pub async fn github_url_preview(event: Event) -> Result<(), Box<dyn std::error::Error>> {
    let url="https://opengraph.githubassets.com/3ce26901f1f7120dd7eb84e7e7bdcb82210d183ab7270db802a74b9eb32109db/";
    if let Event::GroupMessage(e) = event.clone() {
        if e.start_with("https://github.com/") {
            let mut msg = e.message.clone();
            let data = json!({ "repo_url": msg });
            println!("Start post");
            let res = Client::new()
                .post("http://127.0.0.1:5000/github_repo")
                .json(&data)
                .send()
                .await?;
            println!("ok");
            let json = res.json::<serde_json::Value>().await?;
            let title = json["title"].as_str().unwrap();
            let description = json["description"].as_str().unwrap();
            let image = json["image"].as_str().unwrap();

            //msg = msg.replace("https://github.com/", url);
            //println!("{}", msg);
            let mut re = RowMessage::new();
            re.add_plain_txt(title);
            re.shift_line();
            //if !description.contains(title) {
            re.add_plain_txt("========================\n");
            re.add_plain_txt(description);
            re.shift_line();
            //}
            re.add_image(image);
            //re.add_image(msg.as_str());
            e.reply(re.get_msg()).await?;
        }
    }
    Ok(())
}
