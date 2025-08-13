use crate::import_export::{Playlist, get_file_name, get_file_stem};
use crate::{get_path_from_config, get_retro_arch_config};

use std::env::home_dir;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use zip::ZipArchive;

pub(crate) fn import(
    origin: &PathBuf,
    destination: Option<PathBuf>,
    retro_arch_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    // Read in import archive
    let file = File::open(origin).ok().expect("Could not read import file");
    let mut archive = ZipArchive::new(BufReader::new(file))
        .ok()
        .expect("Could not read import file");

    let mut playlist: String = String::new();
    let mut game: String = String::new();
    let mut parsed_playlist: Playlist = Playlist::default();
    let mut new_playlist: Playlist;
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

    // Get RetroArch config and load the necessary paths from it
    let (config, retro_arch_path) = get_retro_arch_config(retro_arch_path)?;

    let playlist_directory = get_path_from_config(&config, "playlist_directory", &retro_arch_path)?;
    let thumbnails_directory =
        get_path_from_config(&config, "thumbnails_directory", &retro_arch_path)?;

    let playlist_file_path = playlist_directory.join(format!("{}.lpl", playlist));

    // If playlist exists add imported entry to it
    if Path::new(&playlist_file_path).exists() {
        let playlist_file = File::open(playlist_file_path.to_owned())?;
        let reader = BufReader::new(playlist_file);
        let existing_playlist: Playlist = serde_json::from_reader(reader)?;

        new_playlist = existing_playlist.clone();
        new_playlist.items = existing_playlist
            .items
            .iter()
            .filter(|item| item.label != game.to_owned())
            .cloned()
            .collect();
    }
    // If it doesn't create a new playlist
    else {
        new_playlist = parsed_playlist.clone();
        new_playlist.items = vec![];

        let destination = destination
            .or_else(|| Some(home_dir().unwrap().join("Roms")))
            .unwrap();

        new_playlist.scan_content_dir = destination.join(&playlist).to_str().unwrap().to_owned();
    }

    // Add imported game to playlist and write to disk
    let mut new_item = parsed_playlist.items.first().unwrap().clone();
    let rom_file_path = PathBuf::from(new_playlist.scan_content_dir.to_owned())
        .join(get_file_name(new_item.path.as_str()).unwrap());

    new_item.path = rom_file_path.to_str().unwrap().to_owned();
    new_playlist.items.push(new_item);

    let mut new_playlist_file = File::create(&playlist_file_path)?;
    new_playlist_file.write_all(
        serde_json::to_string_pretty(&new_playlist)
            .unwrap()
            .as_bytes(),
    )?;

    // Write game rom (and thumbnails if present) to disk
    let mut files = vec![(&rom_file_buf, rom_file_path.to_str().unwrap())];

    let boxart_file_path = thumbnails_directory
        .join(&playlist)
        .join("Named_Boxarts")
        .join(format!("{}.png", game));
    let snap_file_path = thumbnails_directory
        .join(&playlist)
        .join("Named_Snaps")
        .join(format!("{}.png", game));
    let title_file_path = thumbnails_directory
        .join(&playlist)
        .join("Named_Titles")
        .join(format!("{}.png", game));

    if !boxart_file_buf.is_empty() {
        files.push((&boxart_file_buf, boxart_file_path.to_str().unwrap()));
    }
    if !snap_file_buf.is_empty() {
        files.push((&snap_file_buf, snap_file_path.to_str().unwrap()));
    }
    if !title_file_buf.is_empty() {
        files.push((&title_file_buf, title_file_path.to_str().unwrap()));
    }

    write_files_to_disk(&*files)?;

    Ok(())
}

fn write_files_to_disk(files: &[(&Vec<u8>, &str)]) -> anyhow::Result<()> {
    // Calculate total bytes to write across all files
    let total_bytes: u64 = files.iter().map(|(data, _)| data.len() as u64).sum();

    // Create single progress bar for all files
    let progress_bar = ProgressBar::new(total_bytes);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-"),
    );
    progress_bar.set_message("Importing game...");

    let mut total_written = 0u64;

    // Write each file
    for (_file_idx, (data, path)) in files.iter().enumerate() {
        let dir = Path::new(path)
            .parent()
            .ok_or_else(|| anyhow::anyhow!("no parent directory"))?;

        // Create every missing directory in the chain.
        fs::create_dir_all(dir)?;

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write this file's data in chunks
        for chunk in data.chunks(8192) {
            writer.write_all(chunk)?;
            total_written += chunk.len() as u64;
            progress_bar.set_position(total_written);
        }

        writer.flush()?;
    }

    progress_bar.finish();

    Ok(())
}
