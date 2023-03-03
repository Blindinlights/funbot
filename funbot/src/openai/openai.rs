use mysql_async::{
    self,
    prelude::{Query, Queryable},
    Conn,
};
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
#[derive(Default, Serialize, Debug, Deserialize)]
struct Chat {
    role: String,
    content: String,
}

#[handler]
pub async fn chat(event: &Event, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::PrivateMessage(ref e) = event {
        let msg = e.message.as_str();
        if config.is_command(msg) {
            return Ok(());
        }
        let ans = &chat_gpt(e.user_id, msg, 0).await?;
        e.reply(ans).await?;
    };

    todo!()
}
async fn chat_gpt(
    user_id: i64,
    prompt: &str,
    group_id: i64,
) -> Result<String, Box<dyn std::error::Error>> {
    let pool = get_db()?;
    let mut conn = pool.get_conn().await?;
    init_database(&mut conn).await?;

    let mut context = if user_id != 0 {
        get_context_private(user_id, &mut conn)
            .await
            .unwrap_or_default()
    } else {
        get_context_group(group_id, &mut conn)
            .await
            .unwrap_or_default()
    };
    let new_chat = Chat {
        role: "user".to_string(),
        content: prompt.to_string(),
    };
    context.push(new_chat);
    let data = json!({
        "model":"gpt-3.5-turbo",
        "message":context
    });
    let new_chat = serde_json::to_string(&context)?;
    if user_id != 0 {
        update_context_private(&new_chat, user_id, 1, &mut conn).await?;
    } else {
        update_context_group(&new_chat, group_id, 1, &mut conn).await?;
    }
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
    context.push(new_chat);
    //json str
    let new_chat = serde_json::to_string(&context)?;
    if user_id != 0 {
        update_context_private(&new_chat, user_id, 0, &mut conn).await?;
    } else {
        update_context_group(&new_chat, group_id, 0, &mut conn).await?;
    }
    Ok(ans.unwrap().to_string())
}

async fn init_database(conn: &mut mysql_async::Conn) -> Result<(), Box<dyn std::error::Error>> {
    let sql = r"CREATE TABLE IF NOT EXISTS theme(
        id INT NOT NULL AUTO_INCREMENT,
        prompt TEXT NOT NULL,
    );
    CREATE TABLE IF NOT EXISTS private_context(
        id INT NOT NULL AUTO_INCREMENT,
        theme_id INT,
        user_id INT NOT NULL PRIMARY KEY,
        content TEXT,
        pending INT NOT NULL,
    );
    CREATE TABLE IF NOT EXISTS group_context(
        id INT NOT NULL AUTO_INCREMENT,
        theme_id INT ,
        group_id INT NOT NULL PRIMARY KEY,
        content TEXT,
        pending INT NOT NULL,
    );
    ";
    sql.ignore(conn).await?;
    Ok(())
}
async fn update_context_private(
    new_chat: &str,
    user_id: i64,
    pending: i32,
    conn: &mut mysql_async::Conn,
) -> Result<(), Box<dyn std::error::Error>> {
    let sql = format!(
        "UPDATE private_context SET content = '{}',pending = {} WHERE user_id = {}",
        new_chat, user_id, pending
    );
    sql.ignore(conn).await?;
    Ok(())
}
async fn update_context_group(
    new_chat: &str,
    group_id: i64,
    pending: i32,
    conn: &mut mysql_async::Conn,
) -> Result<(), Box<dyn std::error::Error>> {
    let sql = format!(
        "UPDATE private_context SET content = '{}',pending = {} WHERE user_id = {}",
        new_chat, group_id, pending
    );
    sql.ignore(conn).await?;
    Ok(())
}

async fn get_context_private(
    user_id: i64,
    conn: &mut Conn,
) -> Result<Vec<Chat>, Box<dyn std::error::Error>> {
    insert_context_private(user_id, "default", "", conn).await?;
    let sql = format!(
        "SELECT content,pending FROM private_context WHERE user_id = {}",
        user_id
    );
    let mut res: Vec<(String, i32)> = Vec::new();
    let mut pending = 1;
    while pending == 1 {
        res = conn.query(sql.as_str()).await?;
        pending = res[0].1;
    }
    let res = res[0].0.clone();
    let res: Vec<Chat> = serde_json::from_str(&res)?;
    Ok(res)
}
async fn insert_context_private(
    user_id: i64,
    theme: &str,
    content: &str,
    conn: &mut Conn,
) -> Result<(), Box<dyn std::error::Error>> {
    let sql = format!(
        "INSERT IGNORE INTO private_context VALUES({},\"{}\", \"{}\", 1)",
        user_id, theme, content
    );
    sql.ignore(conn).await?;
    Ok(())
}
async fn insert_context_group(
    group_id: i64,
    theme: &str,
    content: &str,
    conn: &mut Conn,
) -> Result<(), Box<dyn std::error::Error>> {
    let sql = format!(
        "INSERT IGNORE INTO group_context VALUES({},\"{}\", \"{}\", 0)",
        group_id, theme, content
    );
    sql.ignore(conn).await?;
    Ok(())
}
async fn get_context_group(
    group_id: i64,
    conn: &mut Conn,
) -> Result<Vec<Chat>, Box<dyn std::error::Error>> {
    insert_context_group(group_id, "default", "", conn).await?;
    let sql = format!(
        "SELECT content,pending FROM group_context WHERE user_id = {}",
        group_id
    );
    let mut res: Vec<(String, i32)> = Vec::new();
    let mut pending = 1;
    while pending == 1 {
        res = conn.query(sql.as_str()).await?;
        pending = res[0].1;
    }
    let res = res[0].0.clone();
    let res: Vec<Chat> = serde_json::from_str(&res)?;
    Ok(res)
}
fn get_db() -> Result<mysql_async::Pool, Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")?;
    let pool = mysql_async::Pool::new(url.as_str());
    Ok(pool)
}
