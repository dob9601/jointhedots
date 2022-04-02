mod dotfile;
mod manifest;
mod metadata;
mod config;

pub use dotfile::Dotfile;
pub use manifest::Manifest;
pub use config::Config;

pub use metadata::{AggregatedDotfileMetadata, DotfileMetadata};
