use crate::event::{events::*, Reply};
use actix_web::{web, Responder};
use log::info;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use super::service::{IntoService, Service};
use super::{AsyncJob, AsyncJobScheduler};
type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type EventHandles = Vec<Box<dyn EventHandler + Send + Sync>>;

pub struct App {
    ip: SocketAddr,
    handler: EventHandles,
    pub scheduler: AsyncJobScheduler,
    services: Vec<Service>,
}
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn register(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>>;
}

impl App {
    pub fn new() -> Self {
        Self {
            ip: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 8080),
            handler: Vec::new(),
            scheduler: AsyncJobScheduler::new(),
            services: Vec::new(),
        }
    }

    pub fn bind(mut self, ip: SocketAddr) -> Self {
        self.ip = ip;
        self
    }
    pub fn event<E: EventHandler + 'static>(mut self, handler: E) -> Self {
        self.handler.push(Box::new(handler));
        self
    }
    pub fn job(&mut self, job: AsyncJob) {
        self.scheduler.add_job(job);
    }
    pub fn service<T: IntoService>(mut self, service: T) -> Self {
        self.services.push(service.into_service());
        self
    }

    pub async fn run(self) -> BoxResult<()> {
        let services = Arc::new(self.services);
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .service(web::resource("/").route(web::post().to(index)))
                .app_data(actix_web::web::Data::new(services.clone()))
        })
        .bind(self.ip)?
        .run()
        .await?;

        Ok(())
    }
}

async fn index(
    req: actix_web::web::Json<Event>,
    handler: web::Data<Arc<Vec<Service>>>,
) -> impl Responder {
    let event = req.0;
    match &event {
        Event::PrivateMessage(e) => {
            info!(
                "收到 {} ({}) 的消息: {}",
                e.sender.nickname, e.user_id, e.message
            );
        }
        Event::GroupMessage(e) => {
            info!(
                "收到 {} ({}) 的群消息: {}",
                e.sender.nickname, e.user_id, e.message
            );
        }
        _ => return "ok",
    }
    if let Some(e) = event.msg_event() {
        if e.msg().starts_with("/help") {
            let name = e.msg().trim_start_matches("/help").trim();
            if name.is_empty() {
                let msg = handler
                    .iter()
                    .map(|s| s.info.clone())
                    .filter(|s| !s.name.is_empty())
                    .map(|s| format!("--{}\n {}", s.name, s.description))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                let message = msg.join("\n");
                let mut msg = "帮助信息:\n".to_string() + &message;
                msg.push_str("\n\n输入/help <命令名> 获取详细信息");

                e.reply(&msg)
                    .await
                    .map_err(|e| {
                        info!("回复消息出错:{}", e);
                    })
                    .ok();
                return "ok";
            } else {
                let info = handler
                    .iter()
                    .map(|s| s.info.clone())
                    .find(|s| s.name == name);
                if let Some(info) = info {
                    let msg = format!(
                        "名称:{}\n说明:{}\n命令:{}\n别名:{}",
                        info.name, info.description, info.command, info.alias
                    );
                    e.reply(&msg)
                        .await
                        .map_err(|e| {
                            info!("回复消息出错:{}", e);
                        })
                        .ok();
                    return "ok";
                } else {
                    e.reply("未找到该命令")
                        .await
                        .map_err(|e| {
                            info!("回复消息出错:{}", e);
                        })
                        .ok();
                }
            }
        }
    }

    let cmds = handler
        .iter()
        .map(|f| {
            let cmd = f.info.command.clone();
            let mut cmd = vec![cmd];
            if !f.info.alias.is_empty() {
                let alias: Vec<String> = f
                    .info
                    .alias
                    .split("|")
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                cmd.extend(alias);
            }
            cmd
        })
        .flatten()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    for f in handler.iter() {
        if let Some(e) = event.msg_event() {
            let msg = e.msg();
            if f.info.exclude && cmds.iter().any(|c| msg.starts_with(c)) {
                continue;
            }
            if !f.info.command.is_empty() {
                let cmd = f.info.command.clone();
                let cmd = vec![cmd];
                let cmd = f
                    .info
                    .alias
                    .split("|")
                    .map(|s| s.to_string())
                    .chain(cmd)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                if !cmd.iter().any(|c| msg.starts_with(c)) {
                    continue;
                }
            }
        }

        let _ = f.handler.register(&event).await.map_err(|e| {
            info!("处理消息出错:{}", e);
        });
    }
    "ok"
}
#[async_trait::async_trait]
pub trait Command: IntoService {
    async fn proc(&self, msg: MsgEvent) -> BoxResult<()>;
}
#[async_trait::async_trait]
impl<T> EventHandler for T
where
    T: Command + Send + Sync,
{
    async fn register(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(e) = event.msg_event() {
            self.proc(e).await?;
        }
        Ok(())
    }
}
