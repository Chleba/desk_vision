use super::Component;
use crate::{
    app_state::AppState,
    enums::{BroadcastMsg, DirectoryImages},
};
use egui::{CollapsingHeader, Color32, ScrollArea, Sense, Vec2};
use std::{
    path::{self, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

pub struct MainPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    app_state: Option<Arc<Mutex<AppState>>>,
    dir_images: Vec<DirectoryImages>,
}

impl MainPanel {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            app_state: None,
            dir_images: vec![],
        }
    }

    fn save_thumbnails(&mut self, dir_images: DirectoryImages) {
        if self
            .dir_images
            .iter()
            .all(|item| item.dir != dir_images.dir)
        {
            self.dir_images.push(dir_images);
        }
    }

    fn remove_thumbnails(&mut self, path: PathBuf) {
        self.dir_images.retain(|p| p.dir != path);
    }

    fn render_dir_images(&mut self, dir: DirectoryImages, ui: &mut egui::Ui) {
        let path_title = format!("{} ({})", dir.dir.to_string_lossy(), dir.images.len());
        CollapsingHeader::new(path_title).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for image in dir.images {
                    let s_text =
                        egui::load::SizedTexture::new(image.texture.id(), egui::vec2(160.0, 160.0));

                    let resp = ui.add(
                        egui::Image::from_texture(s_text)
                            .fit_to_exact_size(Vec2::new(120.0, 120.0))
                            .bg_fill(Color32::from_rgb(33, 33, 33))
                            // .max_width(440.0)
                            .sense(Sense::click())
                            .rounding(6.0),
                    );
                    if resp.clicked() {
                        println!("open files {}", image.file);
                        let _ = open::that(image.file);
                    }
                }
            });
        });
    }
}

impl Component for MainPanel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }

    fn update(&mut self, msg: BroadcastMsg) {
        match msg {
            BroadcastMsg::RemovedDirectory(path) => {
                self.remove_thumbnails(path);
            }
            BroadcastMsg::DirectoryImages(dir_file) => {
                self.save_thumbnails(dir_file);
            }
            _ => {}
        }
    }

    fn register_app_state(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = Some(app_state);
    }

    fn render(&mut self, ctx: &egui::Context) {
        // ctx.request_repaint_after_secs(1.0);

        let images = self.dir_images.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                egui::Frame::default().show(ui, |ui| {
                    ScrollArea::vertical()
                        .animated(false)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for dir in images.iter() {
                                self.render_dir_images(dir.clone(), ui);
                            }
                        });
                });
            });
        });
    }
}
