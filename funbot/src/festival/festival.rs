use rustqq::{
    app::AsyncJob,
    client::{api, message::RowMessage},
};
const GROUPS: [i64; 2] = [256658318, 806179273];
const IMG:&str="http://blindinlights.cn:5212/api/v3/file/get/329/cdd20-wcciKKyBFkc-unsplash.jpg?sign=NfVtVVkAGkyz-iIfiPxHX0al4iY7QikKFXMZanjsJpo%3D%3A0";

async fn new_year() {
    for group in GROUPS {
        let mut msg = RowMessage::new();
        msg.add_image(IMG);
        let api = api::SendGroupMessage::new(group, msg.get_msg().to_string());
        api.post().await.unwrap();
    }
}

pub fn get_job() -> AsyncJob {
    let job=AsyncJob::new("0 0 0 1 1* *".parse().unwrap(),new_year);
    job
}
