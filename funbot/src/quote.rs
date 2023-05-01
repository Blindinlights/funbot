use log::error;
use rustqq::event::reply_trait::Reply;
use rustqq::event::events::Meassages;
use rustqq::event::events::Event;
use rustqq::client::message::RowMessage;
use rustqq::handler;
#[handler]
pub async fn bing_pic(event: Event) ->Result<(),Box<dyn std::error::Error>>{
    
    if let Event::GroupMessage(e) =event{
        
        if e.start_with("/bing_pic"){
            let cmd =e.message.split(' ').collect::<Vec<&str>>();
            let mut day=1;
            let cmd:Vec<&str>=cmd.into_iter().filter(|&x| !x.is_empty()).collect();
            if  cmd.len()<=1{
                e.reply("获取必应壁纸指令\n示例：\n \\bing_pic 1 \n最多获取近七天的壁纸").await?;
                return Ok(());
            }
            let parse=cmd[1].parse::<i32>();
            if parse.is_ok(){
                day=parse.unwrap();
            }
            let url=format!("https://cn.bing.com/HPImageArchive.aspx?format=js&idx={}&n=1&mkt=zh-CN", (day-1)%8);
            e.reply("获取中.....").await?;
            //reqwest
            let res = reqwest::get(url).await;
            if let Err(err) =&res  {
                error!(target:"funbot","error:{err}");
                e.reply("获取失败").await?;
            }
            let text = res.unwrap().text().await?;

  
            let json: serde_json::Value = serde_json::from_str(&text)?;
            let url=json["images"][0]["url"].as_str().unwrap();
            let title=json["images"][0]["title"].as_str().unwrap();
            let copyright=json["images"][0]["copyright"].as_str().unwrap();
            let url=format!("https://cn.bing.com{url}");
            let mut msg=RowMessage::new();
            //msg.add_plain_txt(&format!("{}\n{}\n",title,copyright));
            msg.add_plain_txt(title)
            .shift_line()
            .add_plain_txt("========================")
            .shift_line()
            .add_plain_txt(copyright)
            .shift_line();
            e.reply(msg.get_msg()).await?;
            msg.clear();
            msg.add_image(&url);
            e.reply(msg.get_msg()).await?;
        }
    }

    Ok(())
}
