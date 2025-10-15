use eframe::egui;
use egui_commonmark::CommonMarkCache;

use crate::config::Config;

pub struct ResumePage;

impl ResumePage {
    pub fn render(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        config: &Config,
        resume_content: &Option<String>,
        markdown_cache: &mut CommonMarkCache,
    ) {
        ui.heading("Resume");
        ui.add_space(10.0);

        if ui.button("🔗 Open in New Tab").clicked() {
            ctx.open_url(egui::OpenUrl::new_tab(&config.app.resume));
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(markdown) = resume_content {
                // Render markdown with proper formatting
                egui_commonmark::CommonMarkViewer::new()
                    .show(ui, markdown_cache, markdown);
            } else {
                ui.label("Loading resume...");
            }
        });
    }
}
