use eframe::egui;
use serde_json::Value;
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
use tokio::spawn;

use crate::{
    App,
    api::get_json,
    model::sensors::Sensor,
    view::widgets::{fmt_duration, fmt_ts, stat_card},
};

const AUTO_REFRESH: Duration = Duration::from_secs(10);
const CARD_WIDTH: f32 = 280.0;
const ONLINE: egui::Color32 = egui::Color32::from_rgb(80, 200, 120);

pub fn sensors_view(app: &mut App, ui: &mut egui::Ui) {
    poll_sensors(app);

    // Carga inicial y actualización automática
    let refresh_due = app.sensors.auto_refresh
        && !app.sensors.is_loading
        && app
            .sensors
            .last_fetch
            .is_none_or(|t| t.elapsed() >= AUTO_REFRESH);
    if !app.sensors.loaded || refresh_due {
        request_sensors(app, ui.ctx());
    }
    if app.sensors.auto_refresh {
        // Garantiza que la UI se reevalúe aunque el usuario no mueva el mouse
        ui.ctx().request_repaint_after(Duration::from_secs(1));
    }

    header(app, ui);

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    if let Some(err) = &app.sensors.error {
        ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
        ui.add_space(10.0);
    }

    let sensors = &app.sensors.list;

    if sensors.is_empty() {
        if app.sensors.loaded && !app.sensors.is_loading && app.sensors.error.is_none() {
            ui.label("No hay sensores registrados todavía.");
        }
        return;
    }

    // Resumen
    let online = sensors.iter().filter(|s| s.online).count();
    ui.horizontal(|ui| {
        stat_card(ui, "Sensores", sensors.len().to_string());
        stat_card(ui, "En línea", online.to_string());
        stat_card(ui, "Sin conexión", (sensors.len() - online).to_string());
    });
    ui.add_space(15.0);

    // Tarjetas en filas; el número de columnas depende del ancho disponible
    let spacing = ui.spacing().item_spacing.x;
    let columns = (((ui.available_width() + spacing) / (CARD_WIDTH + spacing)) as usize).max(1);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for chunk in sensors.chunks(columns) {
                ui.horizontal_top(|ui| {
                    for sensor in chunk {
                        sensor_card(ui, sensor);
                    }
                });
                ui.add_space(5.0);
            }
        });
}

fn header(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Sensores");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Actualizar").clicked() && !app.sensors.is_loading {
                request_sensors(app, ui.ctx());
            }
            ui.checkbox(&mut app.sensors.auto_refresh, "Auto (10 s)");
            if app.sensors.is_loading {
                ui.spinner();
            }
        });
    });
}

fn sensor_card(ui: &mut egui::Ui, s: &Sensor) {
    let (color, status) = if s.online {
        (ONLINE, "En línea")
    } else {
        (egui::Color32::RED, "Sin conexión")
    };

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(CARD_WIDTH);

        ui.horizontal(|ui| {
            ui.colored_label(color, "●");
            ui.label(egui::RichText::new(&s.name).heading());
        });
        ui.colored_label(color, status);
        ui.add_space(6.0);

        egui::Grid::new(("sensor_grid", &s.name))
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Última señal").weak());
                ui.label(format!("hace {}", fmt_duration(s.seconds_since_seen)))
                    .on_hover_text(fmt_ts(s.last_seen));
                ui.end_row();

                ui.label(egui::RichText::new("Registrado").weak());
                ui.label(fmt_ts(s.first_seen));
                ui.end_row();
            });

        if !s.online && s.offline_alerted {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Alerta de caída enviada")
                    .size(12.0)
                    .weak(),
            );
        }

        // Datos extra del heartbeat, solo si hay algo que mostrar
        if !s.info.is_empty() {
            ui.add_space(4.0);
            egui::CollapsingHeader::new(format!("Información ({})", s.info.len()))
                .id_salt(("sensor_info", &s.name))
                .show(ui, |ui| {
                    egui::Grid::new(("sensor_info_grid", &s.name))
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            for (key, value) in &s.info {
                                ui.label(egui::RichText::new(key).weak());
                                ui.label(show_value(value));
                                ui.end_row();
                            }
                        });
                });
        }
    });
}

/// Los strings se muestran sin comillas; el resto (números, listas, objetos) como JSON.
fn show_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

// ---------- Red ----------

fn poll_sensors(app: &mut App) {
    if let Some(rx) = &app.sensors.rx
        && let Ok(result) = rx.try_recv()
    {
        app.sensors.is_loading = false;
        app.sensors.rx = None;
        match result {
            Ok(mut data) => {
                // Primero los caídos (requieren atención), luego por nombre
                data.sort_by(|a, b| a.online.cmp(&b.online).then_with(|| a.name.cmp(&b.name)));
                app.sensors.list = data;
            }
            Err(e) => app.sensors.error = Some(e),
        }
    }
}

fn request_sensors(app: &mut App, ctx: &egui::Context) {
    app.sensors.is_loading = true;
    app.sensors.loaded = true;
    app.sensors.error = None;
    app.sensors.last_fetch = Some(Instant::now());

    let (tx, rx) = mpsc::channel();
    app.sensors.rx = Some(rx);

    let api_url = app.api_url;
    let api_key = app.api_key.clone();
    let ctx = ctx.clone();

    spawn(async move {
        let result = get_json::<Vec<Sensor>>(api_url, "/api/v1/sensors", &api_key).await;
        let _ = tx.send(result);
        ctx.request_repaint(); // despierta la UI cuando llega la respuesta
    });
}
