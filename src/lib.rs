#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod app_state;
mod components;
mod config;
mod enums;
mod ollama_state;
mod tools;
mod utils;

pub use app::DeskApp;
