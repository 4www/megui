use eframe::egui;
use egui_commonmark::CommonMarkCache;
use std::sync::mpsc::Receiver;

use crate::artwork::{Artwork, ArtworksResponse};
use crate::components::{sidebar::Sidebar, settings::SettingsModal, ThemeMode};
use crate::config::Config;
use crate::pages::{HomePage, AboutPage, ArtworksPage, ResumePage};
use crate::routes::Route;

pub struct MeguiApp {
    config: Config,
    current_route: Route,
    theme_mode: ThemeMode,

    // Artworks state
    pub artworks: Vec<Artwork>,
    pub loading: bool,
    pub error: Option<String>,
    pub selected_artworks: Vec<Artwork>,
    pub fetch_receiver: Option<Receiver<ehttp::Result<ehttp::Response>>>,

    // Resume state
    resume_content: Option<String>,
    resume_loading: bool,
    resume_receiver: Option<Receiver<ehttp::Result<ehttp::Response>>>,
    markdown_cache: CommonMarkCache,

    // UI state
    sidebar_open: bool,
    settings_open: bool,
}

impl Default for MeguiApp {
    fn default() -> Self {
        let config = Config::default();
        let theme_mode = ThemeMode::from_str(&config.app.default_theme);

        // Check for initial route from URL hash
        let initial_route = Route::get_from_url();

        let mut app = Self {
            config,
            current_route: initial_route,
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

impl MeguiApp {
    fn start_artworks_fetch(&mut self) {
        self.loading = true;
        self.error = None;
        let (sender, receiver) = std::sync::mpsc::channel();
        self.fetch_receiver = Some(receiver);

        let artworks_url = self.config.app.artworks.clone();
        ehttp::fetch(ehttp::Request::get(&artworks_url), move |result| {
            let _ = sender.send(result);
        });
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
                            // Strip script and style tags before converting to markdown
                            let cleaned_html = Self::strip_script_and_style_tags(html);
                            // Convert HTML to Markdown
                            let markdown = html2md::parse_html(&cleaned_html);
                            self.resume_content = Some(markdown);
                            self.current_route = Route::Resume;
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

    fn strip_script_and_style_tags(html: &str) -> String {
        // Use ammonia to sanitize HTML
        // By default, ammonia removes script and style tags along with their content
        // and uses a whitelist-based approach to only keep safe HTML tags
        ammonia::clean(html)
    }

    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(200.0)
            .show_animated(ctx, self.sidebar_open, |ui| {
                Sidebar::render(
                    ui,
                    ctx,
                    &self.config,
                    &mut self.current_route,
                    self.artworks.len(),
                    &self.resume_content,
                    &mut self.resume_loading,
                    &mut self.resume_receiver,
                    &mut self.settings_open,
                );
            });
    }
}

impl eframe::App for MeguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme with font sizes
        self.theme_mode.apply_with_style(ctx);

        // Check for fetch responses
        self.process_fetch_response();
        self.process_resume_response();

        if self.fetch_receiver.is_some() || self.resume_receiver.is_some() {
            ctx.request_repaint();
        }

        // Top bar with menu toggle
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if self.sidebar_open {
                        "☰ Hide Menu"
                    } else {
                        "☰ Show Menu"
                    })
                    .clicked()
                {
                    self.sidebar_open = !self.sidebar_open;
                }
                ui.separator();

                // Make site title clickable to go to homepage
                if ui
                    .add(egui::Label::new(
                        egui::RichText::new(&self.config.app.name)
                            .heading()
                            .color(ui.visuals().hyperlink_color)
                    ).sense(egui::Sense::click()))
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.current_route = Route::Home;
                    self.current_route.update_browser_url();
                }

                ui.separator();
                ui.label(self.current_route.title());
            });
        });

        // Render sidebar
        self.render_sidebar(ctx);

        // Render main content based on current route
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_route {
                Route::Home => HomePage::render(ui, &self.config),
                Route::Artworks => ArtworksPage::render(
                    ui,
                    ctx,
                    &self.config,
                    &self.artworks,
                    &mut self.selected_artworks,
                    self.loading,
                    &self.error,
                ),
                Route::Resume => ResumePage::render(
                    ui,
                    ctx,
                    &self.config,
                    &self.resume_content,
                    &mut self.markdown_cache,
                ),
                Route::About => AboutPage::render(ui, &self.config),
            }
        });

        // Artwork detail modals (can have multiple open at once)
        ArtworksPage::render_artwork_modals(ctx, &self.config, &mut self.selected_artworks);

        // Settings modal
        SettingsModal::render(ctx, &self.config, &mut self.theme_mode, &mut self.settings_open);

        // Sync hframe (required for iframe rendering on web)
        #[cfg(target_arch = "wasm32")]
        hframe::sync(ctx);
    }
}
