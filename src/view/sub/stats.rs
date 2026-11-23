use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::{collections::BTreeSet, sync::mpsc};
use tokio::spawn;

use crate::{
    App,
    api::get_json,
    model::stats::SensorCompare,
    view::widgets::{bar_row, fmt_duration, severity_color},
};

pub fn stats_view(app: &mut App, ui: &mut egui::Ui) {
    poll_stats(app);

    // Carga automática la primera vez que se entra a la pestaña
    if !app.stats.loaded {
        request_stats(app, ui.ctx());
    }

    ui.horizontal(|ui| {
        ui.heading("Comparativa de sensores");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Actualizar").clicked() && !app.stats.is_loading {
                request_stats(app, ui.ctx());
            }
            if app.stats.is_loading {
                ui.spinner();
            }
        });
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    if let Some(err) = &app.stats.error {
        ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
        ui.add_space(10.0);
    }

    if app.stats.sensors.is_empty() {
        if app.stats.loaded && !app.stats.is_loading && app.stats.error.is_none() {
            ui.label("No hay sensores registrados.");
        }
        return;
    }

    let sensors = &app.stats.sensors;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            metrics_table(ui, sensors);
            ui.add_space(25.0);
            types_breakdown(ui, sensors);
        });
}

/// Tabla transpuesta: filas = métricas, columnas = sensores.
fn metrics_table(ui: &mut egui::Ui, sensors: &[SensorCompare]) {
    ui.strong("Métricas");
    ui.add_space(5.0);

    // (etiqueta, descripción para el tooltip, accesor)
    let count_rows: [(&str, &str, fn(&SensorCompare) -> u32); 7] = [
        (
            "Ataques cerrados",
            "Ataques ya finalizados (los únicos que cuentan en las demás métricas)",
            |s| s.closed_attacks,
        ),
        ("Ataques abiertos", "Ataques aún en curso", |s| {
            s.open_attacks
        }),
        ("IPs únicas", "IPs distintas con ataques cerrados", |s| {
            s.unique_ips
        }),
        (
            "Intentos de fuerza bruta",
            "Suma de intentos en ataques de fuerza bruta",
            |s| s.brute_force_attempts,
        ),
        (
            "Logins exitosos",
            "Accesos conseguidos en ataques de intrusión",
            |s| s.successful_logins,
        ),
        (
            "Archivos de malware",
            "Archivos capturados en ataques de malware",
            |s| s.malware_files,
        ),
        (
            "Intentos de acceso admin",
            "Suma de intentos en ataques al panel de administración",
            |s| s.admin_attempts,
        ),
    ];

    let mut table = TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .vscroll(false) // ya estamos dentro de un ScrollArea
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(210.0));
    for _ in sensors {
        table = table.column(Column::remainder().at_least(90.0));
    }

    table
        .header(24.0, |mut header| {
            header.col(|ui| {
                ui.strong("Métrica");
            });
            for s in sensors {
                header.col(|ui| {
                    ui.strong(&s.sensor);
                });
            }
        })
        .body(|mut body| {
            // Contadores: en negrita el valor más alto de la fila
            for (label, hint, get) in count_rows {
                let max = sensors.iter().map(|s| get(s)).max().unwrap_or(0);
                body.row(24.0, |mut row| {
                    row.col(|ui| {
                        ui.label(label).on_hover_text(hint);
                    });
                    for s in sensors {
                        row.col(|ui| {
                            let value = get(s);
                            let mut text = egui::RichText::new(value.to_string());
                            if value == max && max > 0 {
                                text = text.strong();
                            }
                            ui.label(text);
                        });
                    }
                });
            }

            // Severidad máxima
            body.row(24.0, |mut row| {
                row.col(|ui| {
                    ui.label("Severidad máxima");
                });
                for s in sensors {
                    row.col(|ui| {
                        if s.max_severity == 0 {
                            ui.label("—");
                        } else {
                            let color = severity_color(s.max_severity, ui.visuals().text_color());
                            ui.colored_label(color, s.max_severity.to_string());
                        }
                    });
                }
            });

            // Tiempo al primer ataque: en negrita el más rápido
            let fastest = sensors
                .iter()
                .filter_map(|s| s.seconds_to_first_attack)
                .fold(f64::INFINITY, f64::min);
            body.row(24.0, |mut row| {
                row.col(|ui| {
                    ui.label("Tiempo al primer ataque").on_hover_text(
                        "Desde que el sensor apareció hasta su primer ataque cerrado",
                    );
                });
                for s in sensors {
                    row.col(|ui| match s.seconds_to_first_attack {
                        Some(secs) => {
                            let mut text = egui::RichText::new(fmt_duration(secs));
                            if secs == fastest {
                                text = text.strong();
                            }
                            ui.label(text);
                        }
                        None => {
                            ui.label("—");
                        }
                    });
                }
            });
        });
}

/// Por cada tipo de ataque, una barra por sensor (misma escala para todas).
fn types_breakdown(ui: &mut egui::Ui, sensors: &[SensorCompare]) {
    ui.strong("Ataques cerrados por tipo");

    let types: BTreeSet<&str> = sensors
        .iter()
        .flat_map(|s| s.by_type.keys().map(String::as_str))
        .collect();

    if types.is_empty() {
        ui.add_space(5.0);
        ui.weak("Aún no hay ataques cerrados.");
        return;
    }

    let max = sensors
        .iter()
        .flat_map(|s| s.by_type.values().copied())
        .max()
        .unwrap_or(0);

    for kind in types {
        ui.add_space(10.0);
        ui.label(egui::RichText::new(type_label(kind)).strong());
        for s in sensors {
            let count = s.by_type.get(kind).copied().unwrap_or(0);
            bar_row(ui, &s.sensor, count, max, None);
        }
    }
}

/// "brute_force" -> "Brute force". Cámbialo por un match con TYPE_LABELS si lo prefieres.
fn type_label(key: &str) -> String {
    let spaced = key.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

// ---------- Red ----------

fn poll_stats(app: &mut App) {
    if let Some(rx) = &app.stats.rx
        && let Ok(result) = rx.try_recv()
    {
        app.stats.is_loading = false;
        app.stats.rx = None;
        match result {
            Ok(data) => app.stats.sensors = data,
            Err(e) => app.stats.error = Some(e),
        }
    }
}

fn request_stats(app: &mut App, ctx: &egui::Context) {
    app.stats.is_loading = true;
    app.stats.loaded = true;
    app.stats.error = None;

    let (tx, rx) = mpsc::channel();
    app.stats.rx = Some(rx);

    let api_url = app.api_url;
    let api_key = app.api_key.clone();
    let ctx = ctx.clone();

    spawn(async move {
        let result =
            get_json::<Vec<SensorCompare>>(api_url, "/api/v1/stats/compare", &api_key).await;
        let _ = tx.send(result);
        ctx.request_repaint(); // despierta la UI cuando llega la respuesta
    });
}
