use super::{chat_input::ChatInput, Component};
use crate::enums::BroadcastMsg;
use tokio::sync::mpsc::UnboundedSender;

pub struct ChatBottomPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    chat_input: ChatInput,
}

impl ChatBottomPanel {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            chat_input: ChatInput::new(),
        }
    }
}

impl Component for ChatBottomPanel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.chat_input.register_tx(action_tx.clone());

        self.action_tx = Some(action_tx);
    }

    fn update(&mut self, msg: BroadcastMsg) {
        self.chat_input.update(msg);
    }

    fn render(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.set_height(100.0);

            self.chat_input.ui(ui);
        });
    }
}
