use serde::Serialize;
use virt::domain::DomainState;

#[derive(Serialize)]
pub struct InsModel {
    pub ins_id: u32,
    pub ins_name: String,
    pub state: DomainState,
    pub mem: u64,
    pub cpu: u32,
    pub active: bool,
    pub key: u32,
}

#[derive(Serialize)]
pub struct FrontResult {
    pub msg: String,
    pub code: i32,

}

impl FrontResult {
    pub fn new(msg: &str, code: i32) -> FrontResult {
        FrontResult {
            msg: String::from(msg),
            code,
        }
    }

    pub fn haha(&self) -> String {
        format!("Hello, {} {} !", &self.msg, &self.code)
    }
}