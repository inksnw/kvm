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
}

#[derive(Serialize)]
pub struct FrontResult {
    pub msg: String,
    pub code: u32,

}