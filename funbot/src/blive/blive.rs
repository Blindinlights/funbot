use mysql_async::prelude::*;
use rustqq::app::AsyncJob;
use rustqq::client::{api, message::RowMessage};
use rustqq::event::{Event, Meassages, Reply};
use rustqq::handler;

const URL: &str = "mysql://root:394755@localhost:3306/blive";
struct Vtuber {
    bid: String,
    name: String,
    group_id: String,
    live_status: bool,
}
struct LiveInfo {
    title: String,
    url: String,
    cover: String,
    group_id: String,
}
enum LiveStatus {
    Live(LiveInfo),
    Offline,
    Error,
}
async fn init_db() {
    let pool = mysql_async::Pool::new(URL);
    let conn = pool.get_conn().await.unwrap();
    //create table vtuber if not exists
    r"CREATE TABLE IF NOT EXISTS vtuber(
        bid VARCHAR(20),
        name VARCHAR(255),
        group_id VARCHAR(20),
        live_status BOOLEAN
    )"
    .ignore(conn)
    .await
    .unwrap();
}
async fn get_live_status(bid: &String, group_id: &String) -> LiveStatus {
    let url = format!("https://api.bilibili.com/x/space/acc/info?mid={}", bid);
    let client = reqwest::Client::new();
    //set user-agent
    let res = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:108.0) Gecko/20100101 Firefox/108.0",
        )
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let res: serde_json::Value = serde_json::from_str(&res).unwrap();
    //println!("{:?}", res);
    let live_status = res["data"]["live_room"]["liveStatus"].as_i64();

    if live_status == Some(1) {
        let title = res["data"]["live_room"]["title"].as_str().unwrap();
        let url = res["data"]["live_room"]["url"].as_str().unwrap();
        let cover = res["data"]["live_room"]["cover"].as_str().unwrap();
        LiveStatus::Live(LiveInfo {
            title: title.to_string(),
            url: url.to_string(),
            cover: cover.to_string(),
            group_id: group_id.to_string(),
        })
    } else if live_status == Some(0) || live_status == Some(2) {
        LiveStatus::Offline
    } else {
        println!("api error");
        LiveStatus::Error
    }
}
async fn get_vtuber_from_db() -> Vec<Vtuber> {
    let pool = mysql_async::Pool::new(URL);
    let conn = pool.get_conn().await.unwrap();
    let vtubers: Vec<Vtuber> = r"select * from vtuber"
        .with(())
        .map(conn, |(bid, name, group_id, live_status)| Vtuber {
            bid,
            name,
            group_id,
            live_status,
        })
        .await
        .unwrap();
    println!("Vtuber number:{}", vtubers.len());
    vtubers
}
async fn update_status() {
    init_db().await;
    let vtubers = get_vtuber_from_db().await;
    let pool = mysql_async::Pool::new(URL);
    let mut conn = pool.get_conn().await.unwrap();
    for vtuber in vtubers {
        let live_status: LiveStatus = get_live_status(&vtuber.bid, &vtuber.group_id).await;
        match live_status {
            LiveStatus::Live(ref live_info) => {
                if vtuber.live_status == false {
                    let mut msg = RowMessage::new();
                    msg.add_plain_txt(&vtuber.name)
                        .add_plain_txt("正在直播！")
                        .shift_line()
                        .add_plain_txt(&live_info.title)
                        .shift_line()
                        .add_plain_txt(&live_info.url)
                        .add_image(&live_info.cover);
                    let api = api::SendGroupMessage::new(
                        live_info.group_id.parse().unwrap(),
                        msg.get_msg().to_owned(),
                    );
                    api.post().await.unwrap();
                    //update status where bid = ? and group_id = ?
                    r"update vtuber set live_status = true where bid = ? and group_id = ?"
                        .with((vtuber.bid, vtuber.group_id))
                        .ignore(&mut conn)
                        .await
                        .unwrap();
                }
            }
            LiveStatus::Offline => {
                if vtuber.live_status == true {
                    //update status
                    r"update vtuber set live_status = false where bid = ? and group_id = ?"
                        .with((vtuber.bid, vtuber.group_id))
                        .ignore(&mut conn)
                        .await
                        .unwrap();
                }
            }
            LiveStatus::Error => println!("api error"),
        }
        //sleep 0.2s
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
}
async fn add_vtuber(bid: &String, group: &String) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.bilibili.com/x/space/acc/info?mid={}", bid);
    println!("{}", url);
    let client = reqwest::Client::new();
    //set user-agent
    let res = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:108.0) Gecko/20100101 Firefox/108.0",
        )
        .send()
        .await?
        .text()
        .await?;
    let res: serde_json::Value = serde_json::from_str(&res)?;
    //println!("{:?}",res);
    let name = res["data"]["name"].as_str();
    if name.is_none() {
        return Err("请求错误，或ID不存在".into());
    }
    let pool = mysql_async::Pool::new(URL);
    let mut conn = pool.get_conn().await?;
    //if vtuber not exists
    if r"select * from vtuber where bid = ? and group_id = ?"
        .with((bid.to_owned(), group.to_owned()))
        .map(&mut conn, |(bid, name, group_id, live_status)| Vtuber {
            bid,
            name,
            group_id,
            live_status,
        })
        .await?
        .len()
        == 0
    {
        r"insert into vtuber(bid,name,group_id,live_status) values(?,?,?,?)"
            .with((bid, name, group, false))
            .ignore(&mut conn)
            .await?;
    }

    Ok(())
}
async fn delete_vtuber(bid: &String, group: &String) -> Result<(), Box<dyn std::error::Error>> {
    let pool = mysql_async::Pool::new(URL);
    let mut conn = pool.get_conn().await?;
    r"delete from vtuber where bid = ? and group_id = ?"
        .with((bid, group))
        .ignore(&mut conn)
        .await?;
    Ok(())
}
pub fn blive_job() -> AsyncJob {
    AsyncJob::new("0 1/7 * * * * *".parse().unwrap(), update_status)
}
#[handler]
pub async fn add_live(event: rustqq::event::Event) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::GroupMessage(ref msg) = event.clone() {
        if msg.start_with("/addlive") {
            let bid = &msg.message.replace("/addlive", "").trim().to_string();
            if let Err(e) = bid.parse::<i64>() {
                let mut reply = RowMessage::new();
                reply.add_plain_txt("请输入正确格式");
                msg.reply(reply.get_msg()).await.unwrap();
                return Ok(());
            } else {
                add_vtuber(bid, &msg.group_id.to_string()).await?;
                let mut reply = RowMessage::new();
                reply.add_plain_txt("添加成功");
                msg.reply(reply.get_msg()).await.unwrap();
                return Ok(());
            }
        }
    }
    Ok(())
}
#[handler]
pub async fn delete_live(event: &Event) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::GroupMessage(ref msg) = event.clone() {
        if msg.start_with("/dellive") {
            let bid = &msg.message.replace("/dellive", "").trim().to_string();
            if let Err(e) = bid.parse::<i64>() {
                let mut reply = RowMessage::new();
                reply.add_plain_txt("请输入正确格式");
                msg.reply(reply.get_msg()).await?;
                return Ok(());
            } else {
                delete_vtuber(bid, &msg.group_id.to_string()).await?;
                let mut reply = RowMessage::new();
                reply.add_plain_txt("删除成功");
                msg.reply(reply.get_msg()).await?;
                return Ok(());
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_add_vtuber() {
        init_db().await;
        add_vtuber(&"17561885".to_string(), &"256658318".to_string())
            .await
            .unwrap();
    }
}
