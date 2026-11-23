use eframe::egui::{self, Margin};

use crate::{
    App, DashboardTab,
    view::sub::{
        attacks::attacks_view, reports::reports_view, sensors::sensors_view, stats::stats_view,
    },
};

pub fn dashboard_view(app: &mut App, ui: &mut egui::Ui) {
    side_panel(app, ui);

    let frame = egui::frame::Frame::central_panel(ui.style()).inner_margin(Margin::same(20));

    egui::CentralPanel::default()
        .frame(frame)
        .show(ui, |ui| match &app.current_tab {
            DashboardTab::Ataques => attacks_view(app, ui),
            DashboardTab::Reportes => reports_view(app, ui),
            DashboardTab::Estadisticas => stats_view(app, ui),
            DashboardTab::Sensores => sensors_view(app, ui),
        });
}

fn side_panel(app: &mut App, ui: &mut egui::Ui) {
    let mut frame = egui::frame::Frame::central_panel(ui.style());
    frame.inner_margin = Margin::same(0);

    egui::Panel::left("left_panel")
        .frame(frame)
        .resizable(false)
        .show(ui, |ui| {
            ui.add_space(20.0);

            ui.vertical_centered_justified(|ui| {
                ui.style_mut().spacing.button_padding = egui::vec2(20.0, 15.0);

                ui.selectable_value(&mut app.current_tab, DashboardTab::Ataques, "Ataques");
                ui.separator();
                ui.selectable_value(&mut app.current_tab, DashboardTab::Reportes, "Reportes");
                ui.separator();
                ui.selectable_value(
                    &mut app.current_tab,
                    DashboardTab::Estadisticas,
                    "Estadísticas",
                );
                ui.separator();
                ui.selectable_value(&mut app.current_tab, DashboardTab::Sensores, "Sensores");
                ui.separator();
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(15.0);

                egui::Frame::new()
                    .inner_margin(egui::Margin::same(15))
                    .show(ui, |ui| {
                        if ui.button("Cerrar sesión").clicked() {
                            app.login_status = crate::LoginStatus::NotLoggedIn;
                            app.user.clear();
                            app.password.clear();
                            app.attack = Default::default();
                            app.report = Default::default();
                            app.stats = Default::default();
                            app.sensors = Default::default();
                        }
                        let user = if app.user.is_empty() {
                            "Admin"
                        } else {
                            &app.user
                        };
                        ui.label(
                            egui::RichText::new(format!("Sesión iniciada como: {}", user))
                                .color(egui::Color32::DARK_GRAY)
                                .size(12.0),
                        );
                    });
            });
        });
}
