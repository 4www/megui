use eframe::egui;
use serde::Deserialize;
use std::sync::mpsc::{channel, Receiver};

fn main() -> Result<(), eframe::Error> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
            ..Default::default()
        };

        eframe::run_native(
            "megui",
            options,
            Box::new(|_cc| Ok(Box::new(MeguiApp::default()))),
        )
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Redirect tracing to console.log and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    "the_canvas_id",
                    web_options,
                    Box::new(|_cc| Ok(Box::new(MeguiApp::default()))),
                )
                .await
                .expect("failed to start eframe");
        });

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Artwork {
    name: String,
    #[serde(default)]
    info: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ArtworksResponse {
    #[serde(rename = "type")]
    response_type: String,
    name: String,
    contents: Vec<Artwork>,
}

struct MeguiApp {
    show_modal: bool,
    artworks: Vec<Artwork>,
    loading: bool,
    error: Option<String>,
    selected_artwork: Option<Artwork>,
    fetch_receiver: Option<Receiver<ehttp::Result<ehttp::Response>>>,
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

impl eframe::App for MeguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for fetch response
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
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Artworks Gallery");
            ui.add_space(10.0);

            // Fetch button
            ui.horizontal(|ui| {
                if ui.button("Fetch Artworks").clicked() && !self.loading {
                    self.loading = true;
                    self.error = None;
                    let ctx = ctx.clone();
                    let (sender, receiver) = channel();
                    self.fetch_receiver = Some(receiver);

                    ehttp::fetch(
                        ehttp::Request::get("https://artworks.hwww.org/index.json"),
                        move |result| {
                            let _ = sender.send(result);
                            ctx.request_repaint();
                        }
                    );
                }

                if self.loading {
                    ui.spinner();
                }
            });

            // Error display
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Artworks list
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
        });

        // Artwork detail modal
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
