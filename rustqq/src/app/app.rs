use crate::event::events::*;
use actix_web::{web, Responder};
use log::info;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use super::service::{IntoService, IntoServiceInfo, IntoServices, Service};
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
    pub fn service<T: IntoServices>(mut self, service: T) -> Self {
        self.services.extend(service.into_services().0);
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

    for f in handler.iter() {
        f.handler.register(&event).await.ok().unwrap();
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
    T: Command + Send + Sync + IntoServiceInfo,
{
    async fn register(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
        let info = self.into_service_info();
        let mut cmds = info.command.clone();
        if !info.alias.is_empty() {
            cmds.push_str("|");
            cmds.push_str(info.alias.as_str());
        }
        let cmds = cmds.split("|").collect::<Vec<_>>();
        let pre = |s: &str| {
            for cmd in cmds.iter() {
                if s.starts_with(cmd) {
                    return true;
                }
            }
            false
        };
        match event {
            Event::PrivateMessage(e) => {
                if pre(&e.message) {
                    self.proc(MsgEvent::PrivateMessage(e.clone())).await?;
                }
            }
            Event::GroupMessage(e) => {
                if pre(&e.message) {
                    self.proc(MsgEvent::GroupMessage(e.clone())).await?;
                }
            }
            _ => return Ok(()),
        }
        Ok(())
    }
}
