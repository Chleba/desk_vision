use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::ollama_settings::OllamaSettings;
use super::Component;
use crate::{app_state::AppState, enums::BroadcastMsg};
use egui::{Align, CollapsingHeader, Color32, Grid, RichText, ScrollArea};
use tokio::sync::mpsc::UnboundedSender;

pub struct TopPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    input_text: String,
    picked_directories: Vec<PathBuf>,
    app_state: Option<Arc<Mutex<AppState>>>,
    ollama_button: OllamaSettings,
    ollama_connected: bool,
    non_labeled_imgs: usize,
    all_imgs_num: usize,
    is_labeling: bool,
}

impl TopPanel {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            input_text: "".to_string(),
            picked_directories: vec![],
            app_state: None,
            ollama_connected: false,
            ollama_button: OllamaSettings::new(),
            non_labeled_imgs: 0,
            all_imgs_num: 0,
            is_labeling: false,
        }
    }

    fn get_labeled_images(&mut self) {
        let mut dir_files = vec![];
        {
            if let Some(ref app_state) = self.app_state {
                dir_files = app_state.lock().unwrap().dir_files.clone();
            }
        }

        // -- get all files that is not having labels
        let mut non_labeled_imgs = 0;
        let mut all_imgs_num = 0;
        for dir in dir_files.iter() {
            all_imgs_num += dir.files_with_labels.len();
            for file in dir.files_with_labels.iter() {
                if file.labels.is_empty() {
                    non_labeled_imgs += 1;
                }
            }
        }
        self.non_labeled_imgs = non_labeled_imgs;
        self.all_imgs_num = all_imgs_num;
    }

    fn pick_dir(&mut self, path: PathBuf) {
        if !self.picked_directories.contains(&path) {
            self.picked_directories.push(path.clone());
            {
                if let Some(ref app_state) = self.app_state {
                    app_state.lock().unwrap().directories = self.picked_directories.clone();
                }
            }

            if let Some(action_tx) = self.action_tx.clone() {
                let _ = action_tx.send(BroadcastMsg::PickedDirectory(path));
            }
        }
    }

    fn remove_directory(&mut self, path: PathBuf) {
        if self.picked_directories.contains(&path) {
            self.picked_directories.retain(|p| *p != path);
            {
                if let Some(ref app_state) = self.app_state {
                    app_state.lock().unwrap().remove_directory(path.clone());
                }

                if let Some(action_tx) = self.action_tx.clone() {
                    let _ = action_tx.send(BroadcastMsg::RemovedDirectory(path));
                }
            }
        }
    }

    fn draw_left_side(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            egui::Grid::new("left_grid").num_columns(2).show(ui, |ui| {
                ui.label("Search images:");
                // -- search input
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.input_text).hint_text("Search here.."),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    println!("PRESS SAERCH PICO");
                    if let Some(action_tx) = self.action_tx.clone() {
                        let _ =
                            action_tx.send(BroadcastMsg::SearchByLabels(self.input_text.clone()));
                    }
                }

                ui.end_row();

                // -- directory picker
                ui.label("Add new folder:");
                if ui.button("Pick a directory").clicked() {
                    if let Some(dir_path) = rfd::FileDialog::new().pick_folder() {
                        self.pick_dir(dir_path);
                    }
                }
            });

            CollapsingHeader::new("Added directories:").show(ui, |ui| {
                ScrollArea::vertical().max_width(420.0).show(ui, |ui| {
                    Grid::new("dir_grid")
                        .striped(true)
                        .num_columns(2)
                        .min_col_width(50.0)
                        .max_col_width(340.0)
                        .show(ui, |ui| {
                            for dir in self.picked_directories.clone().iter() {
                                ui.small(dir.to_string_lossy());
                                if ui.button("delete").clicked() {
                                    self.remove_directory(dir.clone());
                                    println!("delete: {}", dir.to_string_lossy());
                                }
                                ui.end_row();
                            }
                        });
                });
            });

            // ui.label("");
            // ScrollArea::vertical().max_width(420.0).show(ui, |ui| {
            //     Grid::new("dir_grid")
            //         .striped(true)
            //         .num_columns(2)
            //         .min_col_width(50.0)
            //         .max_col_width(340.0)
            //         .show(ui, |ui| {
            //             for dir in self.picked_directories.clone().iter() {
            //                 ui.small(dir.to_string_lossy());
            //                 if ui.button("delete").clicked() {
            //                     self.remove_directory(dir.clone());
            //                     println!("delete: {}", dir.to_string_lossy());
            //                 }
            //                 ui.end_row();
            //             }
            //         });
            // });
        });
    }

    fn draw_right_side(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // -- ollama menu button
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                self.ollama_button.ui(ui);
                if !self.ollama_connected {
                    ui.small(RichText::new("not connected").color(Color32::from_rgb(255, 0, 0)));
                } else {
                    ui.small(RichText::new("connected").color(Color32::from_rgb(0, 255, 0)));
                }
            });
        });

        let labeled_imgs = self.all_imgs_num - self.non_labeled_imgs;
        ui.horizontal(|ui| {
            if self.all_imgs_num == labeled_imgs {
                ui.label(RichText::new("all done").color(Color32::from_rgb(0, 255, 255)));
            } else {
                if self.is_labeling {
                    ui.spinner();
                    if ui.button("stop").clicked() {
                        if let Some(action_tx) = self.action_tx.clone() {
                            let _ = action_tx.send(BroadcastMsg::StopLabeling);
                        }
                    }
                } else {
                    if ui.button("start").clicked() {
                        if let Some(action_tx) = self.action_tx.clone() {
                            let _ = action_tx.send(BroadcastMsg::StartLabeling);
                        }
                    }
                }
                ui.label(format!("{}/{}", labeled_imgs, self.all_imgs_num));
            }
            ui.label("labels:");
        });
    }
}

impl Component for TopPanel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn init(&mut self, cc: &eframe::CreationContext<'_>) {
        self.ollama_button.init(cc);
        self.get_labeled_images();
    }

    fn update(&mut self, msg: BroadcastMsg) {
        self.ollama_button.update(msg.clone());

        match msg {
            BroadcastMsg::OllamaRunning(r) => self.ollama_connected = r.is_ok(),
            BroadcastMsg::StartLabeling => {
                self.is_labeling = true;
            }
            BroadcastMsg::StopLabeling => {
                self.is_labeling = false;
            }
            BroadcastMsg::DirectoryImages(_) => {
                self.get_labeled_images();
            }
            BroadcastMsg::RemovedDirectory(_) => {
                self.get_labeled_images();
            }
            BroadcastMsg::GetLabelsForImage(_, _) => {
                self.get_labeled_images();
            }
            _ => {}
        }
    }

    fn render(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .min_height(100.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Min), |ui| {
                        self.draw_left_side(ui);
                    });
                    ui.with_layout(
                        egui::Layout::from_main_dir_and_cross_align(
                            egui::Direction::TopDown,
                            Align::Max,
                        ),
                        |ui| {
                            self.draw_right_side(ui);
                        },
                    );
                });
            });
    }

    fn register_app_state(&mut self, app_state: std::sync::Arc<std::sync::Mutex<AppState>>) {
        self.app_state = Some(app_state);

        // -- directories
        {
            if let Some(app_state) = self.app_state.clone() {
                let directories = app_state.lock().unwrap().directories.clone();
                self.picked_directories = directories;
            }
        }
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.ollama_button.register_tx(action_tx.clone());
        self.action_tx = Some(action_tx);
    }
}
