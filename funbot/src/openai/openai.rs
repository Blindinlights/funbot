use mysql_async::prelude::{Query, WithParams};
use reqwest;
use rustqq::{
    client::message::RowMessage,
    event::{Event, Meassages, Reply},
    handler,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[allow(unused)]
use std::collections::HashMap;
async fn generate_image(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/images/generations";
    let mut map = HashMap::new();
    map.insert("prompt", prompt);
    //map.insert("n", "1");
    map.insert("size", "256x256");

    //add header "Content-Type: application/json""Authorization: Bearer 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"
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let api_key = "Bearer {}".replace("{}", &api_key);
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", api_key)
        .json(&map)
        .send()
        .await?
        .text()
        .await?;
    let v: Value = serde_json::from_str(&res)?;
    let image_url = v["data"][0]["url"].as_str().unwrap();
    println!("v:{v}");
    Ok(image_url.to_string())
}
#[handler]
async fn open_image(event: Event) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::GroupMessage(ref msg) = event {
        if msg.start_with("/prompt") {
            let prompt = msg.message.replace("/prompt", "");
            let image_url = generate_image(prompt.as_str()).await?;
            let mut raw_msg = RowMessage::new();
            raw_msg.reply(msg.message_id);
            raw_msg.add_image(image_url.as_str());
            msg.reply(raw_msg.get_msg()).await?;
            return Ok(());
        }
    }
    if let Event::PrivateMessage(ref msg) = event {
        if msg.start_with("/prompt") {
            let prompt = msg.message.replace("/prompt", "");
            let image_url = generate_image(prompt.as_str()).await?;
            let mut raw_msg = RowMessage::new();
            raw_msg.reply(msg.message_id);
            raw_msg.add_image(image_url.as_str());
            msg.reply(raw_msg.get_msg()).await?;
            return Ok(());
        }
    }
    Ok(())
}
const SYSTEM_PROMPT: &str = "You are a helpful assistant";
#[derive(Default, Serialize, Debug)]
struct Chat {
    role: String,
    content: String,
}

#[handler]
pub async fn chat(event: Event) -> Result<(), Box<dyn std::error::Error>> {
    chat_gpt(1, "prompt");
    todo!()
}
async fn chat_gpt(user_id: i64, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    init_database().await;
    let mut context = get_context_private(user_id).await.unwrap_or_default();
    let new_chat = Chat {
        role: "user".to_string(),
        content: prompt.to_string(),
    };
    save_context_private("new_chat", user_id).await;
    context.push(new_chat);
    let data = json!({
        "model":"gpt-3.5-turbo",
        "message":context
    });
    let url = "https://api.openai.com/v1/chat/completions";
    let client = reqwest::Client::new();
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let auth = "Bearer {}".replace("{}", &api_key);
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", auth)
        .json(&data)
        .send()
        .await?
        .text()
        .await?;
    let v: Value = serde_json::from_str(&res)?;
    let ans = v["choices"][0]["message"]["content"].as_str();
    if ans.is_none() {
        return Err("unkown error".into());
    }
    let role = v["choices"][0]["message"]["role"]
        .as_str()
        .unwrap_or("system");
    let new_chat = Chat {
        role: role.to_string(),
        content: ans.unwrap().to_string(),
    };
    save_context_private("", user_id).await;
    Ok(ans.unwrap().to_string())
}

async fn init_database() {
    let pool = mysql_async::Pool::new("");
    let conn = pool.get_conn().await.unwrap();
    r"CREATE TABLE IF NOT EXISTS private(
        id VARCHAR(20) NOT NULL,
        content TEXT ,
    )"
    .ignore(conn)
    .await
    .unwrap();
}
async fn save_context_private(new_chat: &str, user_id: i64) {
    todo!()
}
async fn get_context_private(user_id: i64) -> Result<Vec<Chat>, Box<dyn std::error::Error>> {
    todo!()
}
