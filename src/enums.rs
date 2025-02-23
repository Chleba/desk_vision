use ollama_rs::generation::{chat::ChatMessage, images::Image};
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

#[derive(JsonSchema, Deserialize, Debug, Clone)]
pub struct ImagesStructured {
    pub images: Vec<ImageStructured>,
}

#[derive(Debug, Clone)]
pub struct DeskMessage {
    pub chat_message: Option<ChatMessage>,
    pub images: Option<ImagesStructured>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AgentEnum {
    Chat,
    WebScrape,
    Images,
}

// #[derive(Clone, Debug, PartialEq)]
#[derive(Clone, Debug)]
pub enum BroadcastMsg {
    OllamaRunning(Result<(), String>),
    GetOllamaRunning,

    SetOllamaURL(String),
    SetOllamaModels(Vec<OllamaModel>),

    GetOllamaURL,
    OllamaURL(String),

    GetOllamaModels,
    OllamaModels(Vec<OllamaModel>),

    SelectAgent(AgentEnum),
    SelectAgentModel(OllamaModel),

    SendUserMessage(ChatMessage),
    GetChatSubReponse(ChatMessage),
    GetChatReponse(ChatMessage),
    GetStructuredOutput(String),

    GetFoundImages(ImagesStructured),
    GetDescriptionImageSearch(String, ChatMessage),

    GetRephraseImageSearchPrompt(String),
    GetVisionSeachResult(String, String, ImageBase64Search),
    FinishedImageSearch,
}
