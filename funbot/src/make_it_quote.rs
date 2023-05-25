use image::{self, GenericImageView};
use imageproc::drawing;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::{self, Regex};
use rustqq::client;
use rustqq::client::message::RowMessage;
use rustqq::event::events::Event;
use rustqq::event::reply_trait::Reply;
use rustqq::handler;
use rusttype::{Font, Scale};
use std::path;
#[handler]
pub async fn quote_it(event: Event) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::GroupMessage(ref msg) = event {
        //match [CQ:reply,id={id}] <msg>
        let re = regex::Regex::new(r"\[CQ:reply,id=(-?\d+)\]\[CQ:.*]* (.*)").unwrap();
        if let Some(e) = re.captures(msg.message.as_str()) {
            let id = e.get(1).unwrap().as_str().parse::<i64>()?;
            let cmd = e.get(2).unwrap().as_str();
            if cmd.starts_with("make-it-quote") {
                let api = client::api::GetMessage::new(id);
                let res = api.post().await?;
                let re = Regex::new(r"(\[CQ:.*\])").unwrap();
                let init_msg = res["data"]["message"].as_str().unwrap();
                if re.captures(init_msg).is_some() {
                    msg.reply("只能引用纯文字").await?;
                    return Err("只能引用纯文字".into());
                }
                if init_msg.len() > 120 {
                    msg.reply("引用的文字太长了").await?;
                    return Err("引用的文字太长了".into());
                }
                let nick_name = res["data"]["sender"]["nickname"].as_str().unwrap();
                let sender_id = res["data"]["sender"]["user_id"].as_i64().unwrap();
                let mut file_name: String = thread_rng()
                    .sample_iter(Alphanumeric)
                    .take(12)
                    .map(char::from)
                    .collect();
                file_name.push_str(".png");
                let mut raw_msg = RowMessage::new();
                //get absolute path
                let mut path = path::PathBuf::from("./");
                path = path.canonicalize()?;
                path.push("images/");
                path.push(file_name);
                let path = path.to_str().unwrap();
                file_name = path.to_string();
                get_pic(sender_id, init_msg, nick_name, path).await?;
                let path = "file://".to_owned() + path;
                raw_msg.add_image(path.as_str());
                msg.reply(raw_msg.get_msg()).await?;
                let path = path.replace("file://", "");
                std::fs::remove_file(path)?;
            }
        }
    }
    Ok(())
}
async fn get_pic(
    id: i64,
    msg: &str,
    nick_name: &str,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("http://q1.qlogo.cn/g?b=qq&nk={id}&s=640");
    let nick_name = format!("--{nick_name}");
    let buf = reqwest::get(&url).await?.bytes().await?.to_vec();
    let avatar = image::load_from_memory(&buf)?;
    let mut avg_color = avatar
        .pixels()
        .map(|p| p.2)
        .fold((0, 0, 0), |(r, g, b), p| {
            (r + p[0] as u32, g + p[1] as u32, b + p[2] as u32)
        });
    avg_color = (
        avg_color.0 / (avatar.width() * avatar.height()),
        avg_color.1 / (avatar.width() * avatar.height()),
        avg_color.2 / (avatar.width() * avatar.height()),
    );

    let mut canvas = image::RgbaImage::new(1280, 640);

    for (x, y, p) in avatar.pixels() {
        let mut n = image::Rgba([p[0], p[1], p[2], p[3]]);
        let y_rate = 0.;
        let start_pix = (320. + 320. * y_rate) as u32;
        if x > start_pix {
            let rate = (x - start_pix) as f32 / (640 - start_pix) as f32;
            let rate = 1. / (1. + 1f32.exp().powf(-7.5 * (rate - 0.5)));

            let r = ((p[0] as f32 * (1. - rate)) as u32 + (avg_color.0 as f32 * rate) as u32) as u8;
            let g = ((p[1] as f32 * (1. - rate)) as u32 + (avg_color.1 as f32 * rate) as u32) as u8;
            let b = ((p[2] as f32 * (1. - rate)) as u32 + (avg_color.2 as f32 * rate) as u32) as u8;
            let a = ((p[3] as f32 * (1. - rate)) as u32 + (255. as f32 * rate) as u32) as u8;
            n = image::Rgba([r, g, b, a]);
        }
        canvas.put_pixel(x, y, n);
    }
    let avg_color = image::Rgba([avg_color.0 as u8, avg_color.1 as u8, avg_color.2 as u8, 255]);
    
    canvas
        .pixels_mut()
        .enumerate()
        .filter(|(i, _)| i % 1280 >= 640)
        .for_each(|(_, p)| *p = avg_color);

    let font_data = include_bytes!("../fonts/mergefonts.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).unwrap();
    let scale = Scale { x: 45.0, y: 45.0 };
    let font_color = {
        let mut c = image::Rgba([255, 255, 255, 255]);
        for i in 0..3 {
            c[i] = if avg_color[i] > 128 {
                avg_color[i] - 90
            } else {
                avg_color[i] + 90
            }
        }
        c
    };
    let nick_name_scale = Scale { x: 25.0, y: 25.0 };
    let mut lines: Vec<String> = Vec::new();
    let mut line: String = String::new();
    let mut row_width = 0f32;
    let mut row_height = 0f32;
    let row_max_width = 600f32;

    for (_, c) in msg.chars().enumerate() {
        let font_width = font.glyph(c).scaled(scale).h_metrics().advance_width; //获取字符宽度
        if c == '\n' {
            lines.push(line.clone());
            line = String::new();
            row_width = 0f32;
            row_height += font.v_metrics(scale).ascent + 10f32;
        } else if row_width + font_width > row_max_width {
            lines.push(line.clone());
            line = String::new();
            line.push(c);
            row_width = 0f32;
            row_height += font.v_metrics(scale).ascent + 10f32;
        } else {
            row_width += font_width;
            line.push(c);
        }
    }
    lines.push(line.clone());
    let txt_height = row_height;
    let mut x = 660i32;
    let mut y = ((640f32 - txt_height) / 2f32) as i32;
    for (_, line) in lines.iter().enumerate() {
        drawing::draw_text_mut(&mut canvas, font_color, x, y, scale, &font, line);
        y += font.v_metrics(scale).ascent as i32 + 10;
    }
    y += 20;
    let nick_name_width = font
        .glyphs_for(nick_name.chars())
        .map(|g| g.scaled(nick_name_scale).h_metrics().advance_width)
        .sum::<f32>();
    x = 1280 - 20 - nick_name_width as i32;

    drawing::draw_text_mut(
        &mut canvas,
        font_color,
        x,
        y,
        nick_name_scale,
        &font,
        nick_name.as_str(),
    );
    canvas.save(file_name)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_pic() {
        let id = 1057584970;
        let msg = "Hello world";
        let nick_name = "test";
        let file_name = "test.png";
        get_pic(id, msg, nick_name, file_name).await.unwrap();
    }
}
