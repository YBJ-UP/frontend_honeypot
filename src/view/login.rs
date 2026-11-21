use crate::App;
use eframe::egui::{self, CornerRadius, Layout, Margin, RichText, frame};

pub fn login_view(app: &mut App, ui: &mut egui::Ui) {
    let t =
        ui.ctx()
            .animate_bool_with_time(egui::Id::unique("theme_transition"), app.dark_mode, 0.25);

    let card_bg = egui::Color32::from_rgb(200, 200, 200)
        .lerp_to_gamma(egui::Color32::from_rgb(40, 40, 40), t);

    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        frame::Frame::default()
            .inner_margin(Margin::symmetric(25, 50))
            .fill(card_bg)
            .corner_radius(CornerRadius::same(16))
            .show(ui, |ui| {
                ui.set_width(300.0);
                ui.heading(RichText::new("¡Bienvenido de vuelta!").strong());
                ui.add_space(20.0);
                ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                    ui.label("Usuario:");
                });
                ui.text_edit_singleline(&mut app.user);
                ui.add_space(10.0);
                ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                    ui.label("Contraseña:");
                });
                ui.text_edit_singleline(&mut app.password)
            });
    });
}
