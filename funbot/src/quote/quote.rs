use rustqq::event::reply_trait::Reply;
use rustqq::event::events::Meassages;
use rustqq::event::events::Event;
use rustqq::client::message::RowMessage;
use rustqq::handler;
use crate::quote::txt::TXTS;
#[handler]
pub async fn one_quote(event: Event) ->Result<(),Box<dyn std::error::Error>>{
    let url="https://api.xygeng.cn/one";
    if let Event::GroupMessage(e) =event{
        
        if e.eq("一言") {
            e.reply("正在查询中.....").await?;
            //reqwest
            let res = reqwest::get(url).await?;
            let text = res.text().await?;
            //println!("{}", text);
            //get feilds
            let json: serde_json::Value = serde_json::from_str(&text)?;
            let content=json["data"]["content"].as_str().unwrap();
            let origin=json["data"]["origin"].as_str().unwrap();
            let m=format!("{}\n    --{}",content,origin);

            let mut msg=RowMessage::new(&"".to_string());
            msg.add_plain_txt(&m);
            e.reply(msg.get_msg()).await?;
        }
    }

    Ok(())
}
#[handler]
pub async fn bing_pic(event: Event) ->Result<(),Box<dyn std::error::Error>>{
    let url="https://cn.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN";
    if let Event::GroupMessage(e) =event{
        
        if e.eq("壁纸") {
            e.reply("获取中.....").await?;
            //reqwest
            let res = reqwest::get(url).await;
            if let Err(err) =&res  {
                println!("error:{}",err);
                e.reply("获取失败").await?;
            }
            let text = res.unwrap().text().await?;
            //println!("{}", text);
            //get feilds
            let json: serde_json::Value = serde_json::from_str(&text)?;
            let url=json["images"][0]["url"].as_str().unwrap();
            let url=format!("https://cn.bing.com{}",url);
            let mut msg=RowMessage::new(&"".to_string());
            msg.add_image(&url);
            e.reply(msg.get_msg()).await?;
        }
    }

    Ok(())
}
#[handler]
pub async fn copy_paste(event: Event) ->Result<(),Box<dyn std::error::Error>>{
    if let Event::GroupMessage(e) =event{
        if e.eq("cv文学"){
            let index=rand::random::<usize>()%TXTS.len();
            let saying = TXTS[index];
            e.reply(saying).await?;
        }
    }  
    Ok(())
}