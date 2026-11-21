use eframe::egui::{self, CornerRadius, Margin, Vec2};
use frontend_honeypot::Views;

#[derive(Debug, Default)]
pub struct App {
    dark_mode: bool,
    current_view: Views,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.2);
        cc.egui_ctx.set_visuals(egui::Visuals {
            window_corner_radius: CornerRadius::same(8),
            menu_corner_radius: CornerRadius::same(8),
            striped: true,
            image_loading_spinners: true,
            ..egui::Visuals::light()
        });

        App::default()
    }

    fn set_theme(&self, ctx: &egui::Context) {
        let theme = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        // ctx.set_pixels_per_point(1.2);
        ctx.set_visuals(egui::Visuals {
            window_corner_radius: CornerRadius::same(8),
            menu_corner_radius: CornerRadius::same(8),
            striped: true,
            image_loading_spinners: true,
            ..theme
        });
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            if ui.button("Salir").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
            }

            if ui.checkbox(&mut self.dark_mode, "Modo oscuro").clicked() {
                self.set_theme(ui.ctx());
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        ui.style_mut().spacing.indent = 10.0;
        ui.style_mut().spacing.menu_margin = Margin::same(5);
        ui.style_mut().spacing.window_margin = Margin::same(5);
        ui.style_mut().spacing.item_spacing = Vec2::new(10.0, 5.0);

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
