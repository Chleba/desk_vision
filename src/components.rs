use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{app_state::AppState, enums::BroadcastMsg};

pub mod agents_panel;
pub mod bottom_panel;
pub mod chat_input;
pub mod main_panel;
pub mod messages;
pub mod ollama_settings;
pub mod top_menu;
pub mod top_panel;

pub trait Component: Any {
    fn init(&mut self) {}

    #[allow(dead_code)]
    fn as_any(&self) -> &dyn Any;

    #[allow(unused_variables)]
    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {}

    #[allow(unused_variables)]
    fn register_app_state(&mut self, app_state: Arc<Mutex<AppState>>) {}

    #[allow(unused_variables)]
    fn render(&mut self, ctx: &egui::Context) {}

    #[allow(unused_variables)]
    fn ui(&mut self, ui: &mut egui::Ui) {}

    #[allow(unused_variables)]
    fn update(&mut self, msg: BroadcastMsg) {}
}
