use std::path::PathBuf;

use egui::{
    epaint::{tessellator::path, textures},
    TextureOptions,
};
use thumbnails::Thumbnailer;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    config::SUPPORTED_IMAGE_FORMATS,
    enums::{BroadcastMsg, DirectoryImage, DirectoryImages},
    utils::search_images_at_path,
};

pub struct FileLoader {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
}

impl FileLoader {
    pub fn new() -> Self {
        Self { action_tx: None }
    }

    fn search_images_on_path(&mut self, path: PathBuf, ctx: &egui::Context) {
        let files = search_images_at_path(path.clone());

        let thumbnailer = Thumbnailer::new(160, 160);

        let mut dir_imgs = vec![];
        for file in files.iter() {
            let thumb = thumbnailer.get(file).unwrap();
            // thumbnails.push(thumb.clone());

            let img_buf = thumb.to_rgb8();
            let img = egui::ColorImage::from_rgb([160, 160], &img_buf);
            let texture = ctx.load_texture(file, img, TextureOptions::default());

            let dir_img = DirectoryImage {
                file: file.to_string(),
                texture,
            };

            dir_imgs.push(dir_img);
        }

        let dir_obj = DirectoryImages {
            dir: path,
            images: dir_imgs,
        };

        if let Some(action_tx) = self.action_tx.clone() {
            // let _ = action_tx.send(BroadcastMsg::DirectoryFiles(path, files));
            let _ = action_tx.send(BroadcastMsg::DirectoryImages(dir_obj));
        }
    }
}

impl Component for FileLoader {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update_ctx(&mut self, msg: BroadcastMsg, ctx: &egui::Context) {
        match msg {
            BroadcastMsg::PickedDirectory(path) => {
                self.search_images_on_path(path, ctx);
            }
            BroadcastMsg::OllamaModels(models) => {
                // self.models = models;
            }
            _ => {}
        }
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }
}
