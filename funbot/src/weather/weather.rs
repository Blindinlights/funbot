use reqwest;
use rustqq::client::message::RowMessage;
use rustqq::event::*;
use rustqq::handler;
#[handler]
async fn weather_query(event: Event) -> Result<(),Box<dyn std::error::Error>> {
    let url="https://restapi.amap.com/v3/weather/weatherInfo?city=610116&key=a5499bb69bcd91946805294b372d437c";

    if let Event::GroupMessage(e) = event {
        println!("weather mod");
        if e.start_with("天气查询") {
                e.reply("正在查询中.....").await?;
                //reqwest
                let res = reqwest::get(url).await?;
                let text = res.text().await?;

                //println!("{}", text);
                //get feilds
                let json: serde_json::Value = serde_json::from_str(&text)?;
                let province = json["lives"][0]["province"].as_str().unwrap();
                let city = json["lives"][0]["city"].as_str().unwrap();
                let weather = json["lives"][0]["weather"].as_str().unwrap();
                let temperature = json["lives"][0]["temperature"].as_str().unwrap();
                let winddirection = json["lives"][0]["winddirection"].as_str().unwrap();
                let windpower = json["lives"][0]["windpower"].as_str().unwrap();
                let humidity = json["lives"][0]["humidity"].as_str().unwrap();
                let reporttime = json["lives"][0]["reporttime"].as_str().unwrap();
                //format msg
                let msg = format!(
                    "{province} {city}\n-------------------------------------\n\
                    天气：{weather}\n\
                    温度：{temperature}°C\n\
                    风向：{winddirection}\n\
                    风力：{windpower}级\n\
                    湿度：{humidity}%\n\
                    更新时间：{reporttime}"
                );
                let mut pic = RowMessage::new();
                //add picture
                //pic.add_image("https://static.zhihu.com/heifetz/assets/guide-cover-5.294257c3.jpg");
                pic.add_plain_txt(&msg);

                //reply msg;
                e.reply(pic.get_msg()).await?;
        }
    }
    Ok(())
}
#[handler]
async fn weather_report(event: Event) {
    let url="https://restapi.amap.com/v3/weather/weatherInfo?extensions=all&city=610116&key=a5499bb69bcd91946805294b372d437c";
    if let Event::GroupMessage(e) = event {
        if e.start_with("天气预报") {
            e.reply("正在查询中.....").await?;
            //reqwest
            let res = reqwest::get(url).await?;

            let text = res.text().await?;
            let json: serde_json::Value = serde_json::from_str(&text)?;

            let province = json["forecasts"][0]["province"].as_str().unwrap();
            let city = json["forecasts"][0]["city"].as_str().unwrap();

            let reporttime = json["forecasts"][0]["reporttime"].as_str().unwrap();
            let casts = json["forecasts"][0]["casts"].as_array().unwrap();
            let date = casts[1]["date"].as_str().unwrap();
            let mut week = casts[1]["week"].as_str().unwrap();
            let dayweather = casts[1]["dayweather"].as_str().unwrap();
            let nightweather = casts[1]["nightweather"].as_str().unwrap();
            let daytemp = casts[1]["daytemp"].as_str().unwrap();
            let nighttemp = casts[1]["nighttemp"].as_str().unwrap();
            let daywind = casts[1]["daywind"].as_str().unwrap();
            let nightwind = casts[1]["nightwind"].as_str().unwrap();
            let daypower = casts[1]["daypower"].as_str().unwrap();

            week = match week {
                "1" => "星期一",
                "2" => "星期二",
                "3" => "星期三",
                "4" => "星期四",
                "5" => "星期五",
                "6" => "星期六",
                "7" => "星期日",
                _ => "未知",
            };
            let nightpower = casts[1]["nightpower"].as_str().unwrap();
            let msg = format!(
                "{province} {city}\n-------------------------------------\n\
                日期：{date} {week}\n\
                ====白天====\n\
                天气：{dayweather}\n\
                温度：{daytemp}°C\n\
                风向：{daywind}\n\
                风力：{daypower}级\n\n\
                ====夜间====\n\
                天气：{nightweather}\n\
                温度：{nighttemp}°C\n\
                风向：{nightwind}\n\
                风力：{nightpower}级\n\
                更新时间：{reporttime}"
            );
            e.reply(msg.as_str()).await?;
        }
    }
    Ok(())
}
