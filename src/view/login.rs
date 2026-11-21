use eframe::{
    App,
    egui::{self, Color32, CornerRadius, Margin, RichText, Theme, frame},
};

pub fn login_view(app: &mut impl App, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        frame::Frame::default()
            .inner_margin(Margin::symmetric(25, 50))
            .fill(match ui.ctx().theme() {
                Theme::Dark => Color32::from_rgb(7, 7, 7),
                Theme::Light => Color32::from_rgb(200, 200, 200),
            })
            .corner_radius(CornerRadius::same(16))
            .show(ui, |ui| {
                ui.set_width(300.0);
                ui.heading(RichText::new("¡Bienvenido de vuelta!").strong());
                ui.add_space(50.0);
                ui.label("asnkjdnakjsndaskjnd")
            });
    });
}
