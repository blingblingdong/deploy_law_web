pub struct LawInform {
    pub name: String,
    pub originalhref: String,
    pub update_time: String,
}

impl LawInform {
    pub fn new(name: String, update_time: String) -> Self {
        LawInform {
            name,
            originalhref: "".to_string(),
            update_time,
        }
    }

    pub fn update_href(mut self, href: String) {
        self.originalhref = href;
    }
}
