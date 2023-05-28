use async_openai::types::{
    ChatCompletionRequestMessage as Chat, CreateChatCompletionRequestArgs as CreateChatArgs, Role,
};
use log::debug;
use regex::Regex;
use reqwest;
use rustqq::{
    client::message::RowMessage,
    command,
    event::{Event, Meassages, Reply},
    handler,
};
use serde_json::{json, Value};
use sqlx::{self, PgPool};
use std::{collections::HashMap, path::PathBuf};
use tiktoken_rs::async_openai::get_chat_completion_max_tokens as get_token;
pub const API_BASE: &str = "https://openai.clinkz.top/v1";
use super::tts;
#[allow(unused)]
type HandlerError = Box<dyn std::error::Error>;
async fn generate_image(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://api.openai.com/v1/images/generations";
    let mut map = HashMap::new();
    map.insert("prompt", prompt);
    map.insert("size", "256x256");
    let res = build_openai(url).json(&map).send().await?.text().await?;
    let v: Value = serde_json::from_str(&res)?;
    let image_url = v["data"][0]["url"].as_str().unwrap();
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
#[command(cmd = "/GPT4", desc = "GPT4聊天")]
pub async fn gpt4(msg_event: MsgEvent) -> Result<(), Box<dyn std::error::Error>> {
    let msg = msg_event.msg().trim_start_matches("/GPT4").trim();
    let res = gpt_4_chat(msg)
        .await
        .map_err(|e| {
            error!("{}", e);
            e
        })
        .ok()
        .unwrap_or("Failed to get response".to_string());
    msg_event.reply(&res).await?;
    Ok(())
}
#[handler]
pub async fn gpt_private(event: &Event, config: &Config) -> Result<(), HandlerError> {
    if let Event::PrivateMessage(ref e) = event {
        let user_id = e.user_id;
        if e.message.starts_with("[CQ:") {
            return Ok(());
        }
        if e.message.trim().starts_with('/') {
            return Ok(());
        }

        info!("{} ({}) 对ChatGPT说：{}", e.sender.nickname, e.user_id, e.msg());
        let user_id = e.user_id;
        let msg = e.msg();
        let res = private_chat(user_id, msg).await;

        if let Ok(res) = res {
            info!(target:"funbot","ChatGPT回复：{}", res);
            e.reply(&res).await?;
        } else {
            error!("对话失败：{:?}", res);
            priv_chat_update_in_use(user_id, false).await?;
            e.reply("哎呀，bot又犯糊涂了~ (｡•́︿•̀｡)\n请您再试一次或者联系管理员哦！")
                .await?;
        }
    }
    Ok(())
}

#[handler]
async fn gpt_group(event: &Event, config: &Config) -> Result<(), HandlerError> {
    if let Event::GroupMessage(ref e) = event {
        if let Some(prompt) = e.at_me() {
            info!("{} ({}) 对ChatGPT说：{}", e.sender.nickname, e.user_id, e.msg());
            let group_id = e.group_id;
            let nick_name = e.sender.nickname.clone();
            let res = group_chat(group_id, &prompt, &nick_name).await;
            if let Ok(res) = res {
                info!(target:"funbot","ChatGPT回复：{}", res);
                e.reply(&res).await?;
            } else {
                error!("对话失败：{:?}", res);
                e.reply("哎呀，bot又犯糊涂了~ (｡•́︿•̀｡)\n请您再试一次或者联系管理员哦！")
                    .await?;
            }
            return Ok(());
        }
    }

    Ok(())
}
#[handler]
pub async fn audio_gpt(event: &Event, config: &Config) -> Result<(), HandlerError> {
    if let Event::PrivateMessage(ref e) = event {
        let re = Regex::new(r"\[CQ:record,file=(.*?),url=.*\]").unwrap();
        if !re.is_match(e.msg()) {
            return Ok(());
        }
        let file_name = std::env::var("CQ_DATA").expect("CQ_DATA not set")
            + "/voices/"
            + re.captures(e.msg()).unwrap().get(1).unwrap().as_str();
        info!(target:"funbot","audio_gpt: {:?}", file_name);
        let input_path = PathBuf::from(&file_name);
        let output_path = input_path.with_extension("wav");
        tts::convert_audio_format(&input_path, &output_path)?;
        let prompt = tts::transcribe_audio(&output_path).await?;
        let res = private_chat(e.user_id, &prompt).await?;
        let audio = tts::text_to_speech(&res).await?;
        let file_name = audio.file_name().unwrap().to_str().unwrap();
        let msg = format!("[CQ:record,file=,url=file://{file_name}]");
        e.reply(&msg).await?;
    }

    Ok(())
}

async fn gpt_4_chat(msg: &str) -> anyhow::Result<String> {
    let url = "https://openai.clinkz.top/v1/chat/completions";
    let sys = Chat {
        role: Role::System,
        content: "you are a useful assistant.".to_string(),
        name: None,
    };
    let msg = Chat {
        role: Role::User,
        content: msg.to_string(),
        name: None,
    };
    let msg = vec![sys, msg];

    let res = build_openai(url)
        .json(&json!({
            "model":"gpt-4",
            "messages":msg,
            "max_tokens":300,
        }))
        .send()
        .await?
        .text()
        .await?;
    let response: Value = serde_json::from_str(&res)?;
    debug!(target:"funbot","{:?}", response);
    let res = response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or(anyhow::anyhow!("response error"))?
        .to_string();

    Ok(res)
}
async fn priv_chat_init(user_id: i64) -> anyhow::Result<()> {
    let pool = get_pgpool().await?;
    sqlx::query!(
        "INSERT INTO private_chat (user_id,system,history,in_use) 
        VALUES ($1,$2,$3,$4) 
        ON CONFLICT (user_id) 
        DO NOTHING",
        user_id,
        "You are a helpful assistant",
        "[]",
        false
    )
    .execute(&pool)
    .await?;
    Ok(())
}
async fn priv_chat_reset(user_id: i64) -> anyhow::Result<()> {
    let pool = get_pgpool().await?;
    sqlx::query!("COMMIT;",).execute(&pool).await?;
    sqlx::query!(
        "UPDATE private_chat SET system = $2, history = $3,in_use = $4 WHERE user_id = $1",
        user_id,
        "You are a helpful assistant",
        "[]",
        false
    )
    .execute(&pool)
    .await?;
    Ok(())
}
async fn priv_chat_update_history(user_id: i64, history: &str) -> anyhow::Result<()> {
    let pool = get_pgpool().await?;
    sqlx::query!(
        "UPDATE private_chat SET history = $2 WHERE user_id = $1",
        user_id,
        history
    )
    .execute(&pool)
    .await?;
    sqlx::query!("COMMIT;",).execute(&pool).await?;
    Ok(())
}
async fn priv_chat_update_system(user_id: i64, system: &str) -> anyhow::Result<()> {
    priv_chat_reset(user_id).await?;
    let pool = get_pgpool().await?;
    sqlx::query!(
        "UPDATE private_chat SET system = $2 WHERE user_id = $1",
        user_id,
        system
    )
    .execute(&pool)
    .await?;

    Ok(())
}
async fn priv_chat_update_in_use(user_id: i64, in_use: bool) -> anyhow::Result<()> {
    let pool = get_pgpool().await?;
    sqlx::query!(
        "UPDATE private_chat SET in_use = $2 WHERE user_id = $1",
        user_id,
        in_use
    )
    .execute(&pool)
    .await?;
    Ok(())
}

async fn group_chat_init(group_id: i64) -> anyhow::Result<()> {
    let pool = get_pgpool().await?;
    sqlx::query!(
        "INSERT INTO group_chat (group_id,system,history,in_use) 
        VALUES ($1,$2,$3,$4) 
        ON CONFLICT (group_id) 
        DO NOTHING",
        group_id,
        "You are a helpful assistant",
        "[]",
        false
    )
    .execute(&pool)
    .await?;
    Ok(())
}
async fn group_chat_reset(group_id: i64) -> anyhow::Result<()> {
    let pool = get_pgpool().await?;
    sqlx::query!("COMMIT;",).execute(&pool).await?;
    sqlx::query!(
        "UPDATE group_chat SET system = $2, history = $3,in_use = $4 WHERE group_id = $1",
        group_id,
        "You are a helpful assistant",
        "[]",
        false
    )
    .execute(&pool)
    .await?;
    Ok(())
}
async fn group_chat_update_history(group_id: i64, history: &str) -> anyhow::Result<()> {
    let pool = get_pgpool().await?;
    sqlx::query!(
        "UPDATE group_chat SET history = $2 WHERE group_id = $1;",
        group_id,
        history
    )
    .execute(&pool)
    .await?;
    sqlx::query!("COMMIT;",).execute(&pool).await?;
    Ok(())
}
async fn group_chat_update_system(group_id: i64, system: &str) -> anyhow::Result<()> {
    group_chat_reset(group_id).await?;
    let pool = get_pgpool().await?;
    sqlx::query!(
        "UPDATE group_chat SET system = $2 WHERE group_id = $1",
        group_id,
        system
    )
    .execute(&pool)
    .await?;
    Ok(())
}
async fn get_private_history(user_id: i64) -> anyhow::Result<Vec<Chat>> {
    let pool = get_pgpool().await?;
    sqlx::query!("BEGIN;",).execute(&pool).await?;
    let res = sqlx::query!(
        "SELECT history FROM private_chat WHERE user_id = $1",
        user_id
    )
    .fetch_one(&pool)
    .await;
    let history = if res.is_ok() {
        res.unwrap().history
    } else {
        "[]".to_string()
    };
    let history: Vec<Chat> = serde_json::from_str(&history)?;
    Ok(history)
}
async fn get_private_system(user_id: i64) -> anyhow::Result<String> {
    let pool = get_pgpool().await?;
    let res = sqlx::query!(
        "SELECT system FROM private_chat WHERE user_id = $1 ",
        user_id
    )
    .fetch_one(&pool)
    .await?;
    let system = res.system;
    Ok(system)
}
async fn get_group_history(group_id: i64) -> anyhow::Result<Vec<Chat>> {
    let pool = get_pgpool().await?;
    sqlx::query!("BEGIN;",).execute(&pool).await?;
    let res = sqlx::query!(
        "SELECT history FROM group_chat WHERE group_id = $1",
        group_id
    )
    .fetch_one(&pool)
    .await;
    let history = if res.is_ok() {
        res.unwrap().history
    } else {
        "[]".to_string()
    };
    let history: Vec<Chat> = serde_json::from_str(&history)?;
    Ok(history)
}
async fn get_group_system(group_id: i64) -> anyhow::Result<String> {
    let pool = get_pgpool().await?;
    let res = sqlx::query!(
        "SELECT system FROM group_chat WHERE group_id = $1",
        group_id
    )
    .fetch_one(&pool)
    .await?;
    let system = res.system;
    Ok(system)
}

async fn get_pgpool() -> anyhow::Result<PgPool> {
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&url).await.map_err(|e| {
        error!(target:"funbot","connect to database failed: {}", e);
        e
    })?;
    Ok(pool)
}

fn build_openai(url: &str) -> reqwest::RequestBuilder {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();
    reqwest::Client::builder()
        .build()
        .unwrap()
        .post(url)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
}

async fn gpt3(
    history: &mut Vec<Chat>,
    prompt: &str,
    system: &str,
    _nick_name: Option<String>,
) -> anyhow::Result<String> {
    let system = system.to_string();
    if history.is_empty() {
        history.push(Chat {
            role: Role::System,
            content: system.clone(),
            name: None,
        });
    }
    history.push(Chat {
        role: Role::User,
        content: prompt.to_string(),
        name: None,
    });
    check(history).await?;
    let arg = CreateChatArgs::default()
        .model("gpt-3.5-turbo")
        .messages(history.clone())
        .build()?;
    let response = async_openai::Client::new()
        .with_api_base(API_BASE)
        .chat()
        .create(arg)
        .await?;
    let usage = response.usage.unwrap();
    let (pt, ct) = (usage.prompt_tokens, usage.completion_tokens);

    let res = response.choices[0].message.content.clone();
    info!(target:"funbot","Usage:{} prompt and {} completion",pt,ct);
    history.push(Chat {
        role: Role::Assistant,
        content: res.clone(),
        name: None,
    });
    Ok(res)
}
async fn private_chat(user_id: i64, msg: &str) -> anyhow::Result<String> {
    priv_chat_init(user_id).await?;
    let mut history: Vec<Chat> = get_private_history(user_id).await?;
    let system = get_private_system(user_id).await?;
    let res = gpt3(&mut history, msg, &system, None).await;
    let prompt = serde_json::to_string(&history)?;
    priv_chat_update_history(user_id, &prompt).await?;
    let res = res?;
    Ok(res)
}
async fn group_chat(group_id: i64, msg: &str, nick_name: &str) -> anyhow::Result<String> {
    group_chat_init(group_id).await?;
    let mut history = get_group_history(group_id).await?;
    let system = get_group_system(group_id).await?;
    let res = gpt3(&mut history, msg, &system, Some(nick_name.to_string())).await;
    let prompt = serde_json::to_string(&history)?;

    group_chat_update_history(group_id, &prompt).await?;
    let res = res?;

    Ok(res)
}

async fn check(history: &mut Vec<Chat>) -> anyhow::Result<()> {
    let tokens = get_token("gpt-3.5-turbo", history)?;
    info!(target:"funbot","History token count:{}", 4097-tokens);
    if tokens > 0 {
        return Ok(());
    }

    loop {
        let token = get_token("gpt-3.5-turbo", history)?;
        if token > 0 {
            break;
        }
        history.remove(1);
    }
    Ok(())
}
#[command(cmd = "/gpt", desc = "聊天命令")]
pub async fn chat_set(msg_event: MsgEvent) -> Result<(), Box<dyn Error>> {
    let cmd = msg_event.msg().trim_start_matches("/gpt").trim();
    if cmd.starts_with("reset") {
        let user_id = msg_event.user_id();
        let group_id = msg_event.group_id();
        if msg_event.is_group() {
            group_chat_reset(group_id.unwrap()).await?;
        } else {
            priv_chat_reset(user_id).await?;
        }
        msg_event
            .reply("已经帮您重置了对话记录和system prompt啦！现在我们可以开始全新的对话啦!")
            .await?;
    } else {
        if cmd.starts_with("role") {
            let sys = cmd.trim_start_matches("role").trim();
            let user_id = msg_event.user_id();
            let group_id = msg_event.group_id();
            if msg_event.is_group() {
                group_chat_update_system(group_id.unwrap(), sys).await?;
            } else {
                priv_chat_update_system(user_id, sys).await?;
            }
            msg_event
                .reply("已经帮您更新了system prompt啦！现在我们可以开始全新的对话啦!")
                .await?;
        }
    }

    Ok(())
}
