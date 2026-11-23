use serde::Deserialize;
use std::{sync::mpsc::Receiver, time::Instant};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusFilter {
    #[default]
    All,
    Open,
    Closed,
}

impl StatusFilter {
    pub const ALL: [StatusFilter; 3] =
        [StatusFilter::All, StatusFilter::Open, StatusFilter::Closed];

    pub fn as_query(&self) -> &'static str {
        match self {
            StatusFilter::All => "all",
            StatusFilter::Open => "open",
            StatusFilter::Closed => "closed",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            StatusFilter::All => "Todos",
            StatusFilter::Open => "En curso",
            StatusFilter::Closed => "Cerrados",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackSort {
    #[default]
    Id,
    Ip,
    Sensor,
    Type,
    Severity,
    Status,
}

#[derive(Debug, Default)]
pub struct AttackState {
    pub list: Vec<Attack>,

    // Filtros y orden (los valores por defecto dan: todos, sin mínimo, ID descendente)
    pub status_filter: StatusFilter,
    pub min_severity: u8, // 0 = sin filtro
    pub search: String,
    pub sort: AttackSort,
    pub sort_asc: bool,

    // Actualización
    pub auto_refresh: bool,
    pub last_fetch: Option<Instant>,
    pub loaded: bool, // ya se pidió al menos una vez (evita reintentos en bucle)
    pub is_loading: bool,
    pub error: Option<String>,
    pub rx: Option<Receiver<Result<Vec<Attack>, String>>>,
}
