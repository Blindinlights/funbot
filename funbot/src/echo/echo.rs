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
            let msg=r#"[CQ:json,data={"app":"com.tencent.miniapp"&#44;"desc":""&#44;"view":"notification"&#44;"ver":"0.0.0.1"&#44;"prompt":"&#91;应用&#93;"&#44;"appID":""&#44;"sourceName":""&#44;"actionData":""&#44;"actionData_A":""&#44;"sourceUrl":""&#44;"meta":{"notification":{"appInfo":{"appName":"全国疫情数据统计"&#44;"appType":4&#44;"appid":1109659848&#44;"iconUrl":"http:\/\/gchat.qpic.cn\/gchatpic_new\/719328335\/-2010394141-6383A777BEB79B70B31CE250142D740F\/0"}&#44;"data":&#91;{"title":"确诊"&#44;"value":"80932"}&#44;{"title":"今日确诊"&#44;"value":"28"}&#44;{"title":"疑似"&#44;"value":"72"}&#44;{"title":"今日疑似"&#44;"value":"5"}&#44;{"title":"治愈"&#44;"value":"60197"}&#44;{"title":"今日治愈"&#44;"value":"1513"}&#44;{"title":"死亡"&#44;"value":"3140"}&#44;{"title":"今**亡"&#44;"value":"17"}&#93;&#44;"title":"中国加油，武汉加油"&#44;"button":&#91;{"name":"病毒：SARS-CoV-2，其导致疾病命名 COVID-19"&#44;"action":""}&#44;{"name":"传染源：新冠肺炎的患者。无症状感染者也可能成为传染源。"&#44;"action":""}&#93;&#44;"emphasis_keyword":""}}&#44;"text":""&#44;"sourceAd":""}]"#;
            e.reply(msg).await?;
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
    async fn test_json(){
        let msg=r#"[CQ:json,data={"app":"com.tencent.miniapp"&#44;"desc":""&#44;"view":"notification"&#44;"ver":"0.0.0.1"&#44;"prompt":"&#91;应用&#93;"&#44;"appID":""&#44;"sourceName":""&#44;"actionData":""&#44;"actionData_A":""&#44;"sourceUrl":""&#44;"meta":{"notification":{"appInfo":{"appName":"全国疫情数据统计"&#44;"appType":4&#44;"appid":1109659848&#44;"iconUrl":"http:\/\/gchat.qpic.cn\/gchatpic_new\/719328335\/-2010394141-6383A777BEB79B70B31CE250142D740F\/0"}&#44;"data":&#91;{"title":"确诊"&#44;"value":"80932"}&#44;{"title":"今日确诊"&#44;"value":"28"}&#44;{"title":"疑似"&#44;"value":"72"}&#44;{"title":"今日疑似"&#44;"value":"5"}&#44;{"title":"治愈"&#44;"value":"60197"}&#44;{"title":"今日治愈"&#44;"value":"1513"}&#44;{"title":"死亡"&#44;"value":"3140"}&#44;{"title":"今**亡"&#44;"value":"17"}&#93;&#44;"title":"中国加油，武汉加油"&#44;"button":&#91;{"name":"病毒：SARS-CoV-2，其导致疾病命名 COVID-19"&#44;"action":""}&#44;{"name":"传染源：新冠肺炎的患者。无症状感染者也可能成为传染源。"&#44;"action":""}&#93;&#44;"emphasis_keyword":""}}&#44;"text":""&#44;"sourceAd":""}]"#;
        let api=rustqq::client::api::SendGroupMessage::new(256658318, msg.to_string());
        api.post().await.unwrap();
    }
}
