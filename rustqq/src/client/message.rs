pub struct RowMessage {
    content: String,
}
impl RowMessage {
    pub fn new(str: &String) -> RowMessage {
        Self {
            content: str.clone(),
        }
    }
    pub fn add_plain_txt(&mut self, txt: &String) -> &mut Self {
        self.content.push_str(txt.as_str());
        self
    }
    pub fn add_qq_face(&mut self, id: i32) -> &mut Self {
        self.content
            .push_str(format!("[CQ:face,id={}]", id).as_str());
        self
    }
    pub fn add_at_someone(&mut self, qq: i64) -> &mut Self {
        self.content.push_str(format!("[CQ:at,qq={}]", qq).as_str());
        self
    }
    pub fn add_at_all(&mut self) -> &mut Self {
        self.content.push_str(format!("[CQ:at,qq=all]").as_str());
        self
    }
    pub fn shift_line(&mut self) -> &mut Self {
        self.content.push('\n');
        self
    }
}