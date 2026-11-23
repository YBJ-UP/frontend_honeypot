use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::sync::mpsc;
use tokio::spawn;

use crate::{App, model::attacks::Attack};

pub fn attacks_view(app: &mut App, ui: &mut egui::Ui) {
    if let Some(rx) = &app.attack.rx
        && let Ok(result) = rx.try_recv()
    {
        app.attack.is_loading = false;
        match result {
            Ok(data) => app.attack.list = data,
            Err(e) => app.attack.error = Some(e),
        }
    }

    ui.horizontal(|ui| {
        ui.heading("Ataques Registrados");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Actualizar").clicked() && !app.attack.is_loading {
                app.attack.is_loading = true;
                app.attack.error = None;

                let (tx, rx) = mpsc::channel();
                app.attack.rx = Some(rx);

                let api_url = app.api_url;
                let api_key = app.api_key.clone();

                spawn(async move {
                    let result = fetch_attacks(api_url, &api_key).await;
                    let _ = tx.send(result);
                });
            }

            if app.attack.is_loading {
                ui.spinner();
            }
        });
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    if let Some(err) = &app.attack.error {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
        ui.add_space(10.0);
    }

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto()) // ID
        .column(Column::initial(120.0)) // IP
        .column(Column::initial(100.0)) // Sensor
        .column(Column::initial(150.0)) // Tipo
        .column(Column::initial(80.0)) // Severidad
        .column(Column::initial(80.0)) // Estado
        .min_scrolled_height(0.0);

    table
        .header(24.0, |mut header| {
            header.col(|ui| {
                ui.strong("ID");
            });
            header.col(|ui| {
                ui.strong("IP Atacante");
            });
            header.col(|ui| {
                ui.strong("Sensor");
            });
            header.col(|ui| {
                ui.strong("Tipo");
            });
            header.col(|ui| {
                ui.strong("Severidad");
            });
            header.col(|ui| {
                ui.strong("Estado");
            });
        })
        .body(|mut body| {
            for attack in &app.attack.list {
                body.row(24.0, |mut row| {
                    row.col(|ui| {
                        ui.label(attack.id.to_string());
                    });
                    row.col(|ui| {
                        ui.label(&attack.ip);
                    });
                    row.col(|ui| {
                        ui.label(&attack.sensor);
                    });
                    row.col(|ui| {
                        ui.label(&attack.attack_type);
                    });

                    // Colorear severidad
                    row.col(|ui| {
                        let color = match attack.severity {
                            5 => egui::Color32::RED,
                            4 => egui::Color32::from_rgb(255, 140, 0),
                            _ => ui.visuals().text_color(),
                        };
                        ui.colored_label(color, attack.severity.to_string());
                    });

                    row.col(|ui| {
                        ui.label(&attack.status);
                    });
                });
            }
        });
}

async fn fetch_attacks(api_url: &str, api_key: &str) -> Result<Vec<Attack>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/attacks?status=all&limit=100", api_url);

    let response = client
        .get(&url)
        .header("X-Admin-Key", api_key) // O "Authorization": format!("Bearer {}", api_key) si migraste a JWT
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if response.status().is_success() {
        response
            .json::<Vec<Attack>>()
            .await
            .map_err(|e| format!("Error procesando JSON: {}", e))
    } else {
        Err(format!("Error del servidor: {}", response.status()))
    }
}
