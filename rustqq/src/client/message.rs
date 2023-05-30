#[derive(Default)]
pub struct RowMessage {
    content: String,
}
impl RowMessage {
    pub fn new() -> RowMessage {
        Self::default()
    }
    pub fn text(&mut self, txt: &str) -> &mut Self {
        self.content.push_str(txt);
        self
    }
    pub fn qq_face(&mut self, id: i32) -> &mut Self {
        self.content
            .push_str(format!("[CQ:face,id={id}]").as_str());
        self
    }
    pub fn at_someone(&mut self, qq: i64) -> &mut Self {
        self.content.push_str(format!("[CQ:at,qq={qq}]").as_str());
        self
    }
    pub fn at_all(&mut self) -> &mut Self {
        self.content.push_str("[CQ:at,qq=all]");
        self
    }
    pub fn shift_line(&mut self) -> &mut Self {
        self.content.push('\n');
        self
    }
    pub fn image(&mut self, url: &str) -> &mut Self {
        if url.is_empty(){
            return self;
        }
        self.content
            .push_str(format!("[CQ:image,file={url}]").as_str());
        self
    }
    pub fn add_record(&mut self, url: &str) -> &mut Self {
        self.content
            .push_str(format!("[CQ:record,file={url},cache=1]").as_str());
        self
    }
    pub fn get_msg(&self) -> &str {
        self.content.as_str()
    }
    pub fn clear(&mut self) {
        self.content.clear();
    }
    pub fn reply(&mut self,msg_id:i64)->&mut Self{
        self.content.push_str(format!("[CQ:reply,id={msg_id}]").as_str());
        self
    }
    pub fn msg(&self)->String{
        self.content.clone()
    }
}