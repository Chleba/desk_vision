use super::{chat_input::ChatInput, messages::Messages, Component};
use crate::{components::ollama_settings::OllamaSettings, enums::BroadcastMsg};
use egui::{Color32, RichText, ScrollArea};
use tokio::sync::mpsc::UnboundedSender;

pub struct MainPanel {
    ollama_button: OllamaSettings,
    chat_input: ChatInput,
    messages: Messages,

    ollama_connected: bool,
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
}

impl MainPanel {
    pub fn new() -> Self {
        Self {
            ollama_button: OllamaSettings::new(),
            chat_input: ChatInput::new(),
            messages: Messages::new(),

            ollama_connected: false,
            action_tx: None,
        }
    }
}

impl Component for MainPanel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn init(&mut self) {
        self.ollama_button.init();
        self.chat_input.init();
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.ollama_button.register_tx(action_tx.clone());
        self.chat_input.register_tx(action_tx.clone());
        self.messages.register_tx(action_tx.clone());

        self.action_tx = Some(action_tx);
    }

    fn update(&mut self, msg: BroadcastMsg) {
        self.ollama_button.update(msg.clone());
        self.chat_input.update(msg.clone());
        self.messages.update(msg.clone());

        if let BroadcastMsg::OllamaRunning(r) = msg {
            self.ollama_connected = r.is_ok()
        }
    }

    fn render(&mut self, ctx: &egui::Context) {
        ctx.request_repaint_after_secs(1.0);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // -- ollama menu button
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    self.ollama_button.ui(ui);
                    if !self.ollama_connected {
                        ui.small(
                            RichText::new("not connected").color(Color32::from_rgb(255, 0, 0)),
                        );
                    } else {
                        ui.small(RichText::new("connected").color(Color32::from_rgb(0, 255, 0)));
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(8.0);

            ui.vertical_centered_justified(|ui| {
                egui::Frame::default().show(ui, |ui| {
                    ScrollArea::vertical()
                        .animated(false)
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            self.messages.ui(ui);
                        });
                });
            });
        });
    }
}
