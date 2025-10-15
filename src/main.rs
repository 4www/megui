mod app;
mod artwork;

use app::MeguiApp;
use eframe::egui;

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
