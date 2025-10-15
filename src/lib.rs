mod app;
mod artwork;
mod config;

use app::MeguiApp;
use wasm_bindgen::prelude::*;
use eframe::wasm_bindgen::JsCast;

#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    match eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|_cc| Ok(Box::new(MeguiApp::default()))),
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
    }
}
