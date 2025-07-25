use crate::import_export::{get_file_stem, Playlist};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use zip::ZipArchive;

pub(crate) fn import(origin: &PathBuf, retro_arch_path: Option<PathBuf>) -> anyhow::Result<()> {
    // Read in import archive
    let file = File::open(origin).ok().expect("Could not read import file");
    let mut archive = ZipArchive::new(BufReader::new(file))
        .ok()
        .expect("Could not read import file");

    let mut playlist: String;
    let mut game: String;
    let mut parsed_playlist: Playlist;
    let mut rom_file_buf: Vec<u8> = Vec::new();
    let mut boxart_file_buf: Vec<u8> = Vec::new();
    let mut snap_file_buf: Vec<u8> = Vec::new();
    let mut title_file_buf: Vec<u8> = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).ok().unwrap();
        let name = entry.name().to_owned();

        // Check if the file is at root level and matches the extension
        if name.starts_with("playlists") {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).ok().unwrap();

            playlist = get_file_stem(name.as_str()).unwrap().to_string();

            parsed_playlist = serde_json::from_slice(&*buf)?;
            game = parsed_playlist.items.first().unwrap().label.to_owned();
        } else if name.starts_with("roms") {
            entry.read_to_end(&mut rom_file_buf).ok().unwrap();
        } else if name.starts_with("thumbnails") && name.contains("Named_Boxarts") {
            entry.read_to_end(&mut boxart_file_buf).ok().unwrap();
        } else if name.starts_with("thumbnails") && name.contains("Named_Snaps") {
            entry.read_to_end(&mut snap_file_buf).ok().unwrap();
        } else if name.starts_with("thumbnails") && name.contains("Named_Titles") {
            entry.read_to_end(&mut title_file_buf).ok().unwrap();
        }
    }

    //

    Ok(())
}
