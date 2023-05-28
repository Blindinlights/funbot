use super::EventHandler;

#[derive(Default,Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub command: String,
    pub alias: String,
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
pub trait  IntoServiceInfo{
    fn into_service_info(&self)->ServiceInfo;
}
pub struct Services(pub Vec<Service>);

pub trait IntoServices{
    fn into_services(self)->Services;
}

impl<T> IntoServices for T
where
    T: IntoService,
{
    fn into_services(self)->Services{
        Services(vec![self.into_service()])
    }
}
impl<T> IntoServices for Vec<T>
where
    T: IntoService,
{
    fn into_services(self)->Services{
        let mut services=Vec::new();
        for s in self.into_iter(){
            services.push(s.into_service());
        }
        Services(services)
    }
}