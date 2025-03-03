use std::path::PathBuf;

use egui::TextureHandle;
use ollama_rs::generation::images::Image;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct OllamaTagsResult {
    pub models: Vec<OllamaModel>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct OllamaModelDetail {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    pub size: u64,
    pub details: OllamaModelDetail,
}

#[derive(Debug, Clone)]
pub struct ImageBase64Search {
    pub base64: Image,
    pub path: String,
}

#[derive(JsonSchema, PartialEq, Deserialize, Debug, Clone)]
pub struct ImageStructured {
    pub path: String,
    pub name: String,
    pub extension: String,
}

#[derive(serde::Deserialize, Default, serde::Serialize, Debug, Clone)]
pub struct DirectoryFiles {
    pub dir: String,
    pub files: Vec<String>,
    pub files_with_labels: Vec<FileWithLabel>,
}

#[derive(serde::Deserialize, Default, serde::Serialize, Debug, Clone)]
pub struct FileWithLabel {
    pub file: String,
    pub labels: Vec<String>,
}

#[derive(Clone)]
pub struct DirectoryImage {
    pub file: String,
    pub texture: TextureHandle,
}

#[derive(Clone)]
pub struct DirectoryImages {
    pub dir: PathBuf,
    pub images: Vec<DirectoryImage>,
}

// #[derive(Clone, Debug, PartialEq)]
#[derive(Clone)]
pub enum BroadcastMsg {
    // START -- Ollama settings & state
    OllamaRunning(Result<(), String>),
    GetOllamaRunning,

    SetOllamaURL(String),
    SetOllamaModels(Vec<OllamaModel>),

    GetOllamaURL,
    OllamaURL(String),

    GetOllamaModels,
    OllamaModels(Vec<OllamaModel>),

    // END -- Ollama settings & state
    PickedDirectory(PathBuf),
    RemovedDirectory(PathBuf),
    DirectoryFiles(PathBuf, Vec<String>),
    DirectoryImages(DirectoryImages),

    // -- labeling
    StartLabeling,
    GetLabelsForImage(String, String),
    FinishLabeling,
}
