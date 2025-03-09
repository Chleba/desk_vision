#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[tokio::main]
async fn main() {
    // Log to stderr (if you run with `RUST_LOG=debug`).
    env_logger::init();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Unable to create tokio runtime");
    // -- tokio spawning instantly
    let _rt_enter = rt.enter();

    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(36000)).await;
            }
        });
    });

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "AI Image Seach",
        native_options,
        Box::new(|cc| {
            // -- image loader
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // -- material icons
            egui_material_icons::initialize(&cc.egui_ctx);

            // -- app
            let mut desk_app = desk_imager::DeskApp::new(cc);
            desk_app.init(cc);

            Ok(Box::new(desk_app))
        }),
    )
    .expect("failed to run app");
}
