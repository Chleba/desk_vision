use std::path::PathBuf;

use crate::{
    components::labels,
    enums::{BroadcastMsg, DirectoryFiles, FileWithLabel},
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
            println!("{:?}", s);
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
            let mut d_files = vec![];
            for f in files.iter() {
                d_files.push(FileWithLabel {
                    file: f.to_string(),
                    labels: vec![],
                });
            }

            let d_files = DirectoryFiles {
                dir,
                files_with_labels: d_files,
            };
            self.dir_files.push(d_files);
        }
    }

    fn add_labels_to_file(&mut self, file: String, labels: String) {
        println!("File: {}, labels: {}", file, labels);

        let l_labels: Vec<String> = labels
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        for dir in self.dir_files.iter_mut() {
            if let Some(f_file) = dir.files_with_labels.iter_mut().find(|f| f.file == file) {
                f_file.labels = l_labels.clone();
            }
        }
    }

    pub fn update(&mut self, msg: BroadcastMsg) {
        self.ollama_state.update(msg.clone());

        match msg {
            BroadcastMsg::DirectoryFiles(path, files) => {
                self.save_files_from_dir(path, files);
            }
            BroadcastMsg::GetLabelsForImage(file, labels) => {
                self.add_labels_to_file(file, labels);
            }
            _ => {}
        }
    }

    pub fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.ollama_state.register_tx(action_tx.clone());
        self.action_tx = Some(action_tx);
    }
}
