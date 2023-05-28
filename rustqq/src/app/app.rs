use crate::event::events::*;
use axum::{routing::post, Router};
use log::info;
use serde::Deserialize;
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddr};
use std::{fs::File, sync::Arc};
type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type EventHandles = Vec<Box<dyn EventHandle + Send + Sync>>;

pub struct App {
    ip: SocketAddr,
    pub handler: EventHandles,
    pub config: Config,
}
#[derive(Clone, Default, Deserialize)]
pub struct Config {
    plugin: Vec<Plugin>,
}
impl Config {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn load(&mut self) -> &mut Self {
        //read file plugin.toml
        let mut file = File::open("plugin.toml").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let _conf: Config = toml::from_str(&contents).unwrap();
        self
    }
    pub fn is_command(&self, msg: &str) -> bool {
        self.get_command(msg).is_some()
    }
    pub fn get_command(&self, msg: &str) -> Option<&Plugin> {
        self.plugin.iter().find(|x| x.is_match(msg))
    }
}
#[allow(dead_code)]
#[derive(Default, Clone, Deserialize)]
pub struct Plugin {
    commands: Option<Vec<String>>,
    description: String,
    name: String,
    regex: Option<String>,
    usage: String,
    options: Option<Vec<String>>,
}
impl Plugin {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_match(&self, msg: &str) -> bool {
        if self.regex.is_none() {
            false
        } else {
            let re = regex::Regex::new(self.regex.as_ref().unwrap()).unwrap();
            re.is_match(msg)
        }
    }
}
// unsafe impl Send for App {}
// unsafe impl Sync for App {}
#[async_trait::async_trait]
pub trait EventHandle: Send + Sync {
    async fn register(
        &self,
        event: &Event,
        data: &Config,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

impl App {
    pub fn new() -> Self {
        Self {
            ip: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 8080),
            handler: Vec::new(),
            config: Config::new(),
        }
    }

    pub fn bind(mut self, ip: SocketAddr) -> Self {
        self.ip = ip;
        self
    }
    pub fn event<E: EventHandle + 'static>(mut self, handler: E) -> Self {
        self.handler.push(Box::new(handler));
        self
    }
    pub async fn handle_event(&self, event: &Event) -> BoxResult<()> {
        for f in self.handler.iter() {
            f.register(event, &self.config).await?;
        }
        Ok(())
    }
    pub fn config(&mut self) {
        self.config.load();
    }

    pub async fn run(self) -> BoxResult<()> {
        let eh = Arc::new(self.handler);
        let app = Router::new().route("/", post(move |req| index(req, eh)));
        axum::Server::bind(&self.ip)
            .serve(app.into_make_service())
            .await?;
        Ok(())
    }
}

async fn index(req: axum::extract::Json<Event>, handler: Arc<EventHandles>) {
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
        _ => return ,
    }

    for f in handler.iter() {
        f.register(&event, &Config::new()).await.map_err(|e| {
            log::error!("处理事件失败: {:?}", e);
        }).ok();
    }
}
