#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::Mutex;

use rustqq::app::async_job::AsyncJob;

use async_openai::types::{
    ChatCompletionRequestMessage as Chat, CreateChatCompletionRequestArgs as CreateChatArgs,
};
const POEM_API: &str = "https://v2.jinrishici.com/sentence";
const PROMPT: &str = "请帮我把这个句子 `{sentence}` 翻译成英语，请翻译的有诗意一点儿。";
const DEFAULT_POEM: &str = "断虹霁雨，净秋空，山染修眉新绿。";
async fn get_poem() -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(POEM_API).await?.text().await?;
    let poem: serde_json::Value = serde_json::from_str(&resp)?;
    let poem = poem["data"]["content"].as_str().unwrap_or(DEFAULT_POEM);
    Ok(poem.to_string())
}
async fn get_prompt(sentence: &str) -> Result<String, Box<dyn std::error::Error>> {
    let chat_prompt = PROMPT.replace("{sentence}", sentence);
    let chat_prompt = vec![Chat {
        content: chat_prompt,
        role: async_openai::types::Role::User,
        ..Default::default()
    }];
    let arg = CreateChatArgs::default()
        .model("gpt-3.5-turbo")
        .messages(chat_prompt)
        .build()?;
    let prompt = async_openai::Client::new()
        .chat()
        .create(arg)
        .await?
        .choices
        .first()
        .ok_or("no response")?
        .message
        .content
        .clone();
    Ok(format!("{}\n{}", prompt, "Chinese art style 4k"))
}
async fn generate_image(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://api.openai.com/v1/images/generations";
    let key = std::env::var("OPENAI_API_KEY").unwrap();
    let form = serde_json::json!({
        "prompt":prompt,
        "size":"512x512",
    });
    let client = reqwest::Client::new()
        .post(url)
        .bearer_auth(key.as_str())
        .json(&form)
        .send()
        .await?;
    let res = client.text().await?;
    let res: serde_json::Value = serde_json::from_str(&res)?;
    let file_name = res["data"][0]["url"].as_str().unwrap();
    Ok(file_name.to_string())
}
use rustqq::client::message::RowMessage;
use rustqq::event::{MsgEvent, Reply};
use rustqq::{command, CqHttpApi};
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let sentence = get_poem().await.unwrap_or(DEFAULT_POEM.to_string());
    let prompt = get_prompt(sentence.as_str()).await?;
    let url = generate_image(prompt.as_str()).await?;
    let api = CqHttpApi::new();
    let friend_list = api
        .get_friend_list()
        .await?
        .iter()
        .map(|x| x.user_id)
        .filter(|x| is_on(*x))
        .collect::<Vec<_>>();
    let msg_poem = RowMessage::new().text(sentence.as_str()).msg();
    let msg_image = RowMessage::new().image(url.as_str()).msg();

    for user_id in friend_list {
        api.send_private_message(user_id, &msg_poem).await?;
        api.send_private_message(user_id, &msg_image).await?;
    }

    Ok(())
}
async fn job() {
    let _ = run().await;
}
pub fn daily() -> AsyncJob {
    AsyncJob::new("0 30 8 * * * *".parse().unwrap(), job)
}
lazy_static::lazy_static! {
    pub static ref  CONFIG:Mutex<HashMap<i64,bool>>={
        let config = std::fs::read_to_string("config.toml").unwrap_or_else(
            |_|{
                let config = r#"
                [daily]
                "#;
                std::fs::write("config.toml",config).unwrap();
                info!("config.toml created");
                config.to_string()
            }
        );
        let config:toml::Value = toml::from_str(&config).unwrap();
        let mut map = HashMap::new();
        for (k,v) in config["daily"].as_table().unwrap(){
            map.insert(k.parse().unwrap(),v.as_bool().unwrap());
        }
        Mutex::new(map)
    };

}
fn is_on(user_id: i64) -> bool {
    CONFIG
        .lock()
        .unwrap()
        .get(&user_id)
        .unwrap_or(&false)
        .clone()
}
fn update(user_id: i64, flag: bool) {
    CONFIG.lock().unwrap().insert(user_id, flag);
    let config = std::fs::read_to_string("config.toml").unwrap();
    let mut config: toml::Value = toml::from_str(&config).unwrap();
    config["daily"]
        .as_table_mut()
        .unwrap()
        .insert(user_id.to_string(), toml::Value::Boolean(flag));
    let config = toml::to_string_pretty(&config).unwrap();
    std::fs::write("config.toml", config).unwrap();
}

#[command(
    name = "每日一句",
    cmd = "/daily",
    desc = "每天早上8点半发送一句诗句与配图,用/daily on|off开启关闭"
)]
async fn daily_cmd(msg_event: MsgEvent) -> Result<(), Box<dyn std::error::Error>> {
    if let MsgEvent::PrivateMessage(e) = msg_event {
        info!("设置每日诗句");
        let user_id = e.user_id;
        let flag = e.raw_message.contains("on");
        update(user_id, flag);
        let msg = if flag {
            "每日诗句已开启"
        } else {
            "每日诗句已关闭"
        };
        e.reply(msg).await.map_err(|e| {
            error!("回复失败:{}", e);
            e
        })?;
    }
    Ok(())
}
