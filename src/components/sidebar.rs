use eframe::egui;
use std::sync::mpsc;

use crate::config::Config;
use crate::routes::Route;

pub struct Sidebar;

impl Sidebar {
    pub fn render(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        config: &Config,
        current_route: &mut Route,
        artworks_count: usize,
        resume_content: &Option<String>,
        resume_loading: &mut bool,
        resume_receiver: &mut Option<mpsc::Receiver<ehttp::Result<ehttp::Response>>>,
        settings_open: &mut bool,
    ) {
        ui.heading("Menu");
        ui.separator();
        ui.add_space(10.0);

        // Artworks route
        let artworks_selected = *current_route == Route::Artworks;
        if ui.selectable_label(artworks_selected, "Artworks").clicked() {
            *current_route = Route::Artworks;
            current_route.update_browser_url();
        }

        ui.add_space(5.0);

        // Resume route
        let resume_selected = *current_route == Route::Resume;
        ui.horizontal(|ui| {
            if ui.selectable_label(resume_selected, "Resume").clicked() {
                if resume_content.is_none() && !*resume_loading {
                    // Fetch resume if not already loaded
                    *resume_loading = true;
                    let ctx = ctx.clone();
                    let (sender, receiver) = mpsc::channel();
                    *resume_receiver = Some(receiver);

                    let resume_url = config.app.resume.clone();
                    ehttp::fetch(
                        ehttp::Request::get(&resume_url),
                        move |result| {
                            let _ = sender.send(result);
                            ctx.request_repaint();
                        },
                    );
                } else if resume_content.is_some() {
                    *current_route = Route::Resume;
                }
                current_route.update_browser_url();
            }

            if *resume_loading {
                ui.spinner();
            }
        });

        ui.add_space(5.0);

        // About route
        let about_selected = *current_route == Route::About;
        if ui.selectable_label(about_selected, "About").clicked() {
            *current_route = Route::About;
            current_route.update_browser_url();
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label(format!("Artworks loaded: {}", artworks_count));

        ui.add_space(10.0);

        // Settings button and footer
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(10.0);
            ui.label(format!("{} ©", config.app.name));
            ui.add_space(10.0);

            if ui.button("⚙ Settings").clicked() {
                *settings_open = true;
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
        });
    }
}
