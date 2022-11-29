use  crate::event::events::*;
use crate::server::build_server;
use dyn_clone::DynClone;
#[derive(Clone)]
pub struct App{
    ip:String,
    port:u16,
    pub tasks:Vec<Box<dyn TaskHandle>>,
    pub handler:Vec<Box<dyn EventHandle>>
}
unsafe impl Send for App{}
unsafe impl Sync for App{}
#[async_trait::async_trait]
pub trait EventHandle:Send + Sync+DynClone{
    async fn register(&self,event:&Event)->Result<(),Box<dyn std::error::Error>>;
}
#[async_trait::async_trait]
pub trait TaskHandle:Send + Sync+DynClone{
    async fn tasks(&self,event:Event)->Result<(),Box<dyn std::error::Error>>;
}
dyn_clone::clone_trait_object!(EventHandle);
dyn_clone::clone_trait_object!(TaskHandle);
//dyn_clone::clone_trait_object!(TaskHandle);
impl App{
    pub fn new()->Self{
        Self{
            ip:"127.0.0.1".to_string(),
            port:8080,
            tasks:vec![],
            handler:vec![]
        }
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
            f.register(event.clone()).await?;
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
    pub async fn run( self)->Result<(),Box<dyn std::error::Error>>{
        build_server(self.clone()).await?;
        //block( build_server(self)).await?;
        Ok(())
        
        
    }
}
