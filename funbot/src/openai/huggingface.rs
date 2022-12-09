use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest;
use rustqq::{
    client::message::RowMessage,
    event::{self, Event, Meassages, Reply},
    handler,
};
use serde_json::Value;
use std::{collections::HashMap, io::Write, path, string};
struct HuggingFace {
    api_key: String,
    url: String,
}
impl HuggingFace {
    pub fn new(api_key: String, url: String) -> Self {
        Self { api_key, url }
    }
    async fn generate_image(
        &self,
        prompt: &str,
        file_name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = self.url.as_str();
        let data = serde_json::json!({
            "inputs": prompt,
            "options":{
                "use_cache": false,
                "wait_for_model": true,
            }
        });
        //add header "Content-Type: application/json""Authorization : bear
        let res = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", self.api_key.as_str())
            .json(&data)
            .send()
            .await?
            .bytes()
            .await?;
        let mut file = std::fs::File::create(file_name)?;
        file.write_all(&res)?;
        Ok(())
    }
    pub fn open_journey() -> Self {
        let api_key = std::env::var("HUGGINGFACE_API_KEY").unwrap();
        let hf = HuggingFace::new(
            format!("Bearer {}", api_key),
            "https://api-inference.huggingface.co/models/prompthero/openjourney".to_string(),
        );
        hf
    }
}
fn get_file_name() -> String {
    let mut file_name: String = thread_rng()
        .sample_iter(Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    file_name.push_str(".png");
    println!("file_name:{}", file_name);
    file_name
}
async fn reply_msg(prompt: String, msg_id: i64) -> Result<(String,String), Box<dyn std::error::Error>> {
    let hf = HuggingFace::open_journey();
    let mut file_name = get_file_name();
    let mut path = path::PathBuf::from("./");
    path = path.canonicalize().unwrap();
    path.push("funbot/src/images/");
    path.push(file_name);
    let path = path.to_str().unwrap().to_string();
    hf.generate_image(prompt.as_str(), path.clone()).await?;
    let path = "file://".to_string() + path.as_str();
    let mut raw_msg = RowMessage::new();
    raw_msg.reply(msg_id);
    raw_msg.add_image(&path);
    Ok((raw_msg.get_msg().to_string(),path))
}

#[handler]
async fn open_journey(event: Event) -> Result<(), std::error::Error> {
    if let Event::GroupMessage(ref msg) = event.clone() {
        if msg.start_with("/journey") {
            let prompt = msg.message.replace("/journey", "");
            let (reply_msg,path) = reply_msg(prompt, msg.message_id).await?;
            msg.reply(&reply_msg).await?;
            std::fs::remove_file(path)?;
        }
    }
    if let Event::PrivateMessage(ref msg) = event.clone() {
        if msg.start_with("/journey") {
            let prompt = msg.message.replace("/journey", "");
            let (reply_msg,path) = reply_msg(prompt, msg.message_id).await?;
            msg.reply(&reply_msg).await?;
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_open_journey() {
        let hf = HuggingFace::open_journey();
        let mut file_name = get_file_name();
        let mut path = path::PathBuf::from("./");
        path = path.canonicalize().unwrap();
        path.push("src/images/");
        path.push(file_name);
        let path = path.to_str().unwrap().to_string();
        println!("path:{}", path);
        hf.generate_image("我是一个人", path.clone()).await.unwrap();
        let path = "file://".to_string() + path.as_str();
        let mut raw_msg = RowMessage::new();
        raw_msg.add_image(&path);
        println!("{}", raw_msg.get_msg());
    }
}
