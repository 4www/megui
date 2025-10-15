use eframe::egui;

use crate::config::Config;

pub struct AboutPage;

impl AboutPage {
    pub fn render(ui: &mut egui::Ui, config: &Config) {
        ui.heading("About");
        ui.add_space(10.0);

        ui.label(format!(
            "{} - A simple artworks viewer and portfolio",
            config.app.name
        ));
        ui.add_space(10.0);

        ui.label("Built with:");
        ui.horizontal(|ui| {
            ui.label("•");
            ui.hyperlink_to("Rust", "https://www.rust-lang.org/");
        });
        ui.horizontal(|ui| {
            ui.label("•");
            ui.hyperlink_to("egui", "https://www.egui.rs/");
            ui.label("- Immediate mode GUI framework");
        });
        ui.horizontal(|ui| {
            ui.label("•");
            ui.hyperlink_to(
                "eframe",
                "https://github.com/emilk/egui/tree/master/crates/eframe",
            );
            ui.label("- Web and native support");
        });
        ui.add_space(10.0);
    }
}
