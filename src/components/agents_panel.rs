use super::Component;
use crate::{
    agents::{chat::ChatAgent, images::ImageAgent, web_scrape::WebScrapeAgent, AgentComponent},
    enums::BroadcastMsg,
};
use tokio::sync::mpsc::UnboundedSender;

pub struct AgentPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    agents: Vec<Box<dyn AgentComponent>>,
}

impl AgentPanel {
    pub fn new() -> Self {
        let chat_agent = ChatAgent::new();
        let websearch_agent = WebScrapeAgent::new();
        let images_agent = ImageAgent::new();

        Self {
            action_tx: None,
            agents: vec![
                Box::new(chat_agent),
                Box::new(websearch_agent),
                Box::new(images_agent),
            ],
        }
    }
}

impl Component for AgentPanel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn init(&mut self) {
        for agent in self.agents.iter_mut() {
            agent.init();
        }
    }

    fn register_app_state(
        &mut self,
        app_state: std::sync::Arc<std::sync::Mutex<crate::app_state::AppState>>,
    ) {
        for agent in self.agents.iter_mut() {
            agent.register_app_state(app_state.clone());
        }
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        for agent in self.agents.iter_mut() {
            agent.register_tx(action_tx.clone());
        }
        self.action_tx = Some(action_tx);
    }

    fn update(&mut self, msg: BroadcastMsg) {
        for agent in self.agents.iter_mut() {
            agent.update(msg.clone());
        }
    }

    fn render(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("agent_panel")
            // .default_width(100.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for agent in self.agents.iter_mut() {
                        agent.ui(ui);
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::warn_if_debug_build(ui);
                });
            });
    }
}
