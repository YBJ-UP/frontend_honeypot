use std::sync::mpsc;

use crate::{App, LoginStatus, Views};
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, frame};
use serde::{Deserialize, Serialize};
use tokio::spawn;

pub fn login_view(app: &mut App, ui: &mut egui::Ui) {
    if let LoginStatus::LoggedIn = app.login_status {
        app.current_view = Views::MainDashboard;
    }
    let t =
        ui.ctx()
            .animate_bool_with_time(egui::Id::unique("theme_transition"), app.dark_mode, 0.25);

    let card_bg = egui::Color32::from_rgb(200, 200, 200)
        .lerp_to_gamma(egui::Color32::from_rgb(40, 40, 40), t);

    if let Some(rx) = &app.login_rx
        && let Ok(res) = rx.try_recv()
    {
        match res {
            Ok(key) => {
                app.login_rx = None;
                app.password.clear();
                app.login_status = LoginStatus::LoggedIn;
                app.api_key = key;
                app.current_view = Views::MainDashboard;
            }
            Err(msg) => {
                app.login_status = LoginStatus::LoggedIn(msg);
            }
        }
    }

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

                ui.add(egui::TextEdit::singleline(&mut app.user).hint_text("Usuario"));

                ui.add_space(10.0);

                let pw_field = ui.add(
                    egui::TextEdit::singleline(&mut app.password)
                        .password(true)
                        .hint_text("Contraseña"),
                );

                let pw_enter_pressed =
                    pw_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                ui.add_space(20.0);

                ui.add_enabled_ui(
                    matches!(
                        app.login_status,
                        LoginStatus::NotLoggedIn | LoginStatus::Failed(_)
                    ),
                    |ui| {
                        if (ui.button("Iniciar sesión").clicked() || pw_enter_pressed)
                            && !app.user.is_empty()
                            && !app.password.is_empty()
                        {
                            app.login_status = LoginStatus::Loading;

                            let (tx, rx) = mpsc::channel();
                            app.login_rx = Some(rx);

                            let api_url = app.api_url;
                            let username = app.user.clone();
                            let password = app.password.clone();

                            spawn(async move {
                                let result = login(api_url, &username, &password).await;
                                let _ = tx.send(result);
                            });
                        }
                    },
                );

                match &app.login_status {
                    LoginStatus::Loading => {
                        ui.add_space(10.0);
                        ui.spinner();
                    }
                    LoginStatus::Failed(msg) => {
                        ui.add_space(10.0);
                        ui.colored_label(Color32::RED, msg);
                    }
                    _ => {}
                }
            });
    });
}

#[derive(Serialize, Deserialize)]
struct Credentials<'a> {
    name: &'a str,
    password: &'a str,
}

#[derive(Serialize, Deserialize)]
struct ApiLoginResponse {
    key: String,
}

async fn login(url: &str, user: &str, pw: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let credentials = Credentials {
        name: user,
        password: pw,
    };

    let url = format!("{}/api/v1/auth/login", url);

    let res = client
        .post(&url)
        .json(&credentials)
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {e}"))?;

    if res.status().is_success() {
        let data: ApiLoginResponse = res
            .json()
            .await
            .map_err(|e| format!("Error en la respuesta del servidor: {}", e))?;

        Ok(data.key)
    } else if res.status() == 401 || res.status() == 403 {
        Err("Usuario o contraseña inválidos".to_string())
    } else {
        Err(format!("Error del servidor ({})", res.status()))
    }
}
