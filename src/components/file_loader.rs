use std::path::PathBuf;

use egui::TextureOptions;
use image::ImageReader;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    app_state::AppState,
    enums::{BroadcastMsg, DirectoryFiles, DirectoryImage, DirectoryImages, FileWithLabel},
    utils::search_images_at_path,
};
use std::sync::{Arc, Mutex};

pub struct FileLoader {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    app_state: Option<Arc<Mutex<AppState>>>,
}

impl FileLoader {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            app_state: None,
        }
    }

    fn init_thumbnails(&mut self, cc: &eframe::CreationContext<'_>) {
        let mut dir_files = vec![];
        {
            if let Some(app_state) = self.app_state.clone() {
                dir_files = app_state.lock().unwrap().dir_files.clone();
            }
        }

        for dir in dir_files.iter() {
            self.create_thumbnails(dir, cc.egui_ctx.clone());
        }
    }

    fn create_thumbnails(&mut self, dir_files: &DirectoryFiles, ctx: egui::Context) {
        let path = PathBuf::from(dir_files.dir.clone());
        let files = dir_files.files_with_labels.clone();

        let mut dir_imgs = vec![];
        for file in files.iter() {
            let thumb = ImageReader::open(file.file.clone())
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap()
                .thumbnail(160, 160);

            println!("{} - {} = img w,h", thumb.width(), thumb.height());

            let rgba = thumb.to_rgba8();
            let img = egui::ColorImage::from_rgba_unmultiplied(
                [thumb.width() as usize, thumb.height() as usize],
                rgba.as_raw(),
            );

            let texture = ctx.load_texture(file.file.to_string(), img, TextureOptions::default());
            let dir_img = DirectoryImage {
                file: file.file.to_string(),
                texture,
            };

            dir_imgs.push(dir_img);
        }

        let dir_obj = DirectoryImages {
            dir: path,
            images: dir_imgs,
        };

        if let Some(action_tx) = self.action_tx.clone() {
            let _ = action_tx.send(BroadcastMsg::DirectoryImages(dir_obj));
        }
    }

    fn search_images_on_path(&mut self, path: PathBuf, ctx: &egui::Context) {
        let files = search_images_at_path(path.clone());
        if let Some(action_tx) = self.action_tx.clone() {
            let _ = action_tx.send(BroadcastMsg::DirectoryFiles(path.clone(), files.clone()));
        }

        let mut d_files = vec![];
        for f in files.iter() {
            d_files.push(FileWithLabel {
                file: f.to_string(),
                labels: vec![],
            });
        }

        self.create_thumbnails(
            &DirectoryFiles {
                dir: path.to_string_lossy().to_string(),
                files_with_labels: d_files,
                // files,
                // files_with_labels: vec![],
            },
            ctx.clone(),
        );
    }
}

impl Component for FileLoader {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn init(&mut self, cc: &eframe::CreationContext<'_>) {
        self.init_thumbnails(cc);
    }

    fn update_ctx(&mut self, msg: BroadcastMsg, ctx: &egui::Context) {
        if let BroadcastMsg::PickedDirectory(path) = msg {
            self.search_images_on_path(path, ctx);
        }
    }

    fn register_app_state(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = Some(app_state);
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }
}
