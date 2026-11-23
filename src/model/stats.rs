use serde::Deserialize;
use std::{collections::BTreeMap, sync::mpsc::Receiver};

/// Una fila de GET /stats/compare (un sensor).
#[derive(Debug, Deserialize, Clone)]
pub struct SensorCompare {
    pub sensor: String,
    pub closed_attacks: u32,
    pub open_attacks: u32,
    pub unique_ips: u32,
    #[serde(default)]
    pub by_type: BTreeMap<String, u32>,
    pub max_severity: u8,
    pub brute_force_attempts: u32,
    pub successful_logins: u32,
    pub malware_files: u32,
    pub admin_attempts: u32,
    pub seconds_to_first_attack: Option<f64>,
}

#[derive(Debug, Default)]
pub struct StatsState {
    pub sensors: Vec<SensorCompare>,
    pub loaded: bool, // ya se pidió al menos una vez (evita reintentos en bucle)
    pub is_loading: bool,
    pub error: Option<String>,
    pub rx: Option<Receiver<Result<Vec<SensorCompare>, String>>>,
}
