use crate::{
    enums::{AgentEnum, BroadcastMsg},
    ollama_state::OllamaState,
};
use tokio::sync::mpsc::UnboundedSender;

pub struct AppState {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    pub ollama_state: OllamaState,
    pub active_agent: AgentEnum,
}

impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            action_tx: None,
            active_agent: AgentEnum::Chat,
            ollama_state: OllamaState::new(cc, String::from("http://127.0.0.1:11434/")),
        }
    }

    pub fn init(&mut self) {
        self.ollama_state.init();
    }

    pub fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.ollama_state.save(storage);
    }

    pub fn update(&mut self, msg: BroadcastMsg) {
        if let BroadcastMsg::SelectAgent(agent) = msg.clone() {
            self.active_agent = agent;
        }

        self.ollama_state.update(msg);
    }

    pub fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.ollama_state.register_tx(action_tx.clone());
        self.action_tx = Some(action_tx);
    }
}
