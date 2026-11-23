use eframe::egui;

pub fn stat_card(ui: &mut egui::Ui, title: &str, value: String) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(120.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).size(12.0).weak());
            ui.heading(value);
        });
    });
}

/// Fila "etiqueta + barra proporcional con el conteo dentro".
pub fn bar_row(ui: &mut egui::Ui, label: &str, count: u32, max: u32, fill: Option<egui::Color32>) {
    ui.horizontal(|ui| {
        ui.add_sized([120.0, 18.0], egui::Label::new(label).truncate());

        let fraction = if max == 0 {
            0.0
        } else {
            count as f32 / max as f32
        };
        let width = ui.available_width().clamp(60.0, 260.0);
        let mut bar = egui::ProgressBar::new(fraction)
            .desired_width(width)
            .text(count.to_string());
        if let Some(color) = fill {
            bar = bar.fill(color);
        }
        ui.add(bar);
    });
}

pub fn severity_color(severity: u8, default: egui::Color32) -> egui::Color32 {
    match severity {
        5 => egui::Color32::RED,
        4 => egui::Color32::from_rgb(255, 140, 0),
        _ => default,
    }
}

/// Epoch (segundos, f64) -> "2026-11-23 10:42" en hora local.
pub fn fmt_ts(ts: f64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|d| {
            d.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "—".to_string())
}
