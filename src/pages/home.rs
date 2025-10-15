use eframe::egui;

use crate::config::Config;

pub struct HomePage;

impl HomePage {
    pub fn render(ui: &mut egui::Ui, config: &Config) {
        ui.heading("Home");
        ui.add_space(5.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label("Welcome to my digital house made in Rust and WASM");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Visit my HTML/CSS/Javascript traditional website:");
            ui.hyperlink_to("hwww.org", &config.app.website);
        });
    }
}
