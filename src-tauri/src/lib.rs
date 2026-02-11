// Motrix - A full-featured download manager
// Built with Tauri + Vue 3

pub mod aria2;
pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod power;
pub mod tray;

pub use error::{Error, Result};
