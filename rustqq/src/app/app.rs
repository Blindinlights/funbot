use  crate::event::events::*;
use crate::server::build_server;
use dyn_clone::DynClone;
use toml;
#[derive(Clone)]
pub struct App{
    ip:String,
    port:u16,
    pub tasks:Vec<Box<dyn TaskHandle>>,
    pub handler:Vec<Box<dyn EventHandle>>,
    pub config:Config
    
}
#[derive(Clone,Default)]
pub struct Config{
    plugins:Vec<Plugin>

}
#[allow(dead_code)]
#[derive(Default,Clone)]
pub struct Plugin{
    commands:Option<Vec<String>>,
    description:String,
    name:String,
    regex:Option<String>,
    usage:String,
    options:Option<Vec<String>>,

}
unsafe impl Send for App{}
unsafe impl Sync for App{}
#[async_trait::async_trait]
pub trait EventHandle:Send + Sync+DynClone{
    async fn register(&self,event:&Event,data:&Config)->Result<(),Box<dyn std::error::Error>>;
}
#[async_trait::async_trait]
pub trait TaskHandle:Send + Sync+DynClone{
    async fn tasks(&self)->Result<(),Box<dyn std::error::Error>>;
    fn schedule(&self)->String;
}
dyn_clone::clone_trait_object!(EventHandle);
dyn_clone::clone_trait_object!(TaskHandle);
impl App{
    pub fn new()->Self{
        Self::default()
    }
    pub  fn socket(&self)->(&str,u16){
        (self.ip.as_str(),self.port)
    }
    pub fn bind(&mut self,ip:&str,port:u16)->&mut Self{
        self.ip=ip.to_string();
        self.port=port;
        self
    }
    pub fn event(mut self,handler:Box<dyn EventHandle>)->Self{
        self.handler.push(handler);
        self
    }
    pub async fn handle_event(&self,event:&Event)->Result<(),Box<dyn std::error::Error>>{
        for f in self.handler.iter(){
            f.register(event,&self.config).await?;
        }
        Ok(())
        //todo!()
    }
    pub async fn hadle_task(_task:Box<dyn TaskHandle>){
        //todo!()
    }
    pub fn task(mut self,task:Box<dyn TaskHandle>)->Self{
        self.tasks.push(task);
        self
    }
    pub fn add_tasks(&mut self,tasks:Vec<Box<dyn TaskHandle>>){
        self.tasks.extend(tasks);
    }
    pub async fn run( self)->Result<(),Box<dyn std::error::Error>>{
        build_server(self.clone()).await?;
        Ok(())
    }
}
impl Default for App{
    fn default()->Self{
        Self{
            ip:"127.0.0.1".to_string(),
            port:8080,
            tasks:vec![],
            handler:vec![],
            config:Config::default()
        }
    }
}