mod app;
mod artwork;
mod config;
mod routes;
mod components;
mod pages;

use app::MeguiApp;

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
        use eframe::wasm_bindgen::JsCast;

        // Redirect tracing to console.log and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");

            let canvas = document
                .get_element_by_id("the_canvas_id")
                .expect("Failed to find the_canvas_id")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("the_canvas_id was not a HtmlCanvasElement");

            eframe::WebRunner::new()
                .start(
                    canvas,
                    web_options,
                    Box::new(|_cc| Ok(Box::new(MeguiApp::default()))),
                )
                .await
                .expect("failed to start eframe");
        });

        Ok(())
    }
}
