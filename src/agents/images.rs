use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crate::enums::{ImageBase64Search, ImageStructured};
use crate::utils::img_paths_to_base64;
use crate::{
    app_state::AppState,
    components::Component,
    enums::{AgentEnum, BroadcastMsg, ImagesStructured, OllamaModel},
    tools::{get_images_from_path, path_contains_substring, search_images_from_path},
};
use egui::{Color32, Frame, Id, Sense, UiBuilder};
use egui_flex::{Flex, FlexAlignContent, FlexItem};
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::options::GenerationOptions;
use ollama_rs::generation::parameters::{FormatType, JsonStructure};
use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};
use tokio::sync::mpsc::UnboundedSender;

use super::Agent;

const SYS_MSG: &str = r##"  
You are helpul desktop assistant mainly used for searching or listing images.
If you will not find any images from given path you will output only: I din't find any Images.
If you'll get any images from toll call response only with list of those images with format '- name of image file: absolute path'"##;

pub struct ImageAgent {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    app_state: Option<Arc<Mutex<AppState>>>,
    models: Vec<OllamaModel>,
    active_model: Option<OllamaModel>,
    history: Vec<ChatMessage>,
    coordinator: Option<
        Arc<
            tokio::sync::Mutex<
                Coordinator<
                    Vec<ChatMessage>,
                    (
                        get_images_from_path,
                        search_images_from_path,
                        // path_contains_substring,
                    ),
                >,
            >,
        >,
    >,
    sys_msg: RefCell<String>,
    all_found_images: Option<ImagesStructured>,
    images_to_search: Vec<ImageBase64Search>,
    images_found: Vec<ImageBase64Search>,
}

impl ImageAgent {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            app_state: None,
            models: vec![],
            active_model: None,
            history: vec![],
            coordinator: None,
            sys_msg: RefCell::new(SYS_MSG.to_string()),
            all_found_images: None,
            images_to_search: vec![],
            images_found: vec![],
        }
    }

    fn get_coordinator(
        &mut self,
        active_model: OllamaModel,
        history: Vec<ChatMessage>,
        app_state: Option<Arc<Mutex<AppState>>>,
        action_tx: Option<UnboundedSender<BroadcastMsg>>,
    ) -> Arc<
        tokio::sync::Mutex<
            Coordinator<
                Vec<ChatMessage>,
                (
                    get_images_from_path,
                    search_images_from_path,
                    // path_contains_substring,
                ),
            >,
        >,
    > {
        let (url, port) = self.get_ollama_url(app_state);

        let ollama = Ollama::new(url, port);
        let model = active_model.name.clone();
        let tools = ollama_rs::tool_group![
            get_images_from_path,
            search_images_from_path,
            // path_contains_substring
        ];
        let coordinator = Arc::new(tokio::sync::Mutex::new(Coordinator::new_with_tools(
            ollama,
            model,
            history.clone(),
            tools,
        )));

        if let Some(tx) = action_tx.clone() {
            let _ = tx.send(BroadcastMsg::SelectAgentModel(active_model));
        }

        coordinator
    }

    fn msg_to_coordinator(&mut self, msg: ChatMessage) {
        if let Some(coordinator) = self.coordinator.clone() {
            let action_tx = self.action_tx.clone();
            let sys_msg = self.sys_msg.borrow().clone();

            tokio::spawn(async move {
                let _ = Self::send_chat_msg(action_tx, coordinator, msg.clone(), sys_msg).await;
            });
        }
    }

    fn msg_to_vision(&mut self, model_name: String, prompt: String, img: ImageBase64Search) {
        let sys_msg = r##"You are an AI vision model. 
        You will receive an image and a question asking whether a specific object or feature is present in the image. 
        Respond only with 'true' if it is present or 'false' if it is not. 
        Do not provide any additional explanations or details."##;
        let (url, port) = self.get_ollama_url(self.app_state.clone());
        let ollama = Ollama::new(url, port);
        if let Some(action_tx) = self.action_tx.clone() {
            tokio::spawn(async move {
                let res = ollama
                    .generate(
                        GenerationRequest::new(
                            model_name,
                            format!(
                                "{}. Answer alway only with 'true' or 'false'.",
                                prompt.clone()
                            ),
                        )
                        .add_image(img.clone().base64)
                        .system(sys_msg)
                        .options(GenerationOptions::default().temperature(0.0)),
                    )
                    .await;
                if let Ok(resp) = res {
                    println!("{:?} desc vision search", &resp.response);

                    let _ = action_tx.send(BroadcastMsg::GetVisionSeachResult(
                        prompt,
                        resp.response.clone(),
                        img,
                    ));
                }
            });
        }
    }

    fn get_vision_model(&mut self) -> Option<String> {
        if let Some(app_state) = self.app_state.clone() {
            let a_state = app_state.lock().unwrap();
            let v_models = a_state.ollama_state.get_vision_models();
            if !v_models.is_empty() {
                let model_name = v_models[0].name.clone();
                println!("SELECTED FIRST VISION MODEL: {}", model_name.clone());
                return Some(model_name);
            }
        }
        None
    }

    fn image_to_coordinator(&mut self, prompt: String, img: ImageBase64Search) {
        let msg = ChatMessage::new(
            ollama_rs::generation::chat::MessageRole::User,
            prompt.clone(),
        );
        // msg.add_image(img.base64);

        if let Some(vision_model) = self.get_vision_model() {
            self.msg_to_vision(vision_model, prompt, img);
        } else {
            self.msg_to_coordinator(msg.clone());
        }
    }

    async fn send_chat_msg(
        action_tx: Option<UnboundedSender<BroadcastMsg>>,
        coordinator: Arc<
            tokio::sync::Mutex<
                Coordinator<Vec<ChatMessage>, (get_images_from_path, search_images_from_path)>,
            >,
        >,
        msg: ChatMessage,
        sys_msg: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // ) {
        let mut msgs = vec![];
        if !sys_msg.trim().is_empty() {
            let sys_chat_msg =
                ChatMessage::new(ollama_rs::generation::chat::MessageRole::System, sys_msg);
            msgs.push(sys_chat_msg);
        }
        msgs.push(msg.clone());

        println!("USER: {}", msg.content.clone());
        let resp = coordinator.lock().await.chat(msgs).await?;

        if let Some(tx) = action_tx {
            let _ = tx.send(BroadcastMsg::GetChatSubReponse(resp.message.clone()));
            let _ = tx.send(BroadcastMsg::GetStructuredOutput(
                resp.message.content.clone(),
            ));
        }

        println!("{:?} CHAT RESPONSE", resp);

        Ok(())
    }

    fn decide_search_msg(&mut self, msg: ChatMessage) {
        let sys_msg = r##"You are not AI Assistant.You are an AI agent that determines whether 
        the user is requesting an image containing a specific object, person, or scenery. 
        Respond with 'true' if the user's request includes a description of what should be present in the image. 
        Otherwise, respond with 'false'. Only return 'true' or 'false' without any explanation."##;

        let (url, port) = self.get_ollama_url(self.app_state.clone());
        let ollama = Ollama::new(url, port);
        if let Some(action_tx) = self.action_tx.clone() {
            if let Some(model) = self.active_model.clone() {
                tokio::spawn(async move {
                    let res = ollama
                        .generate(
                            GenerationRequest::new(model.name.clone(), msg.clone().content)
                                .system(sys_msg)
                                .options(GenerationOptions::default().temperature(0.0)),
                        )
                        .await;
                    if let Ok(resp) = res {
                        let _ = action_tx.send(BroadcastMsg::GetDescriptionImageSearch(
                            resp.response.clone(),
                            msg,
                        ));
                        println!("{:?} desc search", &resp.response);
                    }
                });
            }
        }
    }

    fn get_structured_output(&mut self, msg: String) {
        let (url, port) = self.get_ollama_url(self.app_state.clone());
        let ollama = Ollama::new(url, port);
        let format = FormatType::StructuredJson(JsonStructure::new::<ImagesStructured>());

        if let Some(action_tx) = self.action_tx.clone() {
            if let Some(model) = self.active_model.clone() {
                tokio::spawn(async move {
                    let res = ollama
                        .generate(
                            GenerationRequest::new(model.name.clone(), msg)
                                .format(format)
                                .options(GenerationOptions::default().temperature(0.0)),
                        )
                        .await;
                    if let Ok(resp) = res {
                        println!("{:?}", &resp.response);
                        if let Ok(json) =
                            serde_json::from_str::<ImagesStructured>(&resp.response.clone())
                        {
                            let _ = action_tx.send(BroadcastMsg::GetFoundImages(json.clone()));
                            println!("{:?}", json);
                        }
                    }
                });
            }
        }
    }

    fn search_images_by_vision(&mut self, msg: ChatMessage) {
        // -- get "is {something} on this picutre ?" prompt
        let sys_msg = r##"
            You are not AI Assistant. 
            Your only task is to rephrase user prompt with description of what to find on pictures 
            into a question prompt in format 'Is {description} on this picture?'"##;

        let (url, port) = self.get_ollama_url(self.app_state.clone());
        let ollama = Ollama::new(url, port);
        if let Some(action_tx) = self.action_tx.clone() {
            if let Some(model) = self.active_model.clone() {
                tokio::spawn(async move {
                    let res = ollama
                        .generate(
                            GenerationRequest::new(model.name.clone(), msg.clone().content)
                                .system(sys_msg)
                                .options(GenerationOptions::default().temperature(0.0)),
                        )
                        .await;
                    if let Ok(resp) = res {
                        let _ = action_tx.send(BroadcastMsg::GetRephraseImageSearchPrompt(
                            resp.response.clone(),
                        ));
                        println!("{:?} rephrase prompt", &resp.response);
                    }
                });
            }
        }
    }

    fn start_vision_search(&mut self, prompt: String) {
        if let Some(images) = self.all_found_images.clone() {
            let base64_images = img_paths_to_base64(images.images);
            self.images_to_search = base64_images;
            self.images_found.clear();
            self.next_vision_search(prompt);
        }
    }

    fn next_vision_search(&mut self, prompt: String) {
        if let Some(img) = self.images_to_search.pop() {
            self.image_to_coordinator(prompt, img);
        } else {
            self.finished_image_search();
        }
    }

    fn validate_checked_image(&mut self, answer: String, img: ImageBase64Search) {
        if answer.contains("Yes")
            || answer.contains("yes")
            || answer.contains("True")
            || answer.contains("true")
        {
            self.images_found.push(img);
        }
    }

    fn finished_image_search(&mut self) {
        if let Some(action_tx) = self.action_tx.clone() {
            // let mut img_s = vec![];
            let ims = self
                .images_found
                .iter()
                .map(|i| ImageStructured {
                    path: i.path.clone(),
                    name: "test".to_string(),
                    extension: "png".to_string(),
                })
                .collect();

            let mut img_struct = ImagesStructured { images: ims };
            let _ = action_tx.send(BroadcastMsg::GetFoundImages(img_struct));
        }
    }

    fn change_active_model(&mut self) {
        if let Some(active_model) = self.active_model.clone() {
            self.history.clear();
            self.coordinator = Some(self.get_coordinator(
                active_model.clone(),
                self.history.clone(),
                self.app_state.clone(),
                self.action_tx.clone(),
            ));
        }
    }

    fn grid_ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("")
            .num_columns(2)
            .spacing(egui::Vec2 { x: 4.0, y: 0.0 })
            .show(ui, |ui| {
                // --------
                ui.small("agent:");
                ui.label(self.name().to_string());
                ui.end_row();

                // --------
                let previous_value = self.active_model.clone();
                ui.small("model:");
                if let Some(m) = self.active_model.clone() {
                    egui::ComboBox::from_id_salt(Id::new(format!("{}_combo", m.name)))
                        .width(100.0)
                        .truncate()
                        .selected_text(m.name)
                        .show_ui(ui, |ui| {
                            for om in self.models.iter() {
                                ui.selectable_value(
                                    &mut self.active_model,
                                    Some(om.clone()),
                                    &om.name,
                                );
                            }
                        });

                    if self.active_model != previous_value {
                        self.change_active_model();
                    }
                }
                ui.end_row();

                // --------
                ui.small("");
                let mut sys_msg = self.sys_msg.take();
                self.advanced_ui(&mut sys_msg, ui);
                self.sys_msg.replace(sys_msg);
                ui.end_row();
            });
    }
}

impl Agent for ImageAgent {
    fn name(&self) -> &'static str {
        "Images"
    }

    fn description(&self) -> &'static str {
        "Searching images in local PC"
    }

    fn agent(&self) -> AgentEnum {
        AgentEnum::Images
    }

    fn select_agent(&mut self, agent: AgentEnum) {
        if agent.eq(&self.agent()) {
            if let Some(app_state) = self.app_state.clone() {
                let models = app_state.lock().unwrap().ollama_state.models.clone();
                if !models.is_empty() {
                    let model = models[0].clone();
                    self.coordinator = Some(self.get_coordinator(
                        model,
                        self.history.clone(),
                        self.app_state.clone(),
                        self.action_tx.clone(),
                    ));
                }
            }
        } else {
            self.coordinator = None;
        }
    }
}

impl Component for ImageAgent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn register_app_state(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = Some(app_state);
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }

    fn update(&mut self, msg: BroadcastMsg) {
        match msg {
            BroadcastMsg::SelectAgent(agent) => {
                self.select_agent(agent);
            }
            BroadcastMsg::OllamaModels(models) => {
                self.models = models.clone();
                if !self.models.is_empty() {
                    if self.active_model.is_none() {
                        self.active_model = Some(models[0].clone());
                    } else {
                        let active_model = self.active_model.clone().unwrap();
                        if !self.models.contains(&active_model) {
                            self.active_model = Some(models[0].clone());
                        }
                    }

                    if let Some(aps) = self.app_state.clone() {
                        let agent = aps.lock().unwrap().active_agent.clone();
                        self.select_agent(agent);
                    }
                }
            }
            BroadcastMsg::SendUserMessage(msg) => {
                self.decide_search_msg(msg);
                // self.msg_to_coordinator(msg);
            }
            BroadcastMsg::GetStructuredOutput(msg) => {
                self.get_structured_output(msg);
            }
            BroadcastMsg::GetFoundImages(json) => {
                if let Some(ref mut f_images) = self.all_found_images.clone() {
                    for img in json.clone().images {
                        if !f_images.images.iter().any(|i| i == &img) {
                            f_images.images.push(img);
                        }
                    }
                } else {
                    self.all_found_images = Some(json);
                }
            }
            BroadcastMsg::GetDescriptionImageSearch(is_by_desc, msg) => {
                if is_by_desc == "true" && self.all_found_images.is_some() {
                    self.search_images_by_vision(msg);
                } else {
                    self.msg_to_coordinator(msg);
                }
            }
            BroadcastMsg::GetRephraseImageSearchPrompt(prompt) => {
                self.start_vision_search(prompt);
            }
            BroadcastMsg::GetVisionSeachResult(prompt, answer, img) => {
                self.validate_checked_image(answer, img);
                // println!("{}, {}", answer, img.path);
                self.next_vision_search(prompt);
            }
            _ => {}
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let resp = ui
            .scope_builder(
                UiBuilder::new()
                    .id_salt("images_agent_component")
                    .sense(Sense::click()),
                |ui| {
                    let resp = ui.response();
                    let visual = ui.style().interact(&resp);

                    Frame::canvas(ui.style())
                        .fill(visual.bg_fill.linear_multiply(0.3))
                        .stroke(visual.bg_stroke)
                        .show(ui, |ui| {
                            Flex::horizontal()
                                .align_content(FlexAlignContent::Stretch)
                                .w_full()
                                .show(ui, |flex| {
                                    flex.add_ui(FlexItem::new().grow(0.8), |ui| {
                                        // flex.add_ui(FlexItem::new().basis(200.0), |ui| {
                                        self.grid_ui(ui);
                                    });

                                    flex.add_ui(FlexItem::new().basis(25.0), |ui| {
                                        // flex.add_ui(FlexItem::new().grow(0.1), |ui| {
                                        if let Some(app_state) = self.app_state.clone() {
                                            let active_agent =
                                                app_state.lock().unwrap().active_agent.clone();
                                            if active_agent.eq(&self.agent()) {
                                                egui::Frame::default()
                                                    .stroke(
                                                        ui.visuals()
                                                            .widgets
                                                            .noninteractive
                                                            .bg_stroke,
                                                    )
                                                    .rounding(
                                                        ui.visuals()
                                                            .widgets
                                                            .noninteractive
                                                            .rounding,
                                                    )
                                                    .fill(Color32::from_rgb(200, 200, 0))
                                                    .show(ui, |ui| {
                                                        ui.set_width(ui.available_width());
                                                        // ui.set_max_height(50.0);
                                                        ui.set_height(50.0);
                                                        ui.label("");
                                                    });
                                            }
                                        }
                                    });
                                });
                            ui.end_row();
                        });
                },
            )
            .response;

        if resp.clicked() {
            self.send_selected_agent(self.action_tx.clone());
        }
    }
}
