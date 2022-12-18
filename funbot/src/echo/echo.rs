use reqwest::Client;
use reqwest::ClientBuilder;
use rustqq::client::message::RowMessage;
use rustqq::event::events::Event;
use rustqq::event::events::Meassages;
use rustqq::event::reply_trait::Reply;
use rustqq::event::MsgEvent;
use rustqq::handler;
use scraper;
use serde_json::json;

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
pub async fn github_url_preview(event: Event) -> Result<(), Box<dyn std::error::Error>> {
    let _url="https://opengraph.githubassets.com/3ce26901f1f7120dd7eb84e7e7bdcb82210d183ab7270db802a74b9eb32109db/";
    if let Event::GroupMessage(e) = event.clone() {
        if e.start_with("https://github.com/") {
            let msg = e.message.clone();
            let data = json!({ "repo_url": msg });
            println!("Start post");
            let res = Client::new()
                .post("http://127.0.0.1:5000/github_repo")
                .json(&data)
                .send()
                .await?;
            println!("ok");
            let json = res.json::<serde_json::Value>().await?;
            let title = json["title"].as_str().unwrap();
            let description = json["description"].as_str().unwrap();
            let image = json["image"].as_str().unwrap();

            //msg = msg.replace("https://github.com/", url);
            //println!("{}", msg);
            let mut re = RowMessage::new();
            re.add_plain_txt(title);
            re.shift_line();
            //if !description.contains(title) {
            re.add_plain_txt("========================\n");
            re.add_plain_txt(description);
            re.shift_line();
            //}
            re.add_image(image);
            //re.add_image(msg.as_str());
            e.reply(re.get_msg()).await?;
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
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap();
    let mut description=description.to_string();
    if description.chars().count()>100{
        description=description.chars().enumerate().filter(|(i,_)|i<&100).map(|(_,c)|c).collect();
        description.push_str("...");
    }
    let image = document
        .select(&scraper::Selector::parse("meta[property=\"og:image\"]").unwrap())
        .next();
    let image = match image {
        Some(i) => i.value().attr("content").unwrap(),
        None => "",
    };

    let raw_msg = RowMessage::new()
        .add_plain_txt(title.as_str())
        .shift_line()
        .add_plain_txt("========================\n")
        .add_plain_txt(description.as_str())
        .shift_line()
        .add_image(image)
        .get_msg()
        .to_string();
    Ok(raw_msg)
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_get_page_info() {
        let url = "https://www.zhihu.com/hot";
        let res = get_page_info(url).await.unwrap();
        println!("{}", res);
    }
}
