mod app;
mod data;
mod export;
mod updater;

use app::DesktopApp;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Desktop Application with Auto-Update",
        options,
        Box::new(|_cc| {
            let app = DesktopApp::new(_cc);
            Box::new(app)
        }),
    )
}
