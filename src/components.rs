use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{app_state::AppState, enums::BroadcastMsg};

pub mod file_loader;
pub mod main_panel;
pub mod ollama_settings;
pub mod top_menu;
pub mod top_panel;

pub trait Component: Any {
    #[allow(unused_variables)]
    fn init(&mut self, cc: &eframe::CreationContext<'_>) {}

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

    #[allow(unused_variables)]
    fn update_ctx(&mut self, msg: BroadcastMsg, ctx: &egui::Context) {}
}
