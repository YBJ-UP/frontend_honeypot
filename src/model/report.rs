use serde::Deserialize;
use std::{collections::BTreeMap, sync::mpsc::Receiver};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Period {
    #[default]
    Day,
    Week,
    Month,
}

impl Period {
    pub const ALL: [Period; 3] = [Period::Day, Period::Week, Period::Month];

    pub fn as_query(&self) -> &'static str {
        match self {
            Period::Day => "24h",
            Period::Week => "7d",
            Period::Month => "30d",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Period::Day => "Últimas 24 horas",
            Period::Week => "Últimos 7 días",
            Period::Month => "Últimos 30 días",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportSection {
    #[default]
    PorIp,
    Distribucion,
    Top,
}

impl ReportSection {
    pub const ALL: [ReportSection; 3] = [
        ReportSection::PorIp,
        ReportSection::Distribucion,
        ReportSection::Top,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ReportSection::PorIp => "Por IP",
            ReportSection::Distribucion => "Distribución",
            ReportSection::Top => "Top",
        }
    }
}

// ---------- Respuesta de GET /reports (y /stats/overview, sin `ips`) ----------

#[derive(Debug, Deserialize, Clone)]
pub struct Totals {
    pub attacks: u32,
    pub unique_ips: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TypeCount {
    #[serde(rename = "type")]
    pub kind: String,
    pub label: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SensorCount {
    pub sensor: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ValueCount {
    pub value: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IpCount {
    pub ip: String,
    pub attacks: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IpRow {
    pub ip: String,
    pub country: Option<String>,
    pub attacks: u32,
    pub types: Vec<String>,
    pub sensors: Vec<String>,
    pub max_severity: u8,
    pub first_seen: f64,
    pub last_seen: f64,
    pub furthest_reached: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralReport {
    pub period: String,
    pub since: Option<f64>,
    pub generated_at: f64,
    pub totals: Totals,
    pub by_type: Vec<TypeCount>,
    pub by_sensor: Vec<SensorCount>,
    /// Claves "1".."5"
    pub by_severity: BTreeMap<String, u32>,
    pub top_ips: Vec<IpCount>,
    pub top_usernames: Vec<ValueCount>,
    pub top_passwords: Vec<ValueCount>,
    pub top_countries: Vec<ValueCount>,
    /// Ausente en /stats/overview
    #[serde(default)]
    pub ips: Vec<IpRow>,
}

// ---------- Estado de la vista ----------

#[derive(Debug, Default)]
pub struct ReportState {
    pub period: Period,
    pub section: ReportSection,
    pub data: Option<GeneralReport>,
    pub filter: String,
    pub loaded: bool, // ya se pidió al menos una vez (evita reintentos en bucle)
    pub is_loading: bool,
    pub error: Option<String>,
    pub rx: Option<Receiver<Result<GeneralReport, String>>>,
}
