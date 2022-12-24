use reqwest::ClientBuilder;
use rustqq::client::message::RowMessage;
use rustqq::event::events::Event;
use rustqq::event::events::Meassages;
use rustqq::event::reply_trait::Reply;
use rustqq::event::MsgEvent;
use rustqq::handler;
use scraper;

#[handler]
async fn echo_msg(event: &Event) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(e) = MsgEvent::new(event) {
        if e.start_with("echo") {
            let msg = e.msg().replace("echo", "");
            e.reply(msg.as_str()).await?;
        }
    }
    Ok(())
}
#[handler]
async fn url_preview(event: &Event) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(e) = MsgEvent::new(event) {
        let msg = e.msg();
        let regex = regex::Regex::new(
            r"^((ht|f)tps?)://[\w\-]+(\.[\w\-]+)+([\w\-\.,@?^=%&:/~\+#]*[\w\-@?^=%&/~\+#])?$",
        )
        .unwrap();
        if let Some(url) = regex.find(msg) {
            let url = url.as_str();
            let page_info = get_page_info(url).await?;
            e.reply(page_info.as_str()).await?;
        }
    }
    Ok(())
}
async fn get_page_info(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let res = ClientBuilder::new()
        .gzip(true)
        .build()?
        .get(url)
        .send()
        .await?
        .text()
        .await?;

    let document = scraper::Html::parse_document(&res);
    //get title fome
    let title_selectors = Vec::from([
        scraper::Selector::parse("meta[property=\"og:title\"]").unwrap(),
        scraper::Selector::parse("meta[name=title]").unwrap(),
        scraper::Selector::parse("title").unwrap(),
    ]);
    let title = title_selectors
        .iter()
        .map(|s| document.select(s).next())
        .find(|s| s.is_some())
        .unwrap()
        .unwrap()
        .value()
        .attr("content");
    let title = match title {
        Some(t) => t.to_string(),
        None => document
            .select(&title_selectors[2])
            .next()
            .unwrap()
            .text()
            .collect::<String>(),
    };

    let description = document
        .select(&scraper::Selector::parse("meta[name=description]").unwrap())
        .next();
    let description = match description {
        Some(d) => d.value().attr("content").unwrap(),
        None => "",
    };
    let mut description = description.to_string();
    if description.chars().count() > 100 {
        description = description
            .chars()
            .enumerate()
            .filter(|(i, _)| i < &100)
            .map(|(_, c)| c)
            .collect();
        description.push_str("...");
    }
    let image = document
        .select(&scraper::Selector::parse("meta[property=\"og:image\"]").unwrap())
        .next();
    let image = match image {
        Some(i) => i.value().attr("content").unwrap(),
        None => "",
    };
    let image = if image.starts_with("//") {
        format!("https:{}", image)
    } else {
        image.to_string()
    };
    let raw_msg = RowMessage::new()
        .add_plain_txt(title.as_str())
        .shift_line()
        .add_plain_txt("========================\n")
        .add_plain_txt(description.as_str())
        .shift_line()
        .add_image(image.as_str())
        .get_msg()
        .to_string();
    Ok(raw_msg)
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_get_page_info() {
        let url = "https://bilibili.com/video/BV1mP4y1D7Mu";
        let res = get_page_info(url).await.unwrap();
        println!("{}", res);

    }
    #[tokio::test]
    async fn tesr_xml(){
        let msg=r#"[CQ:xml,data=<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <msg serviceID="1">
            <item layout="4">
                <title>test title</title>
                <picture cover="http://url.cn/5CEwIUy"/>
            </item>
        </msg>]"#;
        let api=rustqq::client::api::SendGroupMessage::new(256658318,msg.to_string());
        if let Err(e)=api.post().await{
            println!("{}",e);
        }
    }
}
