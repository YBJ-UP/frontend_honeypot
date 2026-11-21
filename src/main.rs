use eframe::egui::{self, CornerRadius, Margin, Vec2};
use frontend_honeypot::Views;

#[derive(Debug, Default)]
pub struct App {
    dark_mode: bool,
    current_view: Views,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let app = Self::default();
        app.set_theme(&cc.egui_ctx);

        app
    }

    fn set_theme(&self, ctx: &egui::Context) {
        let theme = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_pixels_per_point(1.2);
        ctx.set_visuals(egui::Visuals {
            window_corner_radius: CornerRadius::same(8),
            menu_corner_radius: CornerRadius::same(8),
            striped: true,
            image_loading_spinners: true,
            ..theme
        });

        ctx.all_styles_mut(|style| {
            style.spacing.indent = 10.0;
            style.spacing.menu_margin = Margin::same(5);
            style.spacing.window_margin = Margin::same(5);
            style.spacing.item_spacing = Vec2::new(10.0, 5.0);
        });
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let mut frame = egui::frame::Frame::central_panel(ui.style());
        frame.inner_margin = Margin::same(10);
        egui::Panel::top("top_panel").frame(frame).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Salir").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.dark_mode, "Modo oscuro").clicked() {
                        self.set_theme(ui.ctx());
                    }
                });
            });
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        self.top_bar(ui);

        egui::CentralPanel::default_margins().show(ui, |ui| {
            ui.heading("Hola mundo");
            ui.label("asaksbdajhb")
        });
    }
}

fn main() -> eframe::Result {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 1080.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Visualizador de honeypot",
        opts,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
