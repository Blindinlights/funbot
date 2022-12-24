extern crate proc_macro;
use async_trait::async_trait;
use codegen::Meassages;

use super::Reply;

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
            #[derive(serde::Serialize,serde::Deserialize,Debug,Clone)]
            pub struct $struct_name{
                pub post_type: String,
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
                message_type:String,
                sub_type:String,
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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Sender {
    age: i32,
    nickname: String,
    sex: String,
    user_id: i64,
}

make_msg_event! {
    struct PrivateMessage{
        temp_source:i64,
    }
}
make_msg_event! {
    struct GroupMessage{
        group_id:i64,
        anonymous:Option<Anonymous>,
    }
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FileInfo {
    id: String,
    name: String,
    size: i64,
    busid: i64,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
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
make_notice_event!{
    struct OfflineFile{
        file:FileInfo,
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
#[derive(Debug)]
pub enum Event {
    PrivateMessage(PrivateMessage),
    GroupMessage(GroupMessage),
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
    OfflineFile(OfflineFile),
    Unknown,
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
