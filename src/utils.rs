use base64::Engine;
use egui::Ui;
use ollama_rs::generation::images::Image;
use rust_search_fork::FilterExt;
use rust_search_fork::SearchBuilder;
use std::cmp;
use std::fs;
use std::future::Future;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use crate::config::SUPPORTED_IMAGE_FORMATS;
use crate::enums::ImageBase64Search;
use crate::enums::ImageStructured;
pub mod easing {
    pub use simple_easing::*;
}

type Easing = fn(f32) -> f32;

pub fn spawn(f: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(f);
}

pub fn sleep(d: Duration) -> impl Future<Output = ()> {
    tokio::time::sleep(d)
}

pub fn bytes_convert(num: f64) -> String {
    let num = num.abs();
    let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    if num < 1_f64 {
        return format!("{}{}", num, "B");
    }
    let delimiter = 1000_f64;
    let exponent = cmp::min(
        (num.ln() / delimiter.ln()).floor() as i32,
        (units.len() - 1) as i32,
    );
    let pretty_bytes = format!("{:.2}", num / delimiter.powi(exponent))
        .parse::<f64>()
        .unwrap()
        * 1_f64;
    let unit = units[exponent as usize];
    format!("{}{}", pretty_bytes, unit)
}

pub fn animate_repeating(ui: &mut Ui, easing: Easing, duration: Duration, offset: f32) -> f32 {
    ui.ctx().request_repaint();

    let t = ui.input(|i| i.time as f32 + offset);
    let x = t % duration.as_secs_f32();
    easing(x / duration.as_secs_f32())
}

pub fn animate_continuous(ui: &mut Ui, easing: Easing, duration: Duration, offset: f32) -> f32 {
    let t = animate_repeating(ui, easing::linear, duration, offset);
    easing::roundtrip(easing(t))
}

pub fn img_paths_to_base64(images: Vec<ImageStructured>) -> Vec<ImageBase64Search> {
    let mut base64_imgs = vec![];
    for img in images {
        let path = Path::new(&img.path);
        let bytes = fs::read(path);
        if let Ok(img_bytes) = bytes {
            let b64_img = base64::engine::general_purpose::STANDARD.encode(&img_bytes);
            base64_imgs.push(ImageBase64Search {
                base64: Image::from_base64(b64_img),
                path: img.path.to_string(),
            });
        }
    }
    base64_imgs
}

pub fn search_images_at_path(path: PathBuf) -> Vec<String> {
    let p = path.to_string_lossy().to_string();
    let search: Vec<String> = SearchBuilder::default()
        .location(&p)
        .dirs(false)
        .custom_filter(|entry| {
            let e = entry.metadata().unwrap();
            // println!("{:?} - maslo entry on path", e);
            if e.is_file() && !e.is_dir() {
                let path = entry.path();
                // println!("{:?} - path", path.to_string_lossy());
                if let Some(ext) = path.extension() {
                    // println!("{:?} = ext", ext);
                    if let Some(e) = ext.to_str() {
                        return SUPPORTED_IMAGE_FORMATS.contains(&e);
                    }
                }
            }
            false
        })
        .build()
        .collect();
    search
}
