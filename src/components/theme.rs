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
            ThemeMode::Light => ctx.set_visuals(Self::custom_light()),
            ThemeMode::Dark => ctx.set_visuals(Self::custom_dark()),
            ThemeMode::Auto => {
                // Use system preference if available, otherwise default to dark
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(window) = web_sys::window() {
                        if let Ok(media_query) = window.match_media("(prefers-color-scheme: dark)") {
                            if let Some(mq) = media_query {
                                if mq.matches() {
                                    ctx.set_visuals(Self::custom_dark());
                                } else {
                                    ctx.set_visuals(Self::custom_light());
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
                    ctx.set_visuals(Self::custom_dark());
                }
            }
        }
    }

    fn custom_light() -> egui::Visuals {
        let mut visuals = egui::Visuals::light();

        // Only customize background colors - light blue theme
        visuals.panel_fill = egui::Color32::from_rgb(220, 230, 245);
        visuals.window_fill = egui::Color32::from_rgb(235, 242, 250);
        visuals.extreme_bg_color = egui::Color32::from_rgb(200, 215, 235);

        visuals
    }

    pub fn apply_with_style(&self, ctx: &egui::Context) {
        // Increase font sizes first
        let mut style = (*ctx.style()).clone();

        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(18.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(18.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::proportional(28.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::monospace(17.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::proportional(15.0),
        );

        // Improve vertical spacing for better rhythm
        style.spacing.item_spacing.y = 10.0; // More space between items
        style.spacing.window_margin = egui::Margin::same(12); // More padding
        style.spacing.button_padding = egui::vec2(10.0, 5.0); // Better button padding
        style.spacing.indent = 20.0; // Better indentation

        ctx.set_style(style);

        // Apply colors after style
        self.apply(ctx);
    }

    fn custom_dark() -> egui::Visuals {
        let mut visuals = egui::Visuals::dark();

        // Only customize background colors - dark blue theme
        visuals.panel_fill = egui::Color32::from_rgb(15, 20, 35);
        visuals.window_fill = egui::Color32::from_rgb(20, 25, 40);
        visuals.extreme_bg_color = egui::Color32::from_rgb(10, 15, 28);

        visuals
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Auto => "Auto",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }
}
