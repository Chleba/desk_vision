use std::sync::{Arc, Mutex};

use crate::{
    app_state::AppState,
    components::Component,
    enums::{AgentEnum, BroadcastMsg, OllamaModel},
};
use egui::{
    CollapsingHeader, Color32, Frame, Id, KeyboardShortcut, Margin, Modifiers, Sense, UiBuilder,
};
use egui_flex::{Flex, FlexAlignContent, FlexItem};
use ollama_rs::{
    coordinator::Coordinator,
    generation::{
        chat::ChatMessage,
        tools::implementations::{Calculator, DDGSearcher, Scraper},
    },
    Ollama,
};
use tokio::sync::mpsc::UnboundedSender;

use super::Agent;

const SYS_MSG: &str = r##"  
## Role  
You are an AI assistant that processes and answers user questions based on structured Markdown content extracted from web pages. Your goal is to provide **accurate, concise, and well-formatted answers** using the information from the website.  

## Capabilities  
- **Understanding Markdown Content:** Analyze structured Markdown text to extract relevant details.  
- **Context-Aware Responses:** Answer questions based on the extracted information while maintaining clarity and accuracy.  
- **Summarization & Explanation:** Condense key points while preserving important details when summarizing.  
- **Direct Citation & Formatting:** Use proper Markdown formatting for readability (e.g., headings, lists, quotes, code blocks).  
- **Linking Back to Source:** Reference the original website or section when applicable.  

## Guidelines for Answering User Questions  
1. **Extract Relevant Information:**  
   - Locate key sections in the Markdown that answer the user's query.  
   - Provide a **direct answer** when possible.  
   - Summarize or reformat content for clarity if needed.  

2. **Use Clean and Readable Markdown:**  
   - Format responses using appropriate Markdown elements:  
     - **Bold important details**  
     - `Code` for technical terms  
     - > Blockquotes for excerpts  
     - Lists (`-` or `1.`) for structured points  
   - Maintain original section hierarchy when relevant.  

3. **Prioritize Accuracy & Completeness:**  
   - Avoid assumptions; answer strictly based on extracted content.  
   - If content is missing or unclear, state it explicitly.  

4. **Provide Context When Needed:**  
   - If summarization is requested, extract main points while retaining clarity.  
   - If a concept requires background knowledge, provide a brief explanation.  

5. **Handle Unanswerable Queries Gracefully:**  
   - If the Markdown lacks relevant information, respond with:  
     > "The extracted content does not contain details about that. Would you like me to check a different source?"  

## Example User Queries & Responses  

### Q: What is the main topic of this page?  
### A:  
```md  
The webpage discusses **[Main Topic]**, focusing on:  
- **Key aspect 1**  
- **Key aspect 2**  
- **Key aspect 3**  

For full details, refer to the original content."##;

pub struct WebScrapeAgent {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    app_state: Option<Arc<Mutex<AppState>>>,
    models: Vec<OllamaModel>,
    active_model: Option<OllamaModel>,
    history: Vec<ChatMessage>,
    coordinator: Option<
        Arc<
            tokio::sync::Mutex<Coordinator<Vec<ChatMessage>, (DDGSearcher, (Scraper, Calculator))>>,
        >,
    >,
    sys_msg: String,
}

impl WebScrapeAgent {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            app_state: None,
            models: vec![],
            active_model: None,
            history: vec![],
            coordinator: None,
            sys_msg: SYS_MSG.to_string(),
        }
    }

    fn get_coordinator(
        &mut self,
        active_model: OllamaModel,
        history: Vec<ChatMessage>,
        app_state: Option<Arc<Mutex<AppState>>>,
        action_tx: Option<UnboundedSender<BroadcastMsg>>,
    ) -> Arc<tokio::sync::Mutex<Coordinator<Vec<ChatMessage>, (DDGSearcher, (Scraper, Calculator))>>>
    {
        let (url, port) = self.get_ollama_url(app_state);

        let ollama = Ollama::new(url, port);
        let model = active_model.name.clone();
        let tools = (DDGSearcher::new(), (Scraper {}, Calculator {}));
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
            let sys_msg = self.sys_msg.clone();

            tokio::spawn(async move {
                let _ = Self::send_chat_msg(action_tx, coordinator, msg.clone(), sys_msg).await;
            });
        }
    }

    async fn send_chat_msg(
        action_tx: Option<UnboundedSender<BroadcastMsg>>,
        coordinator: Arc<
            tokio::sync::Mutex<Coordinator<Vec<ChatMessage>, (DDGSearcher, (Scraper, Calculator))>>,
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
            let _ = tx.send(BroadcastMsg::GetChatReponse(resp.message.clone()));
        }
        println!("{:?} CHAT RESPONSE", resp);
        Ok(())
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

    fn advanced_ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("advanced options:")
            .default_open(false)
            .show(ui, |ui| {
                ui.small("system message:");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.sys_msg)
                            .return_key(KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::Enter))
                            .desired_rows(2)
                            .hint_text("Type here..")
                            .margin(Margin::symmetric(4.0, 4.0)),
                    );
                    if ui.button("save").clicked() {
                        println!("save system message: {}", self.sys_msg.clone());
                    }
                });
            });
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
                    egui::ComboBox::from_id_salt(Id::new(format!(
                        "{}_combo_{}",
                        m.name,
                        self.name()
                    )))
                    .width(100.0)
                    .truncate()
                    .selected_text(m.name)
                    .show_ui(ui, |ui| {
                        for om in self.models.iter() {
                            ui.selectable_value(&mut self.active_model, Some(om.clone()), &om.name);
                        }
                    });

                    if self.active_model != previous_value {
                        self.change_active_model();
                    }
                }
                ui.end_row();

                // --------
                ui.small("");
                self.advanced_ui(ui);
                ui.end_row();
            });
    }
}

impl Agent for WebScrapeAgent {
    fn name(&self) -> &'static str {
        "Web Text Scraper"
    }

    fn description(&self) -> &'static str {
        "Simple Web text scraper. Parsing HTML into a markdown and work with text from it."
    }

    fn agent(&self) -> AgentEnum {
        AgentEnum::WebScrape
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

impl Component for WebScrapeAgent {
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
                self.msg_to_coordinator(msg);
            }
            _ => {}
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let resp = ui
            .scope_builder(
                UiBuilder::new()
                    .id_salt("chat_agent_component")
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
