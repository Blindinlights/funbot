use rustqq::{client::api, app::AsyncJob};
const GROUPS:[i64;2]=[256658318,806179273];
const CHRISMAS:&str="Let's be jolly~
🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉
Deck the halls with boughs of holly~
🎆🎆🎆🎆🎆🎆🎆🎆🎆🎆
Rocking around the Christmas tree~
🎄🎄🎄🎄🎄🎄🎄🎄🎄🎄
Have a happy holiday!
";

async fn chrismas(){
    for group in GROUPS.iter(){
        let api=api::SendGroupMessage::new(*group,CHRISMAS.to_string());
        api.post().await.unwrap();
    }
}
pub fn get_job()->AsyncJob{
    let job=AsyncJob::new("0 30 7 25 12 * *".parse().unwrap(),chrismas);
    job
}