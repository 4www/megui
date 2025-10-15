/// Application routes/views
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Route {
    Home,
    Artworks,
    Resume,
    About,
}

impl Route {
    #[cfg(target_arch = "wasm32")]
    pub fn to_hash(&self) -> &'static str {
        match self {
            Route::Home => "#/home",
            Route::Artworks => "#/artworks",
            Route::Resume => "#/resume",
            Route::About => "#/about",
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_hash(hash: &str) -> Option<Self> {
        match hash {
            "#/home" | "#home" | "#/" | "#" | "" => Some(Route::Home),
            "#/artworks" | "#artworks" => Some(Route::Artworks),
            "#/resume" | "#resume" => Some(Route::Resume),
            "#/about" | "#about" => Some(Route::About),
            _ => None,
        }
    }

    pub fn get_from_url() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(location) = window.location().hash() {
                    if let Some(route) = Self::from_hash(&location) {
                        return route;
                    }
                }
            }
        }
        Route::Home
    }

    pub fn title(&self) -> &'static str {
        match self {
            Route::Home => "Home",
            Route::Artworks => "Artworks",
            Route::Resume => "Resume",
            Route::About => "About",
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update_browser_url(&self) {
        if let Some(window) = web_sys::window() {
            if let Some(history) = window.history().ok() {
                let hash = self.to_hash();
                let _ = history.replace_state_with_url(
                    &wasm_bindgen::JsValue::NULL,
                    "",
                    Some(hash),
                );
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn update_browser_url(&self) {
        // No-op on native
    }
}
