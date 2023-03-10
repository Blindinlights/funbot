use anyhow;
use log::{debug, info, log};
use mysql_async::{
    self,
    prelude::{Query, Queryable, WithParams},
    Conn,
};
use reqwest;
use rustqq::{
    client::message::RowMessage,
    event::{Event, Meassages, MsgEvent, Reply},
    handler,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

async fn generate_image(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/images/generations";
    let mut map = HashMap::new();
    map.insert("prompt", prompt);
    map.insert("size", "256x256");
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
#[derive(Default, Serialize, Debug, Deserialize)]
struct Chat {
    role: String,
    content: String,
}

#[handler]
pub async fn chat(event: &Event, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(e) = MsgEvent::new(event) {
        // /gpt reset
        if e.eq("/gpt reset") {
            let pool = get_db()?;
            match e {
                MsgEvent::GroupMessage(e) => {
                    reset_content(0, e.group_id).await?;
                    e.reply("reset success").await?;
                    return Ok(());
                }
                MsgEvent::PrivateMessage(e) => {
                    reset_content(e.user_id, 0).await?;
                    e.reply("reset success").await?;
                    return Ok(());
                }
            }
        }
    }
    if let Event::PrivateMessage(ref e) = event {
        let msg = e.message.as_str();
        if config.is_command(msg) {
            return Ok(());
        }
        let ans = &chat_gpt(e.user_id, msg, 0)
            .await
            .unwrap_or("Token超过限制，记忆重置".to_owned());
        e.reply(ans).await?;
    };
    if let Event::GroupMessage(ref e) = event {
        let msg = e.message.as_str();
        if config.is_command(msg) {
            return Ok(());
        }
        let ans = &chat_gpt(0, msg, e.group_id)
            .await
            .unwrap_or("Token超过限制，记忆重置".to_owned());
        e.reply(ans).await?;
    }
    Ok(())
}
async fn chat_gpt(user_id: i64, prompt: &str, group_id: i64) -> anyhow::Result<String> {
    let pool = get_db()?;
    let mut conn = pool.get_conn().await?;
    init_database(&mut conn).await?;
    let mut context = {
        let res = get_content(user_id, group_id, &mut conn).await.unwrap();
        update_data(user_id, group_id, "", 1, &mut conn);
        res
    };
    let new_chat = Chat {
        role: "user".to_string(),
        content: prompt.to_string(),
    };
    context.push(new_chat);
    let data = json!({
        "model":"gpt-3.5-turbo",
        "messages":context
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
        update_data(user_id, group_id, "", 0, &mut conn).await?;
        return anyhow::Result::Err(anyhow::anyhow!("max token 4096"));
    }
    let role = v["choices"][0]["message"]["role"]
        .as_str()
        .unwrap_or("system");
    let new_chat = Chat {
        role: role.to_string(),
        content: ans.unwrap().to_string(),
    };
    context.push(new_chat);
    let new_chat = serde_json::to_string(&context)?;
    update_data(user_id, group_id, &new_chat, 0, &mut conn).await?;
    drop(conn);
    pool.disconnect().await?;
    let ans = ans.unwrap().to_string();
    debug!("GPT response: {}", &ans);
    Ok(ans)
}

async fn init_database(conn: &mut mysql_async::Conn) -> anyhow::Result<()> {
    let sql = r"CREATE TABLE IF NOT EXISTS theme(
        id INT NOT NULL primary key,
        name VARCHAR(255),
        desc VARCHAR(255),
        owner BIGINT NOT NULL,
        group bool NOT NULL,
        prompt TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS private_context(
        theme_id INT,
        id BIGINT NOT NULL PRIMARY KEY,
        content TEXT,
        pending INT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS group_context(
        theme_id INT ,
        id BIGINT NOT NULL PRIMARY KEY,
        content TEXT,
        pending INT NOT NULL
    );
    ";
    sql.ignore(conn).await?;
    Ok(())
}

async fn reset_content(user_id: i64, group_id: i64) -> anyhow::Result<()> {
    let pool = get_db()?;
    let mut conn = pool.get_conn().await?;
    init_database(&mut conn).await?;
    update_data(user_id, group_id, "", 0, &mut conn).await?;
    drop(conn);
    pool.disconnect().await?;
    Ok(())
}

async fn update_data(
    user_id: i64,
    group_id: i64,
    prompt: &str,
    pending: i32,
    conn: &mut mysql_async::Conn,
) -> anyhow::Result<()> {
    let table = if user_id != 0 {
        "private_context"
    } else {
        "group_context"
    };
    let sql = "INSERT INTO table VALUES(?,?,?,?) ON DUPLICATE KEY UPDATE content = ?,pending = ?";
    let sql = sql.replace("table", table);
    sql.with((0, user_id + group_id, prompt, pending, prompt, pending))
        .ignore(conn)
        .await
        .map_err(|e| {
            error!("{}", e);
            e
        })?;
    Ok(())
}

async fn update_theme(
    user_id: i64,
    group_id: i64,
    theme_id: i64,
    conn: &mut mysql_async::Conn,
) -> anyhow::Result<()> {
    let table = if user_id != 0 {
        "private_context"
    } else {
        "group_context"
    };
    todo!("get theme prompt from theme_id");
    todo!("update theme prompt to table");
    Ok(())
}

async fn get_content(user_id: i64, group_id: i64, conn: &mut Conn) -> anyhow::Result<Vec<Chat>> {
    let table = get_table_name(user_id, group_id);
    init_raw(user_id, group_id, conn).await?;
    let sql = format!(
        "SELECT theme_id,content FROM {} WHERE id = {} AND pending = 0",
        table,
        user_id + group_id
    );
    let res = loop {
        let res: Vec<(i32,String)> = conn.query(&sql).await.unwrap();
        if res.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            debug!("waiting for chatbot response...");
            continue;
        }
        break res;
    };
    let (theme_id,res) = res[0].clone();

    let res: Vec<Chat> = if res.is_empty() {

        let theme_prompt = get_theme_prompt().await?;
        let system_chat = Chat {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        };
        let theme_chat = Chat {
            role: "user".to_string(),
            content: theme_prompt,
        };
        vec![system_chat, theme_chat]
    } else {
        serde_json::from_str(&res).unwrap()
    };
    Ok(res)
}

async fn init_raw(user_id: i64, group_id: i64, conn: &mut Conn) -> anyhow::Result<()> {
    let table = get_table_name(user_id, group_id);
    let insert = format!("INSERT IGNORE INTO {} VALUES(?,?,?,?)", table);
    let error_sql = insert.clone();
    insert
        .with((0, user_id + group_id, "", 0))
        .ignore(conn)
        .await
        .map_err(|e| {
            error!("MySQL初始化行出现问题，sql:{}\n{}", error_sql, e);
            e
        })?;
    Ok(())
}
async fn get_theme_prompt() -> anyhow::Result<String> {
    todo!();
}
fn get_table_name(user_id: i64, group_id: i64) -> String {
    if user_id != 0 {
        "private_context".to_string()
    } else {
        "group_context".to_string()
    }
}
fn get_db() -> anyhow::Result<mysql_async::Pool> {
    let mut url = std::env::var("DATABASE_URL")?;
    url.push_str("chatgpt");
    let pool = mysql_async::Pool::new(url.as_str());
    Ok(pool)
}
