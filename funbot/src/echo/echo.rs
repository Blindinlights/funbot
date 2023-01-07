use reqwest::ClientBuilder;
use rustqq::client::message::RowMessage;
use rustqq::event::events::Event;
use rustqq::event::events::Meassages;
use rustqq::event::reply_trait::Reply;
use rustqq::event::MsgEvent;
use rustqq::handler;
use scraper;
use tokio::fs;

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


#[handler]
pub async fn emoji_mix(event: &Event)->Result<(),Box<dyn std::error::Error>>{
    if let Some(e) =MsgEvent::new(event)  {
        let re=regex::Regex::new(r"^\\(?:(\p{Emoji})\p{Emoji_Modifier}?(?:\p{Emoji_Component}\p{Emoji}\p{Emoji_Modifier}?)*)\+(?:(\p{Emoji})\p{Emoji_Modifier}?(?:\p{Emoji_Component}\p{Emoji}\p{Emoji_Modifier}?)*)$")?;
        if !re.is_match(e.msg()){
            return Ok(());
        }
        let msg=e.msg();
        let cap=re.captures(msg).unwrap();
        let (left,right)=(cap.get(1).unwrap().as_str(),cap.get(2).unwrap().as_str());
        let left=left.chars().next().unwrap() as u32;
        let right=right.chars().next().unwrap() as u32;
        let (left,right)=(format!("{:x}",left),format!("{:x}",right));
        let date=get_date(&left, &right).await?;
        let root_url="https://www.gstatic.com/android/keyboard/emojikitchen";
        let url=format!("{root_url}/{date}/u{left}/u{left}_u{right}.png");
        let mut rmsg=RowMessage::new();
        rmsg.add_image(url.as_str());
        e.reply(rmsg.get_msg()).await?;
    }
    Ok(())
}

async fn get_date(left: &str, right: &str) -> Result<String,Box<dyn std::error::Error>> {
    let data=fs::read_to_string("/home/blindinlights/EmojiData.json").await.expect("Fail to open emoji-data.json ");
    let v: serde_json::Value = serde_json::from_str(&data)?;
    //get every key name
    let map = v.as_object().unwrap();
    let mut result = None;
    let de=&serde_json::Value::default();
    let lefts = map.get(&left.to_string()).unwrap_or(de).as_array().expect("No Matched emoji!");
    for entry in lefts.iter() {
        if entry["leftEmoji"].as_str().expect("No matched emoji!") == left.to_string() {
            result = Some(entry["date"].as_str().unwrap().to_string());
        }
    }
    if result.is_none() {
        let rights = map.get(&right.to_string()).unwrap_or(de).as_array().expect("No matched emoji!");
        for entry in rights.iter() {
            if entry["rightEmoji"].as_str().expect("No matched emoji!") == right.to_string() {
                result = Some(entry["date"].as_str().unwrap().to_string());
            }
        }
    }

    Ok(result.unwrap())
}
#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_get_date() {
        let (left, right) = ("🥹", "😯");
        let left = left.chars().next().unwrap() as u32;
        let right = right.chars().next().unwrap() as u32;
        let (left, right) = (format!("{:x}", left), format!("{:x}", right));
        println!("{} {}", left, right);
        let date = get_date(left.as_str(), right.as_str()).await.unwrap();
        println!("https://www.gstatic.com/android/keyboard/emojikitchen/{date}/u{left}/u{left}_u{right}.png")
    }
}