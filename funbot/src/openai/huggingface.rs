use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest;
use rustqq::{
    client::message::RowMessage,
    command,
    event::{Meassages, Reply}
};
use std::{io::Write, path};
pub struct HuggingFace {
    api_key: String,
    url: String,
}
impl HuggingFace {
    pub fn new(api_key: String, url: String) -> Self {
        Self { api_key, url }
    }
    pub async fn generate_image(
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
            .bearer_auth(self.api_key.as_str())
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

        HuggingFace::new(
            api_key,
            "https://api-inference.huggingface.co/models/prompthero/openjourney".to_string(),
        )
    }
    #[allow(dead_code)]
    pub fn sd_2_1() -> Self {
        let api_key = std::env::var("HUGGINGFACE_API_KEY").unwrap();

        HuggingFace::new(
            api_key,
            "https://api-inference.huggingface.co/models/stabilityai/stable-diffusion-2-1"
                .to_string(),
        )
    }
}
fn get_file_name() -> String {
    let mut file_name: String = thread_rng()
        .sample_iter(Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    file_name.push_str(".png");
    file_name
}
async fn reply_msg(
    prompt: String,
    msg_id: i64,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let hf = HuggingFace::open_journey();
    let file_name = get_file_name();
    let mut path = path::PathBuf::from("./");
    path = path.canonicalize().unwrap();
    path.push("images/");
    path.push(file_name);
    let path = path.to_str().unwrap().to_string();
    hf.generate_image(prompt.as_str(), path.clone()).await?;
    let path = "file://".to_string() + path.as_str();
    let mut raw_msg = RowMessage::new();
    raw_msg.reply(msg_id);
    raw_msg.add_image(&path);
    let path = path.replace("file://", "");
    Ok((raw_msg.get_msg().to_string(), path))
}
#[command(cmd = "/journey", desc = "使用huggingface的openjourney模型生成图片",alias="/openjourney|/oj")]
async fn open_journey(msg_event: _) -> Result<(), std::error::Error> {
    info!("openjourney");
    let prompt = msg_event.msg().replace("/journey", "");
    let (reply_msg, path) = reply_msg(prompt, msg_event.msg_id()).await?;
    msg_event.reply(&reply_msg).await?;
    std::fs::remove_file(path)?;

    Ok(())
}
