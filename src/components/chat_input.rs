use super::Component;
use crate::enums::BroadcastMsg;
use egui::{KeyboardShortcut, Margin, Modifiers};
use egui_flex::{Flex, FlexAlignContent, FlexItem};
use ollama_rs::generation::chat::ChatMessage;
use tokio::sync::mpsc::UnboundedSender;

static SEND_BUTTON_SIZE: f32 = 100.0;

pub struct ChatInput {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    pub input_text: String,
}

impl ChatInput {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            input_text: String::new(),
        }
    }

    fn send_user_msg(&mut self, msg: String) {
        if let Some(tx) = self.action_tx.clone() {
            let chat_msg = ChatMessage::user(msg);
            let _ = tx.send(BroadcastMsg::SendUserMessage(chat_msg));
            // let _ = tx.send(BroadcastMsg::SendUserMessage(msg));
        }
    }
}

impl Component for ChatInput {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }

    fn update(&mut self, _msg: BroadcastMsg) {}

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);

        Flex::horizontal()
            .align_content(FlexAlignContent::Stretch)
            .wrap(false)
            .show(ui, |flex| {
                flex.add_ui(FlexItem::default().grow(4.0), |ui| {
                    let mut text_size = ui.available_size();
                    text_size.x -= SEND_BUTTON_SIZE;

                    let resp = ui.add_sized(
                        text_size,
                        egui::TextEdit::multiline(&mut self.input_text)
                            .return_key(KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::Enter))
                            .hint_text("Type here..")
                            .margin(Margin::symmetric(14.0, 18.0)),
                    );
                    if resp.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.is_none())
                    {
                        self.send_user_msg(self.input_text.clone());
                        self.input_text = String::new();
                    }
                });

                flex.add_ui(FlexItem::default().basis(SEND_BUTTON_SIZE), |ui| {
                    let send_button =
                        ui.add_sized([90.0, ui.available_height()], egui::Button::new("send"));
                    if send_button.clicked() {
                        self.send_user_msg(self.input_text.clone());
                        self.input_text = String::new();
                    }
                });
            });

        ui.add_space(8.0);
    }
}
