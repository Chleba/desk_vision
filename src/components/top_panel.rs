use std::path::PathBuf;

use super::Component;
use crate::enums::BroadcastMsg;
use egui::Align;
use tokio::sync::mpsc::UnboundedSender;

pub struct TopPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    input_text: String,
    picked_directories: Vec<PathBuf>,
}

impl TopPanel {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            input_text: "".to_string(),
            picked_directories: vec![],
        }
    }

    fn pick_dir(&mut self, path: PathBuf) {
        if !self.picked_directories.contains(&path) {
            self.picked_directories.push(path);
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
        });
    }

    fn draw_right_side(&mut self, ui: &mut egui::Ui) {
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

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }
}
