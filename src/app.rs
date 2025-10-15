use eframe::egui;
use std::sync::mpsc::Receiver;

use crate::artwork::{Artwork, ArtworksResponse};
use egui_commonmark::CommonMarkCache;

#[cfg(target_arch = "wasm32")]
use hframe::HtmlWindow;

#[derive(Debug, Clone, Copy, PartialEq)]
enum View {
    Artworks,
    Resume,
    About,
}

pub struct MeguiApp {
    current_view: View,
    pub artworks: Vec<Artwork>,
    pub loading: bool,
    pub error: Option<String>,
    pub selected_artworks: Vec<Artwork>,
    pub fetch_receiver: Option<Receiver<ehttp::Result<ehttp::Response>>>,
    sidebar_open: bool,
    resume_content: Option<String>,
    resume_loading: bool,
    resume_receiver: Option<Receiver<ehttp::Result<ehttp::Response>>>,
    markdown_cache: CommonMarkCache,
}

impl Default for MeguiApp {
    fn default() -> Self {
        Self {
            current_view: View::Artworks,
            artworks: Vec::new(),
            loading: false,
            error: None,
            selected_artworks: Vec::new(),
            fetch_receiver: None,
            sidebar_open: true,
            resume_content: None,
            resume_loading: false,
            resume_receiver: None,
            markdown_cache: CommonMarkCache::default(),
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

    fn process_resume_response(&mut self) {
        if let Some(receiver) = &self.resume_receiver {
            if let Ok(result) = receiver.try_recv() {
                self.resume_loading = false;
                self.resume_receiver = None;

                match result {
                    Ok(response) => {
                        if let Some(html) = response.text() {
                            // Convert HTML to Markdown
                            let markdown = html2md::parse_html(html);
                            self.resume_content = Some(markdown);
                            self.current_view = View::Resume;
                        } else {
                            self.error = Some("Failed to read resume".to_string());
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Resume fetch error: {}", e));
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
                            // Add to selected artworks if not already open
                            if !self.selected_artworks.iter().any(|a| a.name == artwork.name) {
                                self.selected_artworks.push(artwork.clone());
                            }
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

    fn render_artwork_modals(&mut self, ctx: &egui::Context) {
        let mut to_remove = Vec::new();

        for (idx, artwork) in self.selected_artworks.iter().enumerate() {
            let mut open = true;
            let artwork_url = format!("https://artworks.hwww.org/{}/", artwork.name);

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

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);
                        ui.label("Note: iframe preview only available in web version.");
                        ui.label("Click 'Open in New Tab' to view the artwork.");
                    }
                });

            if !open {
                to_remove.push(idx);
            }

            // Render the iframe as an HtmlWindow behind the modal (web only)
            #[cfg(target_arch = "wasm32")]
            {
                let iframe_content = format!(
                    r#"<iframe src="{}" style="width: 100%; height: 100%; border: none;"></iframe>"#,
                    artwork_url
                );
                HtmlWindow::new(&format!("iframe_{}", artwork.name))
                    .content(&iframe_content)
                    .show(ctx);
            }
        }

        // Remove closed artworks in reverse order to maintain correct indices
        for idx in to_remove.iter().rev() {
            self.selected_artworks.remove(*idx);
        }
    }

    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(200.0)
            .show_animated(ctx, self.sidebar_open, |ui| {
                ui.heading("Menu");
                ui.separator();
                ui.add_space(10.0);

                // Artworks view
                let artworks_selected = self.current_view == View::Artworks;
                if ui.selectable_label(artworks_selected, "Artworks").clicked() {
                    self.current_view = View::Artworks;
                }

                ui.add_space(5.0);

                // Resume view
                let resume_selected = self.current_view == View::Resume;
                ui.horizontal(|ui| {
                    if ui.selectable_label(resume_selected, "Resume").clicked() {
                        if self.resume_content.is_none() && !self.resume_loading {
                            // Fetch resume if not already loaded
                            self.resume_loading = true;
                            let ctx = ctx.clone();
                            let (sender, receiver) = std::sync::mpsc::channel();
                            self.resume_receiver = Some(receiver);

                            ehttp::fetch(
                                ehttp::Request::get("https://resume.hwww.org"),
                                move |result| {
                                    let _ = sender.send(result);
                                    ctx.request_repaint();
                                },
                            );
                        } else if self.resume_content.is_some() {
                            self.current_view = View::Resume;
                        }
                    }

                    if self.resume_loading {
                        ui.spinner();
                    }
                });

                ui.add_space(5.0);

                // About view
                let about_selected = self.current_view == View::About;
                if ui.selectable_label(about_selected, "About").clicked() {
                    self.current_view = View::About;
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label(format!("Artworks loaded: {}", self.artworks.len()));

                ui.add_space(10.0);

                // External links section
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.hyperlink_to("📄 Resume", "https://resume.hwww.org");
                    ui.add_space(3.0);
                    ui.hyperlink_to("🎨 Artworks", "https://artworks.hwww.org");
                    ui.add_space(3.0);
                    ui.hyperlink_to("🌐 hwww.org", "https://hwww.org");

                    ui.add_space(5.0);
                    ui.label("External Links:");
                    ui.add_space(3.0);
                    ui.separator();
                    ui.add_space(10.0);
                });
            });
    }

    fn render_artworks_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.render_fetch_button(ui, ctx);

        // Error display
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        self.render_artworks_list(ui);
    }

    fn render_resume_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Resume");
        ui.add_space(10.0);

        if ui.button("🔗 Open in New Tab").clicked() {
            ctx.open_url(egui::OpenUrl::new_tab("https://resume.hwww.org"));
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(markdown) = &self.resume_content {
                // Render markdown with proper formatting
                egui_commonmark::CommonMarkViewer::new()
                    .show(ui, &mut self.markdown_cache, markdown);
            } else {
                ui.label("Loading resume...");
            }
        });
    }

    fn render_about_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("About");
        ui.add_space(10.0);

        ui.label("megui - A simple artworks viewer and portfolio");
        ui.add_space(10.0);

        ui.label("Built with:");
        ui.label("• Rust");
        ui.label("• egui - Immediate mode GUI framework");
        ui.label("• eframe - Web and native support");
        ui.add_space(10.0);

        ui.separator();
        ui.add_space(10.0);

        ui.label("This app fetches and displays artworks from artworks.hwww.org");
        ui.label("and renders the resume from resume.hwww.org");
    }
}

impl eframe::App for MeguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for fetch responses
        self.process_fetch_response();
        self.process_resume_response();

        if self.fetch_receiver.is_some() || self.resume_receiver.is_some() {
            ctx.request_repaint();
        }

        // Top bar with menu toggle
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(if self.sidebar_open { "☰ Hide Menu" } else { "☰ Show Menu" }).clicked() {
                    self.sidebar_open = !self.sidebar_open;
                }
                ui.separator();
                let title = match self.current_view {
                    View::Artworks => "Artworks",
                    View::Resume => "Resume",
                    View::About => "About",
                };
                ui.heading(title);
            });
        });

        // Render sidebar
        self.render_sidebar(ctx);

        // Render main content based on current view
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::Artworks => self.render_artworks_view(ui, ctx),
                View::Resume => self.render_resume_view(ui, ctx),
                View::About => self.render_about_view(ui),
            }
        });

        // Artwork detail modals (can have multiple open at once)
        self.render_artwork_modals(ctx);

        // Sync hframe (required for iframe rendering on web)
        #[cfg(target_arch = "wasm32")]
        hframe::sync(ctx);
    }
}
