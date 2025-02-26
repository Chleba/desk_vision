use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{config::SUPPORTED_IMAGE_FORMATS, enums::BroadcastMsg};

pub struct TopMenu {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    formats_checks: Vec<bool>,
}

impl TopMenu {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            formats_checks: vec![false; SUPPORTED_IMAGE_FORMATS.len()],
        }
    }
}

impl Component for TopMenu {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn render(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // -- main button
                ui.menu_button("Imager", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // -- formats button
                ui.menu_button("Formats", |ui| {
                    ui.label("Filter searched formats");
                    ui.separator();
                    for (i, checked) in self.formats_checks.iter_mut().enumerate() {
                        ui.checkbox(checked, SUPPORTED_IMAGE_FORMATS[i]);
                    }
                });
            });
        });
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }
}
