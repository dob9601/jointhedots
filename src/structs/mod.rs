mod config;
mod dotfile;
mod manifest;
mod metadata;

pub use config::Config;
pub use dotfile::Dotfile;
pub use manifest::Manifest;

pub use metadata::{AggregatedDotfileMetadata, DotfileMetadata};
