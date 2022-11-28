
use  crate::event::events::*;
pub struct App{
    factory:Vec<Box<dyn AppServiceFactory>>
}
#[async_trait::async_trait]
pub trait AppServiceFactory:Send + Sync{
    async fn register(&self,event:&Event)->Result<(),Box<dyn std::error::Error>>;
} 
impl App{
    pub fn new()->Self{
        Self{
            factory:Vec::new()
        }
    }
    pub fn service<>(mut self,factory:Box<dyn AppServiceFactory>)->Self{
        self.factory.push(factory);
        self
    }
    pub async fn run(&self,event:&Event){
        for f in self.factory.iter(){
            f.register(event.clone()).await.unwrap();
        }
    }
}
