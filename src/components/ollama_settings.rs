use std::time::Duration;

use egui::{popup_below_widget, CollapsingHeader, Id, PopupCloseBehavior, TextEdit, Vec2};
use egui_flex::{Flex, FlexAlignContent, FlexItem};
use egui_form::{
    garde::{field_path, GardeReport},
    Form, FormField,
};
use garde::Validate;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    enums::{BroadcastMsg, OllamaModel},
    utils::{bytes_convert, sleep, spawn},
};

use super::Component;

#[derive(Debug, Validate)]
struct OllamaURL {
    #[garde(url)]
    url: String,
}

pub struct OllamaSettings {
    url: OllamaURL,
    models: Vec<OllamaModel>,
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
}

impl OllamaSettings {
    pub fn new() -> Self {
        Self {
            url: OllamaURL { url: String::new() },
            models: vec![],
            action_tx: None,
        }
    }

    async fn period_ollama_status(action_tx: Option<UnboundedSender<BroadcastMsg>>) {
        loop {
            sleep(Duration::from_secs(1)).await;

            if let Some(tx) = action_tx.clone() {
                let _ = tx.send(BroadcastMsg::GetOllamaRunning);
            }
        }
    }
}

impl Component for OllamaSettings {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn init(&mut self) {
        spawn(Self::period_ollama_status(self.action_tx.clone()));
    }

    fn update(&mut self, msg: BroadcastMsg) {
        match msg {
            BroadcastMsg::OllamaURL(url) => {
                self.url.url = url;
            }
            BroadcastMsg::OllamaModels(models) => {
                self.models = models;
            }
            _ => {}
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let lamma_img = egui::Image::new(egui::include_image!("../../assets/ollama.png"));

        let button = egui::Button::image_and_text(lamma_img, "Ollama");
        let button_id = Id::new("ollama_popup");
        let button_res = ui.add(button);

        let action_tx = self.action_tx.clone();
        if button_res.clicked() {
            ui.memory_mut(|mem| {
                if let Some(tx) = action_tx {
                    let _ = tx.send(BroadcastMsg::GetOllamaURL);
                    let _ = tx.send(BroadcastMsg::GetOllamaModels);
                }
                mem.toggle_popup(button_id);
            });
        }

        popup_below_widget(
            ui,
            button_id,
            &button_res,
            PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                ui.set_width(310.0);

                let mut form = Form::new().add_report(GardeReport::new(self.url.validate()));

                Flex::horizontal()
                    .align_content(FlexAlignContent::Stretch)
                    .w_full()
                    .show(ui, |flex| {
                        flex.add_ui(FlexItem::new().grow(1.0), |ui| {
                            FormField::new(&mut form, field_path!("url"))
                                .label("Ollama URL:")
                                .ui(ui, TextEdit::singleline(&mut self.url.url));
                        });

                        let action_tx = self.action_tx.clone();

                        flex.add_ui(FlexItem::new().grow(1.0), |ui| {
                            if let Some(Ok(())) = form.handle_submit(&ui.button("save"), ui) {
                                if let Some(tx) = action_tx {
                                    let _ =
                                        tx.send(BroadcastMsg::SetOllamaURL(self.url.url.clone()));
                                }
                            }
                        });
                    });

                ui.label("Models:");
                for model in &self.models {
                    CollapsingHeader::new(model.name.clone())
                        .default_open(false)
                        .show(ui, |ui| {
                            egui::Grid::new("model info:")
                                .num_columns(2)
                                .striped(true)
                                .spacing(Vec2 { x: 4.0, y: 0.0 })
                                .show(ui, |ui| {
                                    ui.small("model:");
                                    ui.small(model.model.clone());
                                    ui.end_row();

                                    ui.small("size:");
                                    ui.small(bytes_convert(model.size as f64));
                                    ui.end_row();

                                    ui.small("parameters:");
                                    ui.small(model.details.parameter_size.clone());
                                    ui.end_row();

                                    ui.small("quantization:");
                                    ui.small(model.details.quantization_level.clone());
                                    ui.end_row();
                                });
                        });
                }
            },
        );
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }
}
