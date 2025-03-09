use super::Component;
use crate::{
    app_state::AppState,
    enums::{BroadcastMsg, DirectoryImage, DirectoryImages},
};
use egui::{CollapsingHeader, Color32, ScrollArea, Sense, Vec2};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

pub struct MainPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    app_state: Option<Arc<Mutex<AppState>>>,
    dir_images: Vec<DirectoryImages>,
    found_images: Vec<DirectoryImage>,
    search_inputs: HashMap<String, String>,
}

impl MainPanel {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            app_state: None,
            dir_images: vec![],
            found_images: vec![],
            search_inputs: HashMap::new(),
        }
    }

    fn save_thumbnails(&mut self, dir_images: DirectoryImages) {
        if self
            .dir_images
            .iter()
            .all(|item| item.dir != dir_images.dir)
        {
            self.dir_images.push(dir_images.clone());

            // -- save to search hashmap
            self.search_inputs
                .insert(dir_images.dir.to_string_lossy().to_string(), "".to_string());
        }
    }

    fn remove_thumbnails(&mut self, path: PathBuf) {
        self.dir_images.retain(|p| p.dir != path);
    }

    fn add_labels_to_file(&mut self, file: String, labels: String) {
        let l_labels: Vec<String> = labels
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        for dir in self.dir_images.iter_mut() {
            if let Some(f_file) = dir.images.iter_mut().find(|f| f.file == file) {
                f_file.labels = l_labels.clone();
            }
        }
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

        let mut imgs = vec![];

        for dir in self.dir_images.iter() {
            for img in dir.images.iter() {
                let labels = img.clone().labels;

                let common: Vec<_> = labels
                    .iter()
                    .map(|s| s.to_lowercase())
                    .filter(|f| l_labels.iter().any(|s| f.contains(s)))
                    .collect();

                if !common.is_empty() {
                    imgs.push(img.clone());
                }
            }
        }

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
            let dir_string = dir.dir.to_string_lossy().to_string();
            let resp = ui.add(
                egui::TextEdit::singleline(self.search_inputs.get_mut(&dir_string).unwrap())
                    .hint_text("Search here.."),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(action_tx) = self.action_tx.clone() {
                    // let _ = action_tx.send(BroadcastMsg::SearchByLabels(self.input_text.clone()));
                }
            }
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
            BroadcastMsg::GetLabelsForImage(file, labels) => {
                self.add_labels_to_file(file, labels);
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
