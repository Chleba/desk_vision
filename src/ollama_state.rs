use crate::{
    enums::{BroadcastMsg, OllamaModel, OllamaTagsResult},
    utils::spawn,
};
use futures::TryFutureExt;
use garde::external::compact_str::CompactStringExt;
use tokio::sync::mpsc::UnboundedSender;

#[derive(serde::Deserialize, Default, serde::Serialize, Debug, Clone)]
pub struct OllamaState {
    #[serde(skip)]
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    pub url: String,
    #[serde(skip)]
    pub models: Vec<OllamaModel>,
}

static OLLAMA_STATE_KEY: &str = "ollama_state";

impl OllamaState {
    pub fn new(cc: &eframe::CreationContext<'_>, url: String) -> Self {
        // -- get storage values
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, OLLAMA_STATE_KEY).unwrap_or_default();
        }

        Self {
            action_tx: None,
            url,
            models: vec![],
        }
    }

    pub fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, OLLAMA_STATE_KEY, self);
    }

    pub fn init(&mut self) {
        self.send_check_ollama_url();
        self.send_get_tags();
    }

    pub fn get_vision_models(&self) -> Vec<OllamaModel> {
        let mut models = vec![];
        for m in self.models.clone() {
            if m.details.families.contains(&"clip".to_string()) {
                models.push(m);
            }
        }
        models
    }

    pub fn update(&mut self, msg: BroadcastMsg) {
        let action_tx = self.action_tx.clone();
        match msg {
            BroadcastMsg::SetOllamaURL(url) => {
                self.set_ollama_url(url);
            }
            BroadcastMsg::SetOllamaModels(models) => {
                self.models = models;
                if let Some(tx) = action_tx {
                    let _ = tx.send(BroadcastMsg::OllamaModels(self.models.clone()));
                }
            }
            BroadcastMsg::GetOllamaURL => {
                if let Some(tx) = action_tx {
                    let _ = tx.send(BroadcastMsg::OllamaURL(self.url.clone()));
                }
            }
            BroadcastMsg::GetOllamaModels => {
                if let Some(tx) = action_tx {
                    let _ = tx.send(BroadcastMsg::OllamaModels(self.models.clone()));
                }
            }
            _ => {}
        }
    }

    fn send_check_ollama_url(&mut self) {
        spawn(Self::check_ollama_url(
            self.url.clone(),
            self.action_tx.clone(),
        ));
    }

    async fn check_ollama_url(url: String, action_tx: Option<UnboundedSender<BroadcastMsg>>) {
        let ollama_status = reqwest::get(url).and_then(reqwest::Response::text).await;
        match ollama_status {
            Ok(s) => {
                if s == "Ollama is running" {
                    if let Some(tx) = action_tx {
                        let _ = tx.send(BroadcastMsg::OllamaRunning(Ok(())));
                    }
                }
            }
            Err(_e) => {
                if let Some(tx) = action_tx {
                    let _ = tx.send(BroadcastMsg::OllamaRunning(Err(
                        "Ollama is not running".to_string()
                    )));
                }
            }
        }
    }

    fn set_ollama_url(&mut self, url: String) {
        self.url = url;

        // -- check if ollama is connected
        self.send_check_ollama_url();
        // -- ollama url has changed, we need to download new tags
        self.send_get_tags();
    }

    fn send_get_tags(&mut self) {
        spawn(Self::get_tags(self.url.clone(), self.action_tx.clone()));
    }

    async fn get_tags(url: String, action_tx: Option<UnboundedSender<BroadcastMsg>>) {
        let tags: Result<OllamaTagsResult, _> = reqwest::get(format!("{}/api/tags", url))
            .and_then(reqwest::Response::json)
            .await;

        match tags {
            Ok(t) => {
                if let Some(tx) = action_tx {
                    // println!("{:?} - ollama models tags", t.models);
                    let _ = tx.send(BroadcastMsg::SetOllamaModels(t.models));
                }
            }
            Err(e) => {
                println!("{:?} - Error getting ollama tags", e);
            }
        }
    }

    pub fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }
}
