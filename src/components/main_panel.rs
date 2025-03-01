use super::Component;
use crate::{
    app_state::{self, AppState},
    enums::{BroadcastMsg, DirectoryFiles, ImageFileItems},
};
use egui::{CollapsingHeader, Color32, ScrollArea, Sense, Vec2};
use std::sync::{Arc, Mutex};
use thumbnails::Thumbnailer;
use tokio::sync::mpsc::UnboundedSender;

pub struct MainPanel {
    action_tx: Option<UnboundedSender<BroadcastMsg>>,
    app_state: Option<Arc<Mutex<AppState>>>,
    dir_files: Vec<ImageFileItems>,
}

impl MainPanel {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            app_state: None,
            dir_files: vec![],
        }
    }

    fn show_images(&mut self) {
        // let mut dir_files = vec![];
        // {
        //     if let Some(app_state) = self.app_state.clone() {
        //         dir_files = app_state.lock().unwrap().dir_files.clone();
        //     }
        // }
        //
        // let thumbnailer = Thumbnailer::new(160, 160);
        //
        // for dir in dir_files.iter() {
        //     let mut thumbnails = vec![];
        //     for file in dir.files.iter() {
        //         let thumb = thumbnailer.get(file).unwrap();
        //         // thumbnails.push(thumb.clone());
        //
        //         let img_buf = thumb.to_rgb8();
        //         let img = egui::ColorImage::from_rgb([160, 160], &img_buf);
        //
        //         // let ui_img =
        //         //     egui::Image::from_bytes(format!("bytes://{}", file), img.clone().as_raw());
        //         // thumbnails.push(ui_img);
        //     }
        //
        //     self.dir_files.push(ImageFileItems {
        //         dir: dir.dir.clone(),
        //         images: thumbnails,
        //     });
        // }
    }

    // fn render_dir_images(&mut self, dir: DirectoryFiles, ui: &mut egui::Ui) {
    fn render_dir_images(&mut self, dir: ImageFileItems, ui: &mut egui::Ui) {
        // println!("{:?} - rendering dir", dir);

        CollapsingHeader::new(dir.dir).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for image in dir.images {
                    // let img = image.to_rgb8();

                    // let resp = ui.add(
                    //     // egui::Image::new(format!("file://{}", image))
                    //     egui::ColorImage::from_rgb([160, 160], &image.as_bytes()), // .fit_to_exact_size(Vec2::new(160.0, 160.0))
                    //                                                                // .bg_fill(Color32::from_rgb(33, 33, 33))
                    //                                                                // .max_width(440.0)
                    //                                                                // .sense(Sense::click())
                    //                                                                // .rounding(6.0),
                    // );
                    // if resp.clicked() {
                    //     println!("open files {}", image);
                    //     let _ = open::that(image);
                    // }

                    // if let Ok(img) = image

                    // let resp = ui.add(
                    //     egui::Image::from(image)
                    //         .fit_to_exact_size(Vec2::new(120.0, 120.0))
                    //         .bg_fill(Color32::from_rgb(33, 33, 33))
                    //         // .max_width(440.0)
                    //         .sense(Sense::click())
                    //         .rounding(6.0),
                    // );
                    // if resp.clicked() {
                    //     println!("open files {}", image);
                    //     let _ = open::that(image);
                    // }
                }
            });
        });

        // // -- images
        // if let Some(images_struct) = item_message.clone().images {
        //     ui.horizontal_wrapped(|ui| {
        //         for image in images_struct.images {
        //             let resp = ui.add(
        //                 egui::Image::new(format!("file://{}", image.path))
        //                     .fit_to_exact_size(Vec2::new(120.0, 120.0))
        //                     .bg_fill(Color32::from_rgb(33, 33, 33))
        //                     // .max_width(440.0)
        //                     .sense(Sense::click())
        //                     .rounding(6.0),
        //             );
        //             if resp.clicked() {
        //                 println!("open files {}", image.path);
        //                 let _ = open::that(image.path);
        //             }
        //         }
        //     });
        // }
    }
}

impl Component for MainPanel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn register_tx(&mut self, action_tx: UnboundedSender<BroadcastMsg>) {
        self.action_tx = Some(action_tx);
    }

    fn update(&mut self, msg: BroadcastMsg) {
        // --
        if let BroadcastMsg::ShowImages = msg {
            self.show_images();
        }
    }

    fn register_app_state(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = Some(app_state);
    }

    fn render(&mut self, ctx: &egui::Context) {
        // ctx.request_repaint_after_secs(1.0);
        //

        // let mut dir_files = vec![];
        // {
        //     if let Some(app_state) = self.app_state.clone() {
        //         dir_files = app_state.lock().unwrap().dir_files.clone();
        //     }
        // }

        let dir_files = egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                egui::Frame::default().show(ui, |ui| {
                    ScrollArea::vertical()
                        .animated(false)
                        .auto_shrink([false, false])
                        // .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for dir in self.dir_files.iter() {
                                // self.render_dir_images(dir.clone(), ui);
                            }
                            // self.messages.ui(ui);
                        });
                });
            });
        });
    }
}
