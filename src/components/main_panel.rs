use super::Component;
use crate::{
    app_state::AppState,
    enums::{BroadcastMsg, DirectoryImage, DirectoryImages},
};
use egui::{CollapsingHeader, Color32, ScrollArea, Sense, Vec2};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

pub struct MainPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    app_state: Option<Arc<Mutex<AppState>>>,
    dir_images: Vec<DirectoryImages>,
    found_images: Vec<DirectoryImage>,
}

impl MainPanel {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            app_state: None,
            dir_images: vec![],
            found_images: vec![],
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

    fn search_by_labels(&mut self, labels: String) {
        println!("SERACH BY LABELS: {:?}", labels);

        let l_labels: Vec<String> = labels
            .split(',')
            .map(|s| s.trim())
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        println!("{:?} - search labels", l_labels);

        let mut imgs = vec![];

        for dir in self.dir_images.iter() {
            for img in dir.images.iter() {
                let labels = img.clone().labels;

                let common: Vec<_> = labels
                    .iter()
                    .map(|s| s.to_lowercase())
                    .filter(|f| l_labels.iter().any(|s| f.contains(s)))
                    .collect();

                println!("{:?} same labels", common);

                if !common.is_empty() {
                    imgs.push(img.clone());
                }

                // if l_labels.iter().any(|s| labels.contains(s)) {
                //     imgs.push(img.clone());
                // }
            }
        }

        println!("{:?}", imgs.len());

        self.found_images = imgs;
    }

    fn render_found_images(&mut self, ui: &mut egui::Ui) {
        let title = format!("Found Images: ({})", self.found_images.len());
        CollapsingHeader::new(title)
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for image in self.found_images.clone() {
                        let s_text = egui::load::SizedTexture::new(
                            image.texture.id(),
                            egui::vec2(160.0, 160.0),
                        );

                        println!("{} - labels", image.labels.join(","));

                        let resp = ui
                            .add(
                                egui::Image::from_texture(s_text)
                                    .fit_to_exact_size(Vec2::new(120.0, 120.0))
                                    .bg_fill(Color32::from_rgb(33, 33, 33))
                                    // .max_width(440.0)
                                    .sense(Sense::click())
                                    // .sense(Sense::hover())
                                    .rounding(6.0),
                            )
                            .on_hover_text(image.labels.join(","));
                        // resp.on_hover_text(image.labels.join(","));
                        if resp.clicked() {
                            println!("open files {}", image.file);
                            let _ = open::that(image.file);
                        }
                    }
                });
            });

        ui.separator();
    }

    fn render_dir_images(&mut self, dir: DirectoryImages, ui: &mut egui::Ui) {
        let path_title = format!("{} ({})", dir.dir.to_string_lossy(), dir.images.len());
        CollapsingHeader::new(path_title).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for image in dir.images {
                    let s_text =
                        egui::load::SizedTexture::new(image.texture.id(), egui::vec2(160.0, 160.0));

                    let resp = ui
                        .add(
                            egui::Image::from_texture(s_text)
                                .fit_to_exact_size(Vec2::new(120.0, 120.0))
                                .bg_fill(Color32::from_rgb(33, 33, 33))
                                .sense(Sense::click())
                                // .sense(Sense::hover())
                                .rounding(6.0),
                        )
                        .on_hover_text(image.labels.join(","));
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
            BroadcastMsg::SearchByLabels(labels) => {
                self.search_by_labels(labels);
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
                            self.render_found_images(ui);

                            for dir in images.iter() {
                                self.render_dir_images(dir.clone(), ui);
                            }
                        });
                });
            });
        });
    }
}
