use eframe::egui;

use crate::artwork::Artwork;
use crate::config::Config;

#[cfg(target_arch = "wasm32")]
use hframe::HtmlWindow;

pub struct ArtworksPage;

impl ArtworksPage {
    pub fn render(
        ui: &mut egui::Ui,
        artworks: &[Artwork],
        selected_artworks: &mut Vec<Artwork>,
        loading: bool,
        error: &Option<String>,
    ) {
        // Error display
        if let Some(error) = error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            ui.add_space(10.0);
        }

        // Artworks list
        Self::render_artworks_list(ui, artworks, selected_artworks, loading);
    }

    fn render_artworks_list(
        ui: &mut egui::Ui,
        artworks: &[Artwork],
        selected_artworks: &mut Vec<Artwork>,
        loading: bool,
    ) {
        if !artworks.is_empty() {
            ui.heading("Available Artworks:");
            ui.add_space(5.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for artwork in artworks {
                    ui.horizontal(|ui| {
                        if ui.button(&artwork.name).clicked() {
                            // Add to selected artworks if not already open
                            if !selected_artworks.iter().any(|a| a.name == artwork.name) {
                                selected_artworks.push(artwork.clone());
                            }
                        }
                        if let Some(info) = &artwork.info {
                            ui.label(info);
                        }
                    });
                }
            });
        } else if loading {
            ui.spinner();
            ui.label("Loading artworks...");
        }
    }

    pub fn render_artwork_modals(
        ctx: &egui::Context,
        config: &Config,
        selected_artworks: &mut Vec<Artwork>,
    ) {
        let mut to_remove = Vec::new();

        #[cfg(target_arch = "wasm32")]
        for (idx, artwork) in selected_artworks.iter().enumerate() {
            let mut open = true;
            let artworks_base = config.app.artworks.trim_end_matches("/index.json");
            let artwork_url = format!("{}/{}/", artworks_base, artwork.name);
            let iframe_content = format!(
                r#"<iframe src="{}" style="width: 100%; height: 100%; border: none;"></iframe>"#,
                artwork_url
            );

            HtmlWindow::new(&artwork.name)
                .id(&format!("artwork_window_{}", artwork.name))
                .open(&mut open)
                .content(&iframe_content)
                .show(ctx);

            if !open {
                to_remove.push(idx);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        for (idx, artwork) in selected_artworks.iter().enumerate() {
            let mut open = true;
            let artworks_base = config.app.artworks.trim_end_matches("/index.json");
            let artwork_url = format!("{}/{}/", artworks_base, artwork.name);

            egui::Window::new(&artwork.name)
                .id(egui::Id::new(format!("artwork_window_{}", artwork.name)))
                .open(&mut open)
                .resizable(true)
                .default_width(800.0)
                .default_height(600.0)
                .show(ctx, |ui| {
                    if let Some(info) = &artwork.info {
                        ui.label(info);
                        ui.add_space(5.0);
                    }

                    if ui.button("🔗 Open in New Tab").clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab(&artwork_url));
                    }

                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    ui.label("Note: iframe preview only available in web version.");
                    ui.label("Click 'Open in New Tab' to view the artwork.");
                });

            if !open {
                to_remove.push(idx);
            }
        }

        // Remove closed artworks in reverse order to maintain correct indices
        for idx in to_remove.iter().rev() {
            selected_artworks.remove(*idx);
        }
    }
}
