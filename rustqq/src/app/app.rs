use std::fs::File;
use std::io::Read;
use serde::Deserialize;

use crate::event::events::*;
use crate::server::build_server;
use dyn_clone::DynClone;

#[derive(Clone)]
pub struct App {
    ip: String,
    port: u16,
    pub tasks: Vec<Box<dyn TaskHandle>>,
    pub handler: Vec<Box<dyn EventHandle>>,
    pub config: Config,
}
#[derive(Clone, Default,Deserialize)]
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
        let _conf:Config= toml::from_str(&contents).unwrap();
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
#[derive(Default, Clone,Deserialize)]
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
unsafe impl Send for App {}
unsafe impl Sync for App {}
#[async_trait::async_trait]
pub trait EventHandle: Send + Sync + DynClone {
    async fn register(
        &self,
        event: &Event,
        data: &Config,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
#[async_trait::async_trait]
pub trait TaskHandle: Send + Sync + DynClone {
    async fn tasks(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn schedule(&self) -> String;
}
dyn_clone::clone_trait_object!(EventHandle);
dyn_clone::clone_trait_object!(TaskHandle);
impl App {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn socket(&self) -> (&str, u16) {
        (self.ip.as_str(), self.port)
    }
    pub fn bind(&mut self, ip: &str, port: u16) -> &mut Self {
        self.ip = ip.to_string();
        self.port = port;
        self
    }
    pub fn event(mut self, handler: Box<dyn EventHandle>) -> Self {
        self.handler.push(handler);
        self
    }
    pub async fn handle_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
        for f in self.handler.iter() {
            f.register(event, &self.config).await?;
        }
        Ok(())
    
    }
    pub async fn hadle_task(_task: Box<dyn TaskHandle>) {
        todo!()
    }
    pub fn task(mut self, task: Box<dyn TaskHandle>) -> Self {
        self.tasks.push(task);
        self
    }
    pub fn add_tasks(&mut self, tasks: Vec<Box<dyn TaskHandle>>) {
        self.tasks.extend(tasks);
    }
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        build_server(self.clone()).await?;
        Ok(())
    }
    pub fn config(&mut self){
        self.config.load();
    }
}
impl Default for App {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 8080,
            tasks: vec![],
            handler: vec![],
            config: Config::default(),
        }
    }
}
