extern crate proc_macro;

use async_trait::async_trait;
use codegen::Meassages;
use serde::{Deserialize, Serialize};

use super::Reply;
#[derive(Deserialize, Debug, Clone, Serialize, Default)]
pub enum PostType {
    Message,
    Notice,
    Request,
    #[serde(rename = "meta_event")]
    MetaEvent,
    #[serde(other)]
    #[default]
    Other,
}
#[derive(Deserialize, Debug, Clone, Serialize, Default)]
pub enum MessageType {
    #[serde(rename = "private")]
    #[default]
    Private,
    #[serde(rename = "group")]
    Group,
}
#[derive(Deserialize, Debug, Clone, Serialize, Default)]
pub enum MsgSubType {
    #[serde(rename = "friend")]
    Friend,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "anonymous")]
    Anonymous,
    #[serde(rename = "group_self")]
    GroupSelf,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "notice")]
    Notice,
    #[serde(other)]
    #[default]
    Other,
}

macro_rules! make_event{
    (
     $(#[$meta:meta])*
     $vis:vis struct $struct_name:ident {
        $(
        $(#[$field_meta:meta])*
        $field_vis:vis $field_name:ident : $field_type:ty
        ),*$(,)+
    }
    ) => {
            $(#[$meta])*
            #[derive(serde::Deserialize,Debug,Clone,serde::Serialize,Default)]
            pub struct $struct_name{
                pub post_type: PostType,
                pub self_id: i64,
                pub time: i64,
                $(
                $(#[$field_meta:meta])*
                pub $field_name : $field_type,
                )*
            }
    }
}
macro_rules! make_msg_event{
    (
     $(#[$meta:meta])*
     $vis:vis struct $struct_name:ident {
        $(
        $(#[$field_meta:meta])*
        $field_vis:vis $field_name:ident : $field_type:ty
        ),*$(,)+
    }
    ) => {
            make_event!{
            $(#[$meta])*
            #[derive(Meassages)]
            pub struct $struct_name{
                message_type:MessageType,
                sub_type:MsgSubType,
                message_id:i64,
                user_id:i64,
                message:String,
                raw_message:String,
                font:i32,
                sender:Sender,
                $(

                pub $field_name : $field_type,
                )*
            }

            }

    }
}
macro_rules! make_notice_event {
    (
        $(#[$meta:meta])*
        $vis:vis struct $struct_name:ident {
           $(
           $(#[$field_meta:meta])*
           $field_vis:vis $field_name:ident : $field_type:ty
           ),*$(,)+
       }
       ) => {
               make_event!{
               $(#[$meta])*
               pub struct $struct_name{
                    pub notice_type:String,
                   $(

                   pub $field_name : $field_type,
                   )*
               }

               }

       }
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Sender {
    pub age: i32,
    pub nickname: String,
    pub sex: String,
    pub user_id: i64,
}

make_event! {
    #[derive(Meassages)]
    struct PrivateMessage{
        message_type:MessageType,
        sub_type:String,
        message_id:i64,
        user_id:i64,
        message:String,
        raw_message:String,
        font:i32,
        sender:Sender,
    }
}
make_msg_event! {
    struct GroupMessage{
        group_id:i64,
        anonymous:Option<Anonymous>,
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FileInfo {
    id: String,
    name: String,
    size: i64,
    busid: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Anonymous {
    id: i64,
    name: String,
    flag: String,
}
make_notice_event! {
    struct GroupFileUpload{
        group_id:i64,
        user_id:i64,
        file:FileInfo,
    }
}
make_notice_event! {
    struct GroupAdminChange{
        sub_type:String,
        group_id:i64,
        user_id:i64,
    }
}
make_notice_event! {
    struct GroupMemberReduce{
        sub_type:String,
        group_id:i64,
        user_id:i64,
        operator_id:i64,
    }
}

make_notice_event! {
    struct GroupMemberIncrease{
        sub_type:String,
        group_id:i64,
        user_id:i64,
        operator_id:i64,
    }
}
make_notice_event! {
    struct GroupMute{
        sub_type:String,
        group_id:i64,
        operator_id:i64,
        user_id:i64,
        duration:i64,
    }
}
make_notice_event! {
    struct FriendAdd{
        user_id:i64,
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfflineFile {
    name: String,
    size: i64,
    url: String,
}
make_notice_event! {
    struct OfflineFileUpload{
        file:OfflineFile,
    }
}

make_notice_event! {
    struct GroupMessageRecall{
        group_id:i64,
        message_id:i64,
        user_id:i64,
        operator_id:i64,

    }
}
make_notice_event! {
    struct FriendMessageRecall{
        user_id:i64,
        message_id:i64,
    }
}
make_notice_event! {
    struct FriendPoke{
        sub_type:String,
        user_id:i64,
        sender_id:i64,
        target_id:i64,
    }
}
make_notice_event! {
    struct GroupPoke{
        sub_type:String,
        group_id:i64,
        sender_id:i64,
        target_id:i64,
    }
}
make_event! {
    struct FriendRequest{
        request_type:String,
        user_id:i64,
        comment:String,
        flag:String,
    }
}
make_event! {
    struct GroupRequest{
        request_type:String,
        sub_type:String,
        group_id:i64,
        user_id:i64,
        comment:String,
        flag:String,
    }
}
make_event! {
    struct MetaEvent{
        meta_event_type:String,
        status:String,
        interval:i64,
    }
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Event {
    GroupMessage(GroupMessage),
    PrivateMessage(PrivateMessage),
    GroupFileUpload(GroupFileUpload),
    GroupAdminChange(GroupAdminChange),
    GroupMemberReduce(GroupMemberReduce),
    GroupMemberIncrease(GroupMemberIncrease),
    GroupMute(GroupMute),
    FriendAdd(FriendAdd),
    GroupMessageRecall(GroupMessageRecall),
    FriendMessageRecall(FriendMessageRecall),
    FriendPoke(FriendPoke),
    GroupPoke(GroupPoke),
    FriendRequest(FriendRequest),
    GroupRequest(GroupRequest),
    MetaEvent(MetaEvent),
    OfflineFileUpload(OfflineFileUpload),
    Unknown,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        let gm = GroupMessage::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();
        matches!(de, Event::GroupMessage(_));
    }
    #[test]
    fn test_json2() {
        let gm = PrivateMessage::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();
        matches!(de, Event::PrivateMessage(_));
    }

    #[test]
    fn test_json3() {
        let gm = GroupFileUpload::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();
        matches!(de, Event::GroupFileUpload(_));
    }
    #[test]
    fn test_json4() {
        let gm = GroupAdminChange::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();
        matches!(de, Event::GroupAdminChange(_));
    }
    #[test]
    fn test_json5() {
        let gm = GroupMemberReduce::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();
        matches!(de, Event::GroupMemberReduce(_));
    }
    #[test]
    fn test_json6() {
        let gm = GroupMemberIncrease::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();
        matches!(de, Event::GroupMemberIncrease(_));
    }
    #[test]
    fn test_json7() {
        let gm = GroupMute::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();
        matches!(de, Event::GroupMute(_));
    }
    #[test]
    fn test_json8() {
        let gm = FriendAdd::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::FriendAdd(_));
    }
    #[test]
    fn test_json9() {
        let gm = GroupMessageRecall::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::GroupMessageRecall(_));
    }
    #[test]
    fn test_json10() {
        let gm = FriendMessageRecall::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::FriendMessageRecall(_));
    }
    #[test]
    fn test_json11() {
        let gm = FriendPoke::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::FriendPoke(_));
    }
    #[test]
    fn test_json12() {
        let gm = GroupPoke::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::GroupPoke(_));
    }
    #[test]
    fn test_json13() {
        let gm = FriendRequest::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::FriendRequest(_));
    }
    #[test]
    fn test_json14() {
        let gm = GroupRequest::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::GroupRequest(_));
    }
    #[test]
    fn test_json15() {
        let gm = MetaEvent::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::MetaEvent(_));
    }
    #[test]
    fn test_json16() {
        let gm = OfflineFileUpload::default();
        let json = serde_json::to_string(&gm).unwrap();
        let de: Event = serde_json::from_str(&json).unwrap();

        matches!(de, Event::OfflineFileUpload(_));
    }
}

impl Event {

    pub fn msg_event(&self) -> Option<MsgEvent> {
        match self {
            Event::PrivateMessage(msg) => Some(MsgEvent::PrivateMessage(msg.clone())),
            Event::GroupMessage(msg) => Some(MsgEvent::GroupMessage(msg.clone())),
            _ => None,
        }
    }
    
}



#[async_trait]
pub trait Meassages {
    fn start_with(&self, s: &str) -> bool;
    fn eq(&self, s: &str) -> bool;
    fn msg(&self) -> &str;
}
pub enum MsgEvent {
    PrivateMessage(PrivateMessage),
    GroupMessage(GroupMessage),
}
impl MsgEvent {
    pub fn new(event: &Event) -> Option<Self> {
        match event {
            Event::PrivateMessage(msg) => Some(MsgEvent::PrivateMessage(msg.clone())),
            Event::GroupMessage(msg) => Some(MsgEvent::GroupMessage(msg.clone())),
            _ => None,
        }
    }
    pub fn msg_id(&self) -> i64 {
        match self {
            MsgEvent::PrivateMessage(msg) => msg.message_id,
            MsgEvent::GroupMessage(msg) => msg.message_id,
        }
    }
    fn is_private(&self) -> bool {
        match self {
            MsgEvent::PrivateMessage(_) => true,
            MsgEvent::GroupMessage(_) => false,
        }
    }
    pub fn is_group(&self) -> bool {
        !self.is_private()
    }
    pub fn group_id(&self) -> Option<i64> {
        match self {
            MsgEvent::PrivateMessage(_) => None,
            MsgEvent::GroupMessage(msg) => Some(msg.group_id),
        }
    }
    pub fn user_id(&self) -> i64 {
        match self {
            MsgEvent::PrivateMessage(msg) => msg.user_id,
            MsgEvent::GroupMessage(msg) => msg.user_id,
        }
    }
}
impl Meassages for MsgEvent {
    fn start_with(&self, s: &str) -> bool {
        match self {
            MsgEvent::PrivateMessage(msg) => msg.message.starts_with(s),
            MsgEvent::GroupMessage(msg) => msg.message.starts_with(s),
        }
    }
    fn eq(&self, s: &str) -> bool {
        match self {
            MsgEvent::PrivateMessage(msg) => msg.message.eq(s),
            MsgEvent::GroupMessage(msg) => msg.message.eq(s),
        }
    }
    fn msg(&self) -> &str {
        match self {
            MsgEvent::PrivateMessage(msg) => &msg.message,
            MsgEvent::GroupMessage(msg) => &msg.message,
        }
    }
}
#[async_trait]
impl Reply for MsgEvent {
    async fn reply(&self, msg: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            MsgEvent::PrivateMessage(e) => e.reply(msg).await,
            MsgEvent::GroupMessage(e) => e.reply(msg).await,
        }
    }
}
impl GroupMessage {
    pub fn at_me(&self) -> Option<String> {
        let msg = &self.message;
        let id = self.self_id;
        let regex = format!(r"^\[CQ:at,qq={}\](.+)$", id);
        let re = regex::Regex::new(&regex).unwrap();
        if let Some(caps) = re.captures(msg) {
            return Some(caps[1].to_string());
        }
        None
    }
}
