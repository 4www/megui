use eframe::egui;

use crate::components::ThemeMode;
use crate::config::Config;

pub struct SettingsModal;

impl SettingsModal {
    pub fn render(
        ctx: &egui::Context,
        config: &Config,
        theme_mode: &mut ThemeMode,
        open: &mut bool,
    ) {
        egui::Window::new("⚙ Settings")
            .open(open)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Theme");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.selectable_label(*theme_mode == ThemeMode::Auto, "Auto").clicked() {
                        *theme_mode = ThemeMode::Auto;
                    }
                    if ui.selectable_label(*theme_mode == ThemeMode::Light, "Light").clicked() {
                        *theme_mode = ThemeMode::Light;
                    }
                    if ui.selectable_label(*theme_mode == ThemeMode::Dark, "Dark").clicked() {
                        *theme_mode = ThemeMode::Dark;
                    }
                });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(15.0);

                ui.heading("Configuration");
                ui.add_space(5.0);

                egui::Grid::new("config_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.label(&config.app.name);
                        ui.end_row();

                        ui.label("Website:");
                        ui.hyperlink_to(&config.app.website, &config.app.website);
                        ui.end_row();

                        ui.label("Resume:");
                        ui.hyperlink_to(&config.app.resume, &config.app.resume);
                        ui.end_row();

                        ui.label("Artworks:");
                        let artworks_base = config.app.artworks.trim_end_matches("/index.json");
                        ui.hyperlink_to(artworks_base, artworks_base);
                        ui.end_row();

                        ui.label("Repository:");
                        ui.hyperlink_to(&config.app.repository, &config.app.repository);
                        ui.end_row();

                        ui.label("Current Theme:");
                        ui.label(theme_mode.as_str());
                        ui.end_row();
                    });

                ui.add_space(15.0);
            });
    }
}
