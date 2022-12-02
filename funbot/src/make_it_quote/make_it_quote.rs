use image::{self, GenericImageView};
use imageproc::drawing;
use regex::{self, Regex};
use reqwest;
use rustqq::client;
use rustqq::client::message::RowMessage;
use rustqq::event::events::Event;
use rustqq::event::reply_trait::Reply;
use rustqq::handler;
use rusttype::{Font, Scale};
use std::path;
#[handler]
pub async fn make_it_quote(event: Event) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::GroupMessage(ref msg) = event.clone() {
        //match [CQ:reply,id={id}] <msg>
        let re = regex::Regex::new(r"\[CQ:reply,id=(-?\d+)\]\[CQ:.*]* (.*)").unwrap();
        println!("make-it-quote");
        println!("{:?}", msg.message);
        if let Some(e) = re.captures(msg.message.as_str()) {
            let id = e.get(1).unwrap().as_str().parse::<i64>()?;
            println!("id:{}", id);
            //get last cap
            let cmd = e.get(2).unwrap().as_str();
            println!("cmd:{}", cmd);

            if cmd.starts_with("make-it-quote") {
                let api = client::api::GetMessage::new(id);
                let res = api.post().await?;
                let re=Regex::new(r"(\[CQ:.*\])").unwrap();

                let init_msg = res["data"]["message"].as_str().unwrap();
                if let Some(_)=re.captures(init_msg){
                    msg.reply("只能引用纯文字").await?;
                    return Err("只能引用纯文字".into());
                }
                let nick_name = res["data"]["sender"]["nickname"].as_str().unwrap();
                let sender_id = res["data"]["sender"]["user_id"].as_i64().unwrap();
                get_quote(sender_id, init_msg, nick_name).await?;
                println!("make-it-quote");

                let mut raw_msg = RowMessage::new();
                //get absolute path
                let mut path = path::PathBuf::from("./");
                path = path.canonicalize()?;
                path.push("funbot/src/images/temp.png");
                let path = path.to_str().unwrap();
                let path = "file://".to_owned() + path;
                raw_msg.add_image(path.as_str());
                msg.reply(raw_msg.get_msg()).await?;
            }
        } else {
            println!("no match");
        }
    }
    Ok(())
}
async fn get_quote(qq: i64, msg: &str, nick_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("qq:{}", qq);
    let url = format!("https://q1.qlogo.cn/g?b=qq&nk={}&s=640", qq);
    println!("url:{}", url);
    let buf = reqwest::get(url).await?.bytes().await?.to_vec();
    let img = image::load_from_memory(&buf)?;
    let mut canvas = image::RgbaImage::new(1280, 640); // Create a canvas
    let mut avg_color = img.pixels().map(|p| p.2).fold((0, 0, 0), |(r, g, b), p| {
        (r + p[0] as u32, g + p[1] as u32, b + p[2] as u32)
    });
    avg_color = (
        avg_color.0 / (img.width() * img.height()) as u32,
        avg_color.1 / (img.width() * img.height()) as u32,
        avg_color.2 / (img.width() * img.height()) as u32,
    );
    let font_color = image::Rgba([
        255 - avg_color.0 as u8,
        255 - avg_color.1 as u8,
        255 - avg_color.2 as u8,
        255,
    ]);
    let font_data = include_bytes!("../fonts/YeZiGongChangShanHaiMingChao-2.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).unwrap();
    let scale = Scale { x: 40.0, y: 40.0 };
    //let font_scale = Scale { x: 25.0, y: 25.0 };
    let nick_name_scale = Scale { x: 25.0, y: 25.0 };
    let quote_from = format!("——{}", nick_name);
    let mut lines: Vec<String> = Vec::new();
    let mut line: String = String::new();
    let mut row_width = 0f32;
    let mut row_height = 0f32;
    let row_max_width = 560f32;
    let txt_height;
    msg.chars().enumerate().for_each(|(_, c)| {
        let font_width = font.glyph(c).scaled(scale).h_metrics().advance_width; //获取字符宽度
        if row_width + font_width > row_max_width {
            lines.push(line.clone());
            line = String::new();
            line.push(c);
            row_width = 0f32;
            row_height += font.v_metrics(scale).ascent + 10f32;
        } else {
            row_width += font_width;
            //获取字符高度
            line.push(c);
        }
    });
    lines.push(line.clone());
    txt_height = row_height;
    let mut x = 660i32;
    let mut y = ((640f32 - txt_height) / 2f32) as i32;
    drawing::draw_filled_rect_mut(
        &mut canvas,
        imageproc::rect::Rect::at(0, 0).of_size(1280, 640),
        image::Rgba([avg_color.0 as u8, avg_color.1 as u8, avg_color.2 as u8, 255]),
    );
    img.pixels().for_each(|(x, y, p)| {
        canvas.put_pixel(x, y, p);
    });
    for (_, line) in lines.iter().enumerate() {
        drawing::draw_text_mut(&mut canvas, font_color, x, y, scale, &font, line);
        y += font.v_metrics(scale).ascent as i32 + 10;
    }
    y += 20;
    let nick_name_width = font
        .glyphs_for(nick_name.chars())
        .map(|g| g.scaled(nick_name_scale).h_metrics().advance_width)
        .sum::<f32>();
    x = 1280 - 100 - nick_name_width as i32;
    drawing::draw_text_mut(
        &mut canvas,
        font_color,
        x,
        y,
        nick_name_scale,
        &font,
        quote_from.as_str(),
    );
    //get absolute path
    let mut path = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/images/temp.png");
    let path = path.to_str().unwrap();
    canvas.save(path)?;
    Ok(())
}
#[cfg(test)]
mod test {
    use std::path;
    #[test]
    fn test() {
        let mut path = path::PathBuf::from("./");
        path = path.canonicalize().unwrap();
        println!("{:?}", path);
    }
}
