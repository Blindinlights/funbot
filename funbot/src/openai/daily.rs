#![allow(dead_code)]
use rustqq::app::async_job::AsyncJob;

use async_openai::types::{
    ChatCompletionRequestMessage as Chat, CreateChatCompletionRequestArgs as CreateChatArgs,
};
const POEM_API: &str = "https://v1.jinrishici.com/all.json";
const PROMPT: &str = "请帮我把这个句子 `{sentence}` 翻译成英语，请翻译的有诗意一点儿。";
const DEFAULT_POEM: &str = "断虹霁雨，净秋空，山染修眉新绿。";
async fn get_poem() -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(POEM_API).await?.text().await?;
    let poem: serde_json::Value = serde_json::from_str(&resp)?;
    let poem = poem["content"].as_str().unwrap_or(DEFAULT_POEM);
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
    println!("{:#?}", res);
    let file_name = res["data"][0]["url"].as_str().unwrap();
    Ok(file_name.to_string())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let sentence = get_poem().await.unwrap_or(DEFAULT_POEM.to_string());
    let prompt = get_prompt(sentence.as_str()).await?;
    let url = generate_image(prompt.as_str()).await?;
    println!("{}", url);
    Ok(())
}
async fn job() {
    let _ = run().await;
}
pub fn daily() -> AsyncJob {
    AsyncJob::new("0 25 6 * * * *".parse().unwrap(), job)
}