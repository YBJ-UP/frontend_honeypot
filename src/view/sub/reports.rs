use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::{collections::HashMap, sync::mpsc};
use tokio::spawn;

use crate::{
    App,
    api::get_json,
    model::report::{GeneralReport, IpRow, Period, ReportSection},
    view::widgets::{bar_row, fmt_ts, severity_color, stat_card},
};

pub fn reports_view(app: &mut App, ui: &mut egui::Ui) {
    poll_reports(app);

    // Carga automática la primera vez que se entra a la pestaña
    if !app.report.loaded {
        request_reports(app, ui.ctx());
    }

    header(app, ui);

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    if let Some(err) = &app.report.error {
        ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
        ui.add_space(10.0);
    }

    let state = &mut app.report;
    let Some(data) = &state.data else {
        if !state.is_loading && state.error.is_none() {
            ui.label("Sin datos.");
        }
        return;
    };

    summary(ui, data);
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        for section in ReportSection::ALL {
            ui.selectable_value(&mut state.section, section, section.label());
        }
    });
    ui.separator();
    ui.add_space(10.0);

    match state.section {
        ReportSection::PorIp => ips_table(ui, data, &mut state.filter),
        ReportSection::Distribucion => distribution(ui, data),
        ReportSection::Top => tops(ui, data),
    }
}

fn header(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Reporte general");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Actualizar").clicked() && !app.report.is_loading {
                request_reports(app, ui.ctx());
            }

            let before = app.report.period;
            egui::ComboBox::from_id_salt("report_period")
                .selected_text(app.report.period.label())
                .show_ui(ui, |ui| {
                    for p in Period::ALL {
                        ui.selectable_value(&mut app.report.period, p, p.label());
                    }
                });
            // Cambiar de periodo siempre relanza la petición
            if app.report.period != before {
                request_reports(app, ui.ctx());
            }

            if app.report.is_loading {
                ui.spinner();
            }
        });
    });
}

fn summary(ui: &mut egui::Ui, data: &GeneralReport) {
    let max_severity = data
        .by_severity
        .iter()
        .filter(|(_, count)| **count > 0)
        .filter_map(|(sev, _)| sev.parse::<u8>().ok())
        .max()
        .unwrap_or(0);

    ui.horizontal(|ui| {
        stat_card(ui, "Ataques cerrados", data.totals.attacks.to_string());
        stat_card(ui, "IPs únicas", data.totals.unique_ips.to_string());
        stat_card(ui, "Severidad máx.", max_severity.to_string());
    });

    ui.add_space(5.0);

    let since = match data.since {
        Some(ts) => format!("desde {}", fmt_ts(ts)),
        None => "todo el historial".to_string(),
    };
    ui.label(
        egui::RichText::new(format!(
            "Periodo: {} ({}) · generado {}",
            data.period,
            since,
            fmt_ts(data.generated_at)
        ))
        .size(12.0)
        .weak(),
    );
}

// ---------- Sección: Por IP ----------

fn ips_table(ui: &mut egui::Ui, data: &GeneralReport, filter: &mut String) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(filter)
                .hint_text("Filtrar por IP o país")
                .desired_width(220.0),
        );
        ui.label(
            egui::RichText::new("Hasta 50 IPs, ordenadas por severidad y ataques")
                .size(12.0)
                .weak(),
        );
    });
    ui.add_space(8.0);

    let needle = filter.trim().to_lowercase();
    let rows: Vec<&IpRow> = data
        .ips
        .iter()
        .filter(|r| {
            needle.is_empty()
                || r.ip.to_lowercase().contains(&needle)
                || r.country
                    .as_deref()
                    .is_some_and(|c| c.to_lowercase().contains(&needle))
        })
        .collect();

    if rows.is_empty() {
        ui.label("Sin resultados.");
        return;
    }

    // tipo (clave) -> etiqueta legible, para el tooltip
    let labels: HashMap<&str, &str> = data
        .by_type
        .iter()
        .map(|t| (t.kind.as_str(), t.label.as_str()))
        .collect();

    TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(120.0)) // IP
        .column(Column::initial(50.0)) // País
        .column(Column::initial(65.0)) // Ataques
        .column(Column::initial(60.0)) // Severidad
        .column(Column::initial(140.0)) // Alcance máximo
        .column(Column::initial(110.0)) // Sensores
        .column(Column::remainder()) // Último evento
        .min_scrolled_height(0.0)
        .header(24.0, |mut header| {
            for title in [
                "IP",
                "País",
                "Ataques",
                "Sev. máx.",
                "Alcance máximo",
                "Sensores",
                "Último evento",
            ] {
                header.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|body| {
            body.rows(24.0, rows.len(), |mut row| {
                let r = rows[row.index()];

                let types = r
                    .types
                    .iter()
                    .map(|t| labels.get(t.as_str()).copied().unwrap_or(t.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ");
                let hover = format!("Tipos: {}\nPrimer evento: {}", types, fmt_ts(r.first_seen));

                row.col(|ui| {
                    ui.label(&r.ip).on_hover_text(hover);
                });
                row.col(|ui| {
                    ui.label(r.country.as_deref().unwrap_or("—"));
                });
                row.col(|ui| {
                    ui.label(r.attacks.to_string());
                });
                row.col(|ui| {
                    let color = severity_color(r.max_severity, ui.visuals().text_color());
                    ui.colored_label(color, r.max_severity.to_string());
                });
                row.col(|ui| {
                    ui.label(&r.furthest_reached);
                });
                row.col(|ui| {
                    ui.label(r.sensors.join(", "));
                });
                row.col(|ui| {
                    ui.label(fmt_ts(r.last_seen));
                });
            });
        });
}

// ---------- Sección: Distribución ----------

fn distribution(ui: &mut egui::Ui, data: &GeneralReport) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.strong("Por tipo de ataque");
            let max = data.by_type.iter().map(|t| t.count).max().unwrap_or(0);
            for t in &data.by_type {
                bar_row(ui, &t.label, t.count, max, None);
            }

            ui.add_space(15.0);
            ui.strong("Por severidad");
            let severities: Vec<(u8, u32)> = (1..=5u8)
                .map(|s| {
                    let count = data.by_severity.get(&s.to_string()).copied().unwrap_or(0);
                    (s, count)
                })
                .collect();
            let max = severities.iter().map(|(_, c)| *c).max().unwrap_or(0);
            let base = ui.visuals().selection.bg_fill;
            for (sev, count) in severities {
                bar_row(
                    ui,
                    &format!("Severidad {sev}"),
                    count,
                    max,
                    Some(severity_color(sev, base)),
                );
            }

            ui.add_space(15.0);
            ui.strong("Por sensor");
            let max = data.by_sensor.iter().map(|s| s.count).max().unwrap_or(0);
            for s in &data.by_sensor {
                bar_row(ui, &s.sensor, s.count, max, None);
            }
        });
}

// ---------- Sección: Top ----------

fn tops(ui: &mut egui::Ui, data: &GeneralReport) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                top_list(
                    &mut cols[0],
                    "IPs más activas",
                    data.top_ips.iter().map(|i| (i.ip.as_str(), i.attacks)),
                );
                top_list(
                    &mut cols[0],
                    "Países",
                    data.top_countries
                        .iter()
                        .map(|v| (v.value.as_str(), v.count)),
                );
                top_list(
                    &mut cols[1],
                    "Usuarios probados",
                    data.top_usernames
                        .iter()
                        .map(|v| (v.value.as_str(), v.count)),
                );
                top_list(
                    &mut cols[1],
                    "Contraseñas probadas",
                    data.top_passwords
                        .iter()
                        .map(|v| (v.value.as_str(), v.count)),
                );
            });
        });
}

fn top_list<'a>(ui: &mut egui::Ui, title: &str, items: impl Iterator<Item = (&'a str, u32)>) {
    ui.strong(title);
    let items: Vec<(&str, u32)> = items.collect();
    let max = items.iter().map(|(_, c)| *c).max().unwrap_or(0);

    if items.is_empty() {
        ui.weak("Sin datos");
    }
    for (label, count) in items {
        bar_row(ui, label, count, max, None);
    }
    ui.add_space(15.0);
}

// ---------- Red ----------

fn poll_reports(app: &mut App) {
    if let Some(rx) = &app.report.rx
        && let Ok(result) = rx.try_recv()
    {
        app.report.is_loading = false;
        app.report.rx = None;
        match result {
            Ok(data) => app.report.data = Some(data),
            Err(e) => app.report.error = Some(e),
        }
    }
}

fn request_reports(app: &mut App, ctx: &egui::Context) {
    app.report.is_loading = true;
    app.report.loaded = true;
    app.report.error = None;

    let (tx, rx) = mpsc::channel();
    app.report.rx = Some(rx);

    let api_url = app.api_url;
    let api_key = app.api_key.clone();
    let path = format!("/api/v1/reports?period={}", app.report.period.as_query());
    let ctx = ctx.clone();

    spawn(async move {
        let result = get_json::<GeneralReport>(api_url, &path, &api_key).await;
        let _ = tx.send(result);
        ctx.request_repaint(); // despierta la UI cuando llega la respuesta
    });
}
