use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "megui",
        options,
        Box::new(|_cc| Ok(Box::new(MeguiApp::default()))),
    )
}

struct MeguiApp;

impl Default for MeguiApp {
    fn default() -> Self {
        Self
    }
}

impl eframe::App for MeguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World");
        });
    }
}
