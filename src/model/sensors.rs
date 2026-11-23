use serde::Deserialize;
use serde_json::{Map, Value};
use std::{sync::mpsc::Receiver, time::Instant};

/// Un elemento de GET /sensors.
#[derive(Debug, Deserialize, Clone)]
pub struct Sensor {
    pub name: String,
    pub first_seen: f64,
    pub last_seen: f64,
    pub seconds_since_seen: f64,
    pub online: bool,
    #[serde(default)]
    pub offline_alerted: bool,
    /// Datos extra enviados en el heartbeat (objeto libre; `{}` si no hay).
    #[serde(default)]
    pub info: Map<String, Value>,
}

#[derive(Debug)]
pub struct SensorsState {
    pub list: Vec<Sensor>,
    pub auto_refresh: bool,
    pub last_fetch: Option<Instant>,
    pub loaded: bool, // ya se pidió al menos una vez (evita reintentos en bucle)
    pub is_loading: bool,
    pub error: Option<String>,
    pub rx: Option<Receiver<Result<Vec<Sensor>, String>>>,
}

impl Default for SensorsState {
    fn default() -> Self {
        Self {
            list: Vec::new(),
            auto_refresh: true, // los heartbeats son en vivo
            last_fetch: None,
            loaded: false,
            is_loading: false,
            error: None,
            rx: None,
        }
    }
}
