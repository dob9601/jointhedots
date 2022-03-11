mod manifest;
mod dotfile;
mod metadata;

pub use manifest::Manifest;
pub use dotfile::Dotfile;

pub use metadata::{AggregatedDotfileMetadata, DotfileMetadata};
