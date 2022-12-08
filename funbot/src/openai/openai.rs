use std::collections::HashMap;
use reqwest;
use rustqq::{handler, event::{self, Event, Meassages, Reply}, client::message::RowMessage};
use  serde_json::Value;
async fn generate_image(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = "https://api.openai.com/v1/images/generations";
        let mut map=HashMap::new();
        map.insert("prompt", prompt);
        //map.insert("n", "1");
        map.insert("size", "1024x1024");

        //add header "Content-Type: application/json""Authorization: Bearer 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"
        let api_key=std::env::var("OPENAI_API_KEY")?;
        println!("api_key:{}", api_key);
        let api_key="Bearer {}".replace("{}", &api_key);
        let res = client.post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", api_key)
            .json(&map)
            .send()
            .await?
            .text()
            .await?;
            let v: Value = serde_json::from_str(&res)?;
            let image_url=v["data"][0]["url"].as_str().unwrap();
            println!("v:{}", v);
            Ok(image_url.to_string())
}
#[handler]
async fn open_image(event:Event)->Result<(), Box<dyn std::error::Error>>{
    if let Event::GroupMessage(ref msg) = event.clone() {
        if msg.start_with("/prompt") {
            let prompt = msg.message.replace("/prompt", "");
            let image_url = generate_image(prompt.as_str()).await?;
            let mut raw_msg = RowMessage::new();
            raw_msg.add_image(image_url.as_str());
            msg.reply(raw_msg.get_msg()).await?;
        }
    }
    Ok(())
}