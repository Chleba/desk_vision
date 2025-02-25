use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::ollama_settings::OllamaSettings;
use super::Component;
use crate::{app_state::AppState, enums::BroadcastMsg};
use egui::{Align, Color32, Grid, RichText, ScrollArea};
use tokio::sync::mpsc::UnboundedSender;

pub struct TopPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    input_text: String,
    picked_directories: Vec<PathBuf>,
    app_state: Option<Arc<Mutex<AppState>>>,
    ollama_button: OllamaSettings,
    ollama_connected: bool,
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
        }
    }

    fn pick_dir(&mut self, path: PathBuf) {
        if !self.picked_directories.contains(&path) {
            self.picked_directories.push(path);
            {
                if let Some(app_state) = self.app_state.clone() {
                    app_state.lock().unwrap().directories = self.picked_directories.clone();
                }
            }
        }
    }

    fn remove_directory(&mut self, path: PathBuf) {
        if self.picked_directories.contains(&path) {
            self.picked_directories.retain(|p| *p != path);
            {
                if let Some(app_state) = self.app_state.clone() {
                    app_state.lock().unwrap().directories = self.picked_directories.clone();
                }
            }
        }
    }

    fn draw_left_side(&mut self, ui: &mut egui::Ui) {
        let mut size = ui.available_size();
        size.x /= 2.0;
        ui.vertical(|ui| {
            egui::Grid::new("left_grid").num_columns(2).show(ui, |ui| {
                ui.label("Search images:");
                // -- search input
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.input_text).hint_text("Search here.."),
                );
                if resp.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.is_none())
                {
                    // self.send_user_msg(self.input_text.clone());
                    // self.input_text = String::new();
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

            ui.label("Added directories:");
            ScrollArea::vertical()
                // .max_width(size.x / 2.0)
                .max_width(420.0)
                .show(ui, |ui| {
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

        // // ui.vertical(|ui| {
        // let half_w = ui.available_width() / 2.0;
        // egui::Grid::new("right_grid")
        //     // .max_col_width(half_w / 2.0)
        //     .num_columns(2)
        //     .show(ui, |ui| {
        //         ui.label(" - ");
        //         ui.label("Searching for images");
        //
        //         ui.end_row();
        //
        //         ui.label(" - ");
        //         ui.label("Labeling images");
        //     });
        // // });

        ui.label("- Searching for images");
        ui.label("- Labeling images");
    }
}

impl Component for TopPanel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn init(&mut self) {
        self.ollama_button.init();
    }

    fn update(&mut self, msg: BroadcastMsg) {
        self.ollama_button.update(msg.clone());

        if let BroadcastMsg::OllamaRunning(r) = msg {
            self.ollama_connected = r.is_ok()
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
