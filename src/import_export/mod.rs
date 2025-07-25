pub mod export;
pub mod import;

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Playlist {
    pub version: String,
    pub default_core_path: String,
    pub default_core_name: String,
    pub label_display_mode: u32,
    pub right_thumbnail_mode: u32,
    pub left_thumbnail_mode: u32,
    pub thumbnail_match_mode: u32,
    pub sort_mode: u32,
    pub scan_content_dir: String,
    pub scan_file_exts: String,
    pub scan_dat_file_path: String,
    pub scan_search_recursively: bool,
    pub scan_search_archives: bool,
    pub scan_filter_dat_content: bool,
    pub scan_overwrite_playlist: bool,
    pub items: Vec<PlaylistItem>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlaylistItem {
    pub path: String,
    pub label: String,
    pub core_path: String,
    pub core_name: String,
    pub crc32: String,
    pub db_name: String,
}

fn get_file_name(path_str: &str) -> Option<&str> {
    Path::new(path_str)
        .file_name()
        .and_then(|os_str| os_str.to_str())
}

fn get_file_stem(path: &str) -> Option<&str> {
    Path::new(path).file_stem().and_then(|stem| stem.to_str())
}
