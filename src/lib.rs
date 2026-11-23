use std::sync::mpsc::Receiver;

use crate::view::login::login_view;
use eframe::egui::{self, Color32, CornerRadius, Margin, Vec2};

pub mod view {
    pub mod login;
}

#[derive(Debug, Default)]
pub enum LoginStatus {
    #[default]
    NotLoggedIn,
    Loading,
    LoggedIn(String), // Llave
    Failed(String),   // Razón de fallo
}

#[derive(Debug, Default)]
pub enum Views {
    #[default]
    Login,
    MainDashboard,
    Stats,
    Details,
}

#[derive(Debug)]
pub struct App {
    pub dark_mode: bool,
    pub current_view: Views,
    pub user: String,
    pub password: String,
    pub api_url: &'static str,
    pub login_status: LoginStatus,
    pub login_rx: Option<Receiver<Result<String, String>>>,
}

impl Default for App {
    fn default() -> Self {
        App {
            dark_mode: true,
            api_url: option_env!("API_KEY").unwrap_or("http://localhost:8000"),
            current_view: Views::default(),
            user: String::default(),
            password: String::default(),
            login_status: LoginStatus::NotLoggedIn,
            login_rx: None,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let app = Self::default();
        app.set_theme(&cc.egui_ctx);

        app
    }

    fn set_theme(&self, ctx: &egui::Context) {
        // Anima de 0.0 (Light) a 1.0 (Dark) en 0.25 segundos
        let t =
            ctx.animate_bool_with_time(egui::Id::unique("theme_transition"), self.dark_mode, 0.5);

        ctx.set_theme(if t >= 0.5 {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });

        let mut light = egui::Visuals::light();
        let mut dark = egui::Visuals::dark();

        light.panel_fill = Color32::from_rgb(230, 230, 230);
        dark.panel_fill = Color32::from_rgb(30, 30, 30);

        // Si ya terminó la transición, usa los visuales directos sin calcular interpolación
        let visuals = if t <= 0.0 {
            light
        } else if t >= 1.0 {
            dark
        } else {
            lerp_visuals(&light, &dark, t)
        };

        ctx.set_pixels_per_point(1.2);
        ctx.set_visuals(egui::Visuals {
            window_corner_radius: CornerRadius::same(8),
            menu_corner_radius: CornerRadius::same(8),
            striped: true,
            image_loading_spinners: true,
            ..visuals
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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Salir").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
                    }
                    ui.checkbox(&mut self.dark_mode, "Modo oscuro")
                });
            });
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        self.set_theme(ui.ctx());
        self.top_bar(ui);

        egui::CentralPanel::default_margins().show(ui, |ui| {
            match self.current_view {
                Views::Login => login_view(self, ui),
                _ => {
                    ui.label("lol");
                }
            };
        });
    }
}

fn lerp_stroke(a: egui::Stroke, b: egui::Stroke, t: f32) -> egui::Stroke {
    egui::Stroke::new(
        egui::lerp(a.width..=b.width, t),
        a.color.lerp_to_gamma(b.color, t),
    )
}

fn lerp_widget_visuals(
    a: &egui::style::WidgetVisuals,
    b: &egui::style::WidgetVisuals,
    t: f32,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: a.bg_fill.lerp_to_gamma(b.bg_fill, t),
        weak_bg_fill: a.weak_bg_fill.lerp_to_gamma(b.weak_bg_fill, t),
        bg_stroke: lerp_stroke(a.bg_stroke, b.bg_stroke, t),
        fg_stroke: lerp_stroke(a.fg_stroke, b.fg_stroke, t),
        ..if t >= 0.5 { *b } else { *a }
    }
}

fn lerp_visuals(light: &egui::Visuals, dark: &egui::Visuals, t: f32) -> egui::Visuals {
    let base = if t >= 0.5 {
        dark.clone()
    } else {
        light.clone()
    };
    egui::Visuals {
        panel_fill: light.panel_fill.lerp_to_gamma(dark.panel_fill, t),
        window_fill: light.window_fill.lerp_to_gamma(dark.window_fill, t),
        faint_bg_color: light.faint_bg_color.lerp_to_gamma(dark.faint_bg_color, t),
        extreme_bg_color: light
            .extreme_bg_color
            .lerp_to_gamma(dark.extreme_bg_color, t),
        code_bg_color: light.code_bg_color.lerp_to_gamma(dark.code_bg_color, t),
        hyperlink_color: light.hyperlink_color.lerp_to_gamma(dark.hyperlink_color, t),
        window_stroke: lerp_stroke(light.window_stroke, dark.window_stroke, t),
        selection: egui::style::Selection {
            bg_fill: light
                .selection
                .bg_fill
                .lerp_to_gamma(dark.selection.bg_fill, t),
            stroke: lerp_stroke(light.selection.stroke, dark.selection.stroke, t),
        },
        widgets: egui::style::Widgets {
            noninteractive: lerp_widget_visuals(
                &light.widgets.noninteractive,
                &dark.widgets.noninteractive,
                t,
            ),
            inactive: lerp_widget_visuals(&light.widgets.inactive, &dark.widgets.inactive, t),
            hovered: lerp_widget_visuals(&light.widgets.hovered, &dark.widgets.hovered, t),
            active: lerp_widget_visuals(&light.widgets.active, &dark.widgets.active, t),
            open: lerp_widget_visuals(&light.widgets.open, &dark.widgets.open, t),
        },
        ..base
    }
}
