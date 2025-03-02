use std::path::PathBuf;

use crate::{
    enums::{BroadcastMsg, DirectoryFiles},
    ollama_state::OllamaState,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(serde::Deserialize, Default, serde::Serialize, Debug, Clone)]
pub struct AppState {
    #[serde(skip)]
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    pub ollama_state: OllamaState,
    pub directories: Vec<PathBuf>,
    pub formats: Vec<String>,
    pub dir_files: Vec<DirectoryFiles>,
}

static APP_STATE_KEY: &str = "app_state";

impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // -- get storage values
        if let Some(storage) = cc.storage {
            let s = eframe::get_value(storage, APP_STATE_KEY).unwrap_or_default();
            // println!("{:?}", s);
            return s;
        }

        Self {
            action_tx: None,
            ollama_state: OllamaState::new(cc, String::from("http://127.0.0.1:11434/")),
            directories: vec![],
            formats: vec![],
            dir_files: vec![],
        }
    }

    pub fn init(&mut self) {
        self.ollama_state.init();
    }

    pub fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.ollama_state.save(storage);
        eframe::set_value(storage, APP_STATE_KEY, self);
    }

    pub fn remove_directory(&mut self, path: PathBuf) {
        self.directories.retain(|p| *p != path);
        self.dir_files
            .retain(|p| p.dir != path.to_string_lossy().to_string());
    }

    fn save_files_from_dir(&mut self, path: PathBuf, files: Vec<String>) {
        let dir = path.to_string_lossy().to_string();
        if self.dir_files.iter().all(|item| item.dir != dir) {
            let d_files = DirectoryFiles { dir, files };
            self.dir_files.push(d_files);
        }

        // if let Some(action_tx) = self.action_tx.clone() {
        //     let _ = action_tx.send(BroadcastMsg::ShowImages);
        // }
    }

    pub fn update(&mut self, msg: BroadcastMsg) {
        self.ollama_state.update(msg.clone());

        match msg {
            BroadcastMsg::DirectoryFiles(path, files) => {
                self.save_files_from_dir(path, files);
            }
            BroadcastMsg::PickedDirectory(dir) => {
                // println!("{} - dir", dir.to_string_lossy());
            }
            _ => {}
        }
    }

    pub fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.ollama_state.register_tx(action_tx.clone());
        self.action_tx = Some(action_tx);
    }
}
