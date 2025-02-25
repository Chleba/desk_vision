use std::path::PathBuf;

use crate::{enums::BroadcastMsg, ollama_state::OllamaState};
use tokio::sync::mpsc::UnboundedSender;

#[derive(serde::Deserialize, Default, serde::Serialize, Debug, Clone)]
pub struct AppState {
    #[serde(skip)]
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    #[serde(skip)]
    pub ollama_state: OllamaState,
    pub directories: Vec<PathBuf>,
}

static APP_STATE_KEY: &str = "app_state";

impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // -- get storage values
        if let Some(storage) = cc.storage {
            let s = eframe::get_value(storage, APP_STATE_KEY).unwrap_or_default();
            println!("{:?}", s);
            return s;
        }

        Self {
            action_tx: None,
            ollama_state: OllamaState::new(cc, String::from("http://127.0.0.1:11434/")),
            directories: vec![],
        }
    }

    pub fn init(&mut self) {
        self.ollama_state.init();
    }

    pub fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_STATE_KEY, self);
        self.ollama_state.save(storage);
    }

    pub fn update(&mut self, msg: BroadcastMsg) {
        self.ollama_state.update(msg);
    }

    pub fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.ollama_state.register_tx(action_tx.clone());
        self.action_tx = Some(action_tx);
    }
}
