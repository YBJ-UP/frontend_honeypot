use eframe::egui::{self};
use frontend_honeypot::App;

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
