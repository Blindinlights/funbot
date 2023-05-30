use super::EventHandler;

#[derive(Default,Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub command: String,
    pub alias: String,
    pub exclude: bool,
}
pub struct Service {
    pub info: ServiceInfo,
    pub handler: Box<dyn EventHandler>,
}
impl Service {
    pub fn new<T: EventHandler+'static>(handle: T) -> Self {
        Self {
            info: ServiceInfo::default(),
            handler: Box::new(handle),
        }
    }
    pub fn build<I:IntoService>(mut self,info:I)->Self{
        self.info=info.into_service().info;
        self
    }
}
pub trait  IntoService{
    fn into_service(self)->Service;
}
