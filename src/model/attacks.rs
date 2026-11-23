use serde::Deserialize;
use std::sync::mpsc::Receiver;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Attack {
    pub id: i32,
    pub status: String,
    pub sensor: String,
    pub ip: String,
    #[serde(rename = "type")]
    pub attack_type: String,
    pub severity: u8,
    pub started_at: Option<String>,
}

#[derive(Debug, Default)]
pub struct AttackState {
    pub list: Vec<Attack>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub rx: Option<Receiver<Result<Vec<Attack>, String>>>,
}
