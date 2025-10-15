use eframe::egui;
use std::sync::mpsc::Receiver;

use crate::artwork::{Artwork, ArtworksResponse};
use crate::config::Config;
use egui_commonmark::CommonMarkCache;

#[cfg(target_arch = "wasm32")]
use hframe::HtmlWindow;

#[derive(Debug, Clone, Copy, PartialEq)]
enum View {
    Artworks,
    Resume,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ThemeMode {
    Auto,
    Dark,
    Light,
}

impl ThemeMode {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => ThemeMode::Dark,
            "light" => ThemeMode::Light,
            _ => ThemeMode::Auto,
        }
    }
}

pub struct MeguiApp {
    config: Config,
    current_view: View,
    theme_mode: ThemeMode,
    pub artworks: Vec<Artwork>,
    pub loading: bool,
    pub error: Option<String>,
    pub selected_artworks: Vec<Artwork>,
    pub fetch_receiver: Option<Receiver<ehttp::Result<ehttp::Response>>>,
    sidebar_open: bool,
    settings_open: bool,
    resume_content: Option<String>,
    resume_loading: bool,
    resume_receiver: Option<Receiver<ehttp::Result<ehttp::Response>>>,
    markdown_cache: CommonMarkCache,
}

impl Default for MeguiApp {
    fn default() -> Self {
        let config = Config::default();
        let theme_mode = ThemeMode::from_str(&config.app.default_theme);

        // Check for initial route from URL hash
        let initial_view = Self::get_view_from_url();

        let mut app = Self {
            config,
            current_view: initial_view,
            theme_mode,
            artworks: Vec::new(),
            loading: false,
            error: None,
            selected_artworks: Vec::new(),
            fetch_receiver: None,
            sidebar_open: true,
            settings_open: false,
            resume_content: None,
            resume_loading: false,
            resume_receiver: None,
            markdown_cache: CommonMarkCache::default(),
        };

        // Auto-fetch artworks on startup
        app.start_artworks_fetch();

        app
    }
}

impl View {
    #[cfg(target_arch = "wasm32")]
    fn to_hash(&self) -> &'static str {
        match self {
            View::Artworks => "#/artworks",
            View::Resume => "#/resume",
            View::About => "#/about",
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn from_hash(hash: &str) -> Option<Self> {
        match hash {
            "#/artworks" | "#artworks" => Some(View::Artworks),
            "#/resume" | "#resume" => Some(View::Resume),
            "#/about" | "#about" => Some(View::About),
            _ => None,
        }
    }
}

impl MeguiApp {
    fn start_artworks_fetch(&mut self) {
        self.loading = true;
        self.error = None;
        let (sender, receiver) = std::sync::mpsc::channel();
        self.fetch_receiver = Some(receiver);

        let artworks_url = self.config.app.artworks.clone();
        ehttp::fetch(
            ehttp::Request::get(&artworks_url),
            move |result| {
                let _ = sender.send(result);
            },
        );
    }

    fn get_view_from_url() -> View {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(location) = window.location().hash() {
                    if let Some(view) = View::from_hash(&location) {
                        return view;
                    }
                }
            }
        }
        View::Artworks
    }

    fn update_url_hash(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let hash = self.current_view.to_hash();
                    let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(hash));
                }
            }
        }
    }

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
            if ui.button("🔄 Refresh Artworks").clicked() && !self.loading {
                self.loading = true;
                self.error = None;
                let ctx = ctx.clone();
                let (sender, receiver) = std::sync::mpsc::channel();
                self.fetch_receiver = Some(receiver);

                let artworks_url = self.config.app.artworks.clone();
                ehttp::fetch(
                    ehttp::Request::get(&artworks_url),
                    move |result| {
                        let _ = sender.send(result);
                        ctx.request_repaint();
                    },
                );
            }

            if self.loading {
                ui.spinner();
                ui.label("Loading...");
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
        } else if self.loading {
            ui.spinner();
            ui.label("Loading artworks...");
        }
    }

    fn render_artwork_modals(&mut self, ctx: &egui::Context) {
        let mut to_remove = Vec::new();

        #[cfg(target_arch = "wasm32")]
        for (idx, artwork) in self.selected_artworks.iter().enumerate() {
            let mut open = true;
            let artworks_base = self.config.app.artworks.trim_end_matches("/index.json");
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
        for (idx, artwork) in self.selected_artworks.iter().enumerate() {
            let mut open = true;
            let artworks_base = self.config.app.artworks.trim_end_matches("/index.json");
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
                    self.update_url_hash();
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

                            let resume_url = self.config.app.resume.clone();
                            ehttp::fetch(
                                ehttp::Request::get(&resume_url),
                                move |result| {
                                    let _ = sender.send(result);
                                    ctx.request_repaint();
                                },
                            );
                        } else if self.resume_content.is_some() {
                            self.current_view = View::Resume;
                        }
                        self.update_url_hash();
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
                    self.update_url_hash();
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label(format!("Artworks loaded: {}", self.artworks.len()));

                ui.add_space(10.0);

                // Settings button and footer
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(10.0);
                    ui.label(format!("{} ©", self.config.app.name));
                    ui.add_space(10.0);

                    if ui.button("⚙ Settings").clicked() {
                        self.settings_open = true;
                    }

                    ui.add_space(10.0);
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
            ctx.open_url(egui::OpenUrl::new_tab(&self.config.app.resume));
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

    fn render_settings_modal(&mut self, ctx: &egui::Context) {
        egui::Window::new("⚙ Settings")
            .open(&mut self.settings_open)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Theme");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.selectable_label(self.theme_mode == ThemeMode::Auto, "Auto").clicked() {
                        self.theme_mode = ThemeMode::Auto;
                    }
                    if ui.selectable_label(self.theme_mode == ThemeMode::Light, "Light").clicked() {
                        self.theme_mode = ThemeMode::Light;
                    }
                    if ui.selectable_label(self.theme_mode == ThemeMode::Dark, "Dark").clicked() {
                        self.theme_mode = ThemeMode::Dark;
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
                        ui.label(&self.config.app.name);
                        ui.end_row();

                        ui.label("Website:");
                        ui.hyperlink_to(&self.config.app.website, &self.config.app.website);
                        ui.end_row();

                        ui.label("Resume:");
                        ui.hyperlink_to(&self.config.app.resume, &self.config.app.resume);
                        ui.end_row();

                        ui.label("Artworks:");
                        let artworks_base = self.config.app.artworks.trim_end_matches("/index.json");
                        ui.hyperlink_to(artworks_base, artworks_base);
                        ui.end_row();

                        ui.label("Repository:");
                        ui.hyperlink_to(&self.config.app.repository, &self.config.app.repository);
                        ui.end_row();

                        ui.label("Current Theme:");
                        let theme_str = match self.theme_mode {
                            ThemeMode::Auto => "Auto",
                            ThemeMode::Light => "Light",
                            ThemeMode::Dark => "Dark",
                        };
                        ui.label(theme_str);
                        ui.end_row();
                    });

                ui.add_space(15.0);
            });
    }

    fn render_about_view(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("About");
        ui.add_space(10.0);

        ui.label(format!("{} - A simple artworks viewer and portfolio", self.config.app.name));
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
            ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
            ui.label("- Web and native support");
        });
        ui.add_space(10.0);
    }
}

impl eframe::App for MeguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        match self.theme_mode {
            ThemeMode::Light => ctx.set_visuals(egui::Visuals::light()),
            ThemeMode::Dark => ctx.set_visuals(egui::Visuals::dark()),
            ThemeMode::Auto => {
                // Use system preference if available, otherwise default to dark
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(window) = web_sys::window() {
                        if let Ok(media_query) = window.match_media("(prefers-color-scheme: dark)") {
                            if let Some(mq) = media_query {
                                if mq.matches() {
                                    ctx.set_visuals(egui::Visuals::dark());
                                } else {
                                    ctx.set_visuals(egui::Visuals::light());
                                }
                            }
                        }
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // On native, we could use a crate like `dark-light` to detect system theme
                    // For now, default to dark
                    ctx.set_visuals(egui::Visuals::dark());
                }
            }
        }

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
                ui.heading(&self.config.app.name);
                ui.separator();
                let title = match self.current_view {
                    View::Artworks => "Artworks",
                    View::Resume => "Resume",
                    View::About => "About",
                };
                ui.label(title);
            });
        });

        // Render sidebar
        self.render_sidebar(ctx);

        // Render main content based on current view
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::Artworks => self.render_artworks_view(ui, ctx),
                View::Resume => self.render_resume_view(ui, ctx),
                View::About => self.render_about_view(ui, ctx),
            }
        });

        // Artwork detail modals (can have multiple open at once)
        self.render_artwork_modals(ctx);

        // Settings modal
        self.render_settings_modal(ctx);

        // Sync hframe (required for iframe rendering on web)
        #[cfg(target_arch = "wasm32")]
        hframe::sync(ctx);
    }
}
