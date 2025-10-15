use eframe::egui;
use std::sync::mpsc::Receiver;

use crate::artwork::{Artwork, ArtworksResponse};

pub struct MeguiApp {
    show_modal: bool,
    pub artworks: Vec<Artwork>,
    pub loading: bool,
    pub error: Option<String>,
    pub selected_artwork: Option<Artwork>,
    pub fetch_receiver: Option<Receiver<ehttp::Result<ehttp::Response>>>,
}

impl Default for MeguiApp {
    fn default() -> Self {
        Self {
            show_modal: false,
            artworks: Vec::new(),
            loading: false,
            error: None,
            selected_artwork: None,
            fetch_receiver: None,
        }
    }
}

impl MeguiApp {
    fn process_fetch_response(&mut self) {
        if let Some(receiver) = &self.fetch_receiver {
            if let Ok(result) = receiver.try_recv() {
                self.loading = false;
                self.fetch_receiver = None;

                match result {
                    Ok(response) => {
                        if let Some(text) = response.text() {
                            match serde_json::from_str::<Vec<ArtworksResponse>>(text) {
                                Ok(data) => {
                                    if let Some(first) = data.first() {
                                        self.artworks = first.contents.clone();
                                    } else {
                                        self.error = Some("No data returned".to_string());
                                    }
                                }
                                Err(e) => {
                                    self.error = Some(format!("Parse error: {}", e));
                                }
                            }
                        } else {
                            self.error = Some("Failed to read response text".to_string());
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Fetch error: {}", e));
                    }
                }
            }
        }
    }

    fn render_fetch_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            if ui.button("Fetch Artworks").clicked() && !self.loading {
                self.loading = true;
                self.error = None;
                let ctx = ctx.clone();
                let (sender, receiver) = std::sync::mpsc::channel();
                self.fetch_receiver = Some(receiver);

                ehttp::fetch(
                    ehttp::Request::get("https://artworks.hwww.org/index.json"),
                    move |result| {
                        let _ = sender.send(result);
                        ctx.request_repaint();
                    },
                );
            }

            if self.loading {
                ui.spinner();
            }
        });
    }

    fn render_artworks_list(&mut self, ui: &mut egui::Ui) {
        if !self.artworks.is_empty() {
            ui.heading("Available Artworks:");
            ui.add_space(5.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for artwork in &self.artworks {
                    ui.horizontal(|ui| {
                        if ui.button(&artwork.name).clicked() {
                            self.selected_artwork = Some(artwork.clone());
                        }
                        if let Some(info) = &artwork.info {
                            ui.label(info);
                        }
                    });
                }
            });
        } else if !self.loading && self.error.is_none() {
            ui.label("Click 'Fetch Artworks' to load the gallery");
        }
    }

    fn render_artwork_modal(&mut self, ctx: &egui::Context) {
        if let Some(artwork) = &self.selected_artwork {
            let mut open = true;
            let artwork_url = format!("https://artworks.hwww.org/{}/", artwork.name);

            egui::Window::new(&artwork.name)
                .open(&mut open)
                .resizable(true)
                .default_width(600.0)
                .default_height(500.0)
                .show(ctx, |ui| {
                    if let Some(info) = &artwork.info {
                        ui.label(info);
                        ui.add_space(10.0);
                    }

                    ui.horizontal(|ui| {
                        if ui.button("🔗 Open in New Tab").clicked() {
                            ctx.open_url(egui::OpenUrl::new_tab(&artwork_url));
                        }

                        ui.label(&artwork_url);
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label("Note: egui cannot embed iframes. Click 'Open in New Tab' to view the artwork.");
                });

            if !open {
                self.selected_artwork = None;
            }
        }
    }
}

impl eframe::App for MeguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for fetch response
        self.process_fetch_response();

        if self.fetch_receiver.is_some() {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Artworks Gallery");
            ui.add_space(10.0);

            self.render_fetch_button(ui, ctx);

            // Error display
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            self.render_artworks_list(ui);
        });

        self.render_artwork_modal(ctx);

        // Old modal window (kept for reference)
        if self.show_modal {
            egui::Window::new("Modal Dialog")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("This is a modal dialog!");
                    ui.add_space(10.0);

                    if ui.button("Close").clicked() {
                        self.show_modal = false;
                    }
                });
        }
    }
}
