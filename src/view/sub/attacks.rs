use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::{
    net::IpAddr,
    sync::mpsc,
    time::{Duration, Instant},
};
use tokio::spawn;

use crate::{
    App,
    api::get_json,
    model::attacks::{Attack, AttackSort, StatusFilter},
    view::widgets::{fmt_duration, fmt_ts, severity_color, type_label},
};

const AUTO_REFRESH: Duration = Duration::from_secs(10);
const LIMIT: usize = 100;

pub fn attacks_view(app: &mut App, ui: &mut egui::Ui) {
    poll_attacks(app);

    // Carga inicial y actualización automática
    let refresh_due = app.attack.auto_refresh
        && !app.attack.is_loading
        && app
            .attack
            .last_fetch
            .is_none_or(|t| t.elapsed() >= AUTO_REFRESH);
    if !app.attack.loaded || refresh_due {
        request_attacks(app, ui.ctx());
    }
    if app.attack.auto_refresh {
        // Garantiza que la UI se reevalúe aunque el usuario no mueva el mouse
        ui.ctx().request_repaint_after(Duration::from_secs(1));
    }

    header(app, ui);

    ui.add_space(10.0);
    filters(app, ui);
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    if let Some(err) = &app.attack.error {
        ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
        ui.add_space(10.0);
    }

    if app.attack.loaded
        && !app.attack.is_loading
        && app.attack.error.is_none()
        && app.attack.list.is_empty()
    {
        ui.label("No hay ataques registrados.");
        return;
    }

    // Filas visibles: filtro en cliente + orden
    let needle = app.attack.search.trim().to_lowercase();
    let min_sev = app.attack.min_severity;
    let mut rows: Vec<&Attack> = app
        .attack
        .list
        .iter()
        .filter(|a| a.severity >= min_sev)
        .filter(|a| {
            needle.is_empty()
                || a.ip.to_lowercase().contains(&needle)
                || a.sensor.to_lowercase().contains(&needle)
                || a.attack_type.to_lowercase().contains(&needle)
                || label_of(a).to_lowercase().contains(&needle)
                || a.country
                    .as_deref()
                    .is_some_and(|c| c.to_lowercase().contains(&needle))
        })
        .collect();
    sort_rows(&mut rows, app.attack.sort, app.attack.sort_asc);

    // Resumen
    let open = app
        .attack
        .list
        .iter()
        .filter(|a| a.status == "open")
        .count();
    let mut info = format!(
        "Mostrando {} de {} cargados · {} en curso",
        rows.len(),
        app.attack.list.len(),
        open
    );
    if app.attack.list.len() >= LIMIT {
        info.push_str(&format!(" · el servidor limita a {LIMIT}"));
    }
    ui.label(egui::RichText::new(info).size(12.0).weak());
    ui.add_space(8.0);

    if rows.is_empty() {
        if !app.attack.list.is_empty() {
            ui.label("Ningún ataque coincide con los filtros.");
        }
        return;
    }

    // Copias locales para poder mutarlas desde los encabezados
    let mut sort = app.attack.sort;
    let mut asc = app.attack.sort_asc;

    TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(45.0)) // ID
        .column(Column::initial(115.0)) // IP
        .column(Column::initial(105.0)) // Sensor
        .column(Column::initial(95.0)) // Tipo
        .column(Column::initial(75.0)) // Severidad
        .column(Column::initial(75.0)) // Estado
        .column(Column::initial(60.0)) // Eventos
        .column(Column::remainder()) // Inicio
        .min_scrolled_height(0.0)
        .header(26.0, |mut header| {
            header.col(|ui| sort_header(ui, "ID", AttackSort::Id, &mut sort, &mut asc));
            header.col(|ui| sort_header(ui, "IP", AttackSort::Ip, &mut sort, &mut asc));
            header.col(|ui| sort_header(ui, "Sensor", AttackSort::Sensor, &mut sort, &mut asc));
            header.col(|ui| sort_header(ui, "Tipo", AttackSort::Type, &mut sort, &mut asc));
            header.col(|ui| sort_header(ui, "Sev.", AttackSort::Severity, &mut sort, &mut asc));
            header.col(|ui| sort_header(ui, "Estado", AttackSort::Status, &mut sort, &mut asc));
            header.col(|ui| {
                ui.strong("Eventos");
            });
            header.col(|ui| {
                ui.strong("Inicio");
            });
        })
        .body(|body| {
            body.rows(24.0, rows.len(), |mut row| {
                let a = rows[row.index()];

                row.col(|ui| {
                    ui.label(a.id.to_string());
                });
                row.col(|ui| {
                    ui.label(&a.ip).on_hover_text(format!(
                        "País: {}",
                        a.country.as_deref().unwrap_or("desconocido")
                    ));
                });
                row.col(|ui| {
                    ui.label(&a.sensor);
                });
                row.col(|ui| {
                    ui.label(label_of(a));
                });
                row.col(|ui| {
                    let color = severity_color(a.severity, ui.visuals().text_color());
                    ui.colored_label(color, a.severity.to_string());
                });
                row.col(|ui| status_badge(ui, &a.status));
                row.col(|ui| {
                    ui.label(a.event_count.to_string());
                });
                row.col(|ui| {
                    let mut hover = format!("Último evento: {}", fmt_ts(a.last_seen));
                    if let Some(d) = a.duration {
                        hover.push_str(&format!("\nDuración: {}", fmt_duration(d)));
                    }
                    if let Some(t) = a.closed_at {
                        hover.push_str(&format!("\nCerrado: {}", fmt_ts(t)));
                    }
                    ui.label(fmt_ts(a.first_seen)).on_hover_text(hover);
                });
            });
        });

    app.attack.sort = sort;
    app.attack.sort_asc = asc;
}

/// Etiqueta del servidor ("Intrusión"); si viniera vacía, una formateada localmente.
fn label_of(a: &Attack) -> String {
    if a.type_label.is_empty() {
        type_label(&a.attack_type)
    } else {
        a.type_label.clone()
    }
}

fn header(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Ataques registrados");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Actualizar").clicked() && !app.attack.is_loading {
                request_attacks(app, ui.ctx());
            }
            ui.checkbox(&mut app.attack.auto_refresh, "Auto (10 s)");
            if app.attack.is_loading {
                ui.spinner();
            }
        });
    });
}

fn filters(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.attack.search)
                .hint_text("Buscar IP, sensor, tipo o país")
                .desired_width(220.0),
        );

        // El estado se filtra en el servidor: cambiarlo relanza la petición
        ui.label("Estado");
        let before = app.attack.status_filter;
        egui::ComboBox::from_id_salt("attack_status")
            .selected_text(app.attack.status_filter.label())
            .show_ui(ui, |ui| {
                for s in StatusFilter::ALL {
                    ui.selectable_value(&mut app.attack.status_filter, s, s.label());
                }
            });
        if app.attack.status_filter != before {
            request_attacks(app, ui.ctx());
        }

        // La severidad se filtra en el cliente
        ui.label("Severidad");
        egui::ComboBox::from_id_salt("attack_severity")
            .selected_text(severity_label(app.attack.min_severity))
            .show_ui(ui, |ui| {
                for s in [0u8, 3, 4, 5] {
                    ui.selectable_value(&mut app.attack.min_severity, s, severity_label(s));
                }
            });
    });
}

fn severity_label(min: u8) -> String {
    match min {
        0 => "Todas".to_string(),
        5 => "5".to_string(),
        n => format!("≥ {n}"),
    }
}

/// Encabezado clicable que alterna entre ascendente y descendente.
fn sort_header(
    ui: &mut egui::Ui,
    title: &str,
    key: AttackSort,
    sort: &mut AttackSort,
    asc: &mut bool,
) {
    let active = *sort == key;
    let arrow = match (active, *asc) {
        (false, _) => "",
        (true, true) => " ↑",
        (true, false) => " ↓",
    };
    let text = egui::RichText::new(format!("{title}{arrow}")).strong();

    if ui.add(egui::Button::new(text).frame(false)).clicked() {
        if active {
            *asc = !*asc;
        } else {
            *sort = key;
            // ID y severidad empiezan de mayor a menor; el resto, alfabético
            *asc = !matches!(key, AttackSort::Id | AttackSort::Severity);
        }
    }
}

fn sort_rows(rows: &mut [&Attack], key: AttackSort, asc: bool) {
    rows.sort_by(|a, b| {
        let ord = match key {
            AttackSort::Id => a.id.cmp(&b.id),
            AttackSort::Ip => {
                // Como direcciones, no como texto ("10.0.0.2" < "10.0.0.10")
                let pa = a.ip.parse::<IpAddr>().ok();
                let pb = b.ip.parse::<IpAddr>().ok();
                pa.cmp(&pb).then_with(|| a.ip.cmp(&b.ip))
            }
            AttackSort::Sensor => a.sensor.cmp(&b.sensor).then(a.id.cmp(&b.id)),
            AttackSort::Type => a.attack_type.cmp(&b.attack_type).then(a.id.cmp(&b.id)),
            AttackSort::Severity => a.severity.cmp(&b.severity).then(a.id.cmp(&b.id)),
            AttackSort::Status => a.status.cmp(&b.status).then(a.id.cmp(&b.id)),
        };
        if asc { ord } else { ord.reverse() }
    });
}

fn status_badge(ui: &mut egui::Ui, status: &str) {
    match status {
        "open" => {
            ui.colored_label(egui::Color32::from_rgb(80, 200, 120), "En curso");
        }
        "closed" => {
            ui.label(egui::RichText::new("Cerrado").weak());
        }
        other => {
            ui.label(other);
        }
    }
}

// ---------- Red ----------

fn poll_attacks(app: &mut App) {
    if let Some(rx) = &app.attack.rx
        && let Ok(result) = rx.try_recv()
    {
        app.attack.is_loading = false;
        app.attack.rx = None;
        match result {
            Ok(data) => app.attack.list = data,
            Err(e) => app.attack.error = Some(e),
        }
    }
}

fn request_attacks(app: &mut App, ctx: &egui::Context) {
    app.attack.is_loading = true;
    app.attack.loaded = true;
    app.attack.error = None;
    app.attack.last_fetch = Some(Instant::now());

    let (tx, rx) = mpsc::channel();
    app.attack.rx = Some(rx);

    let api_url = app.api_url;
    let api_key = app.api_key.clone();
    let path = format!(
        "/api/v1/attacks?status={}&limit={}",
        app.attack.status_filter.as_query(),
        LIMIT
    );
    let ctx = ctx.clone();

    spawn(async move {
        let result = get_json::<Vec<Attack>>(api_url, &path, &api_key).await;
        let _ = tx.send(result);
        ctx.request_repaint(); // despierta la UI cuando llega la respuesta
    });
}
