use crate::{
    app_state::{self, AppState},
    components::Component,
    enums::{AgentEnum, BroadcastMsg, OllamaModel},
    utils::animate_continuous,
};
use eframe::emath::Vec2;
use egui::{
    CollapsingHeader, Color32, Frame, Id, KeyboardShortcut, Margin, Modifiers, Sense, UiBuilder,
};
use ollama_rs::{
    coordinator::Coordinator,
    generation::{
        chat::ChatMessage,
        tools::implementations::{Calculator, DDGSearcher, Scraper},
    },
    Ollama,
};
use std::time::Duration;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

pub mod chat;
pub mod images;
pub mod web_scrape;

pub trait AgentComponent: Component + Agent {}
impl<T: ?Sized + Component + Agent> AgentComponent for T {}

pub trait Agent: Any {
    fn name(&self) -> &'static str;
    #[allow(dead_code)]
    fn description(&self) -> &'static str;
    fn agent(&self) -> AgentEnum;

    #[allow(unused_variables)]
    fn select_agent(&mut self, agent: AgentEnum) {}

    fn send_selected_agent(&mut self, action: Option<UnboundedSender<BroadcastMsg>>) {
        if let Some(tx) = action {
            let _ = tx.send(BroadcastMsg::SelectAgent(self.agent()));
        }
    }

    fn get_msg_vec(&self, sys_msg: String) -> Vec<ChatMessage> {
        let mut msgs = vec![];
        if !sys_msg.trim().is_empty() {
            let sys_chat_msg =
                ChatMessage::new(ollama_rs::generation::chat::MessageRole::System, sys_msg);
            msgs.push(sys_chat_msg);
        }
        msgs
    }

    fn advanced_ui(&mut self, sys_msg: &mut String, ui: &mut egui::Ui) {
        // fn advanced_ui(&mut self, sys_msg: &mut String, ui: &mut egui::Ui) {
        CollapsingHeader::new("advanced options:")
            .default_open(false)
            .show(ui, |ui| {
                ui.small("system message:");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::multiline(sys_msg)
                            .return_key(KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::Enter))
                            .desired_rows(2)
                            .hint_text("Type here..")
                            .margin(Margin::symmetric(4.0, 4.0)),
                    );
                    if ui.button("save").clicked() {
                        println!("save system message: {}", sys_msg.clone());
                    }
                });
            });
    }

    // fn get_coordinator(
    //     &mut self,
    //     active_model: OllamaModel,
    //     history: Vec<ChatMessage>,
    //     app_state: Option<Arc<Mutex<AppState>>>,
    //     action_tx: Option<UnboundedSender<BroadcastMsg>>,
    // ) -> Arc<tokio::sync::Mutex<Coordinator<Vec<ChatMessage>, (DDGSearcher, (Scraper, Calculator))>>>
    // {
    //     let (url, port) = self.get_ollama_url(app_state);
    //
    //     let ollama = Ollama::new(url, port);
    //     let model = active_model.name.clone();
    //     let tools = (DDGSearcher::new(), (Scraper {}, Calculator {}));
    //     let coordinator = Arc::new(tokio::sync::Mutex::new(Coordinator::new_with_tools(
    //         ollama,
    //         model,
    //         history.clone(),
    //         tools,
    //     )));
    //
    //     if let Some(tx) = action_tx.clone() {
    //         let _ = tx.send(BroadcastMsg::SelectAgentModel(active_model));
    //     }
    //
    //     coordinator
    // }

    fn get_ollama_url(&mut self, app_state: Option<Arc<Mutex<AppState>>>) -> (String, u16) {
        if let Some(state) = app_state.clone() {
            let url = state.lock().unwrap().ollama_state.url.clone();
            if let Some((base_url, port)) = url.rsplit_once(':') {
                if let Ok(port_num) = port.parse::<u16>() {
                    return (base_url.to_string(), port_num);
                }
            }
        }
        ("http://localhost/".to_string(), 11343)
    }
}
