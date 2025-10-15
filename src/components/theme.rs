use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Auto,
    Dark,
    Light,
}

impl ThemeMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => ThemeMode::Dark,
            "light" => ThemeMode::Light,
            _ => ThemeMode::Auto,
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        match self {
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
                                return;
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
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Auto => "Auto",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }
}
