use crate::import_export::{Playlist, get_file_name};
use crate::{get_path_from_config, get_retro_arch_config};

use std::fs::File;
use std::io::{BufReader, Read, Seek, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use tempfile::tempdir;
use zip::write::{FileOptions, ZipWriter};

pub(crate) fn export(
    playlist: &String,
    game: &String,
    destination: &PathBuf,
    retro_arch_path: Option<PathBuf>,
) -> Result<()> {
    // Get RetroArch config and load the necessary paths from it
    let (config, retro_arch_path) = get_retro_arch_config(retro_arch_path)?;

    let playlist_directory = get_path_from_config(&config, "playlist_directory", &retro_arch_path)?;
    let thumbnails_directory =
        get_path_from_config(&config, "thumbnails_directory", &retro_arch_path)?;

    let playlist_file_path = playlist_directory.join(format!("{}.lpl", playlist));

    // Parse playlist
    let playlist_file = File::open(playlist_file_path)?;
    let reader = BufReader::new(playlist_file);
    let parsed_playlist: Playlist = serde_json::from_reader(reader)?;

    let playlist_item = parsed_playlist
        .items
        .iter()
        .find(|item| item.label == game.to_owned())
        .expect("Specified game not found in playlist");

    // Find thumbnail files
    let boxart_file_path = thumbnails_directory
        .join(playlist)
        .join("Named_Boxarts")
        .join(format!("{}.png", game));
    let snap_file_path = thumbnails_directory
        .join(playlist)
        .join("Named_Snaps")
        .join(format!("{}.png", game));
    let title_file_path = thumbnails_directory
        .join(playlist)
        .join("Named_Titles")
        .join(format!("{}.png", game));

    // Build new playlist
    let mut new_playlist = parsed_playlist.clone();
    new_playlist.items = parsed_playlist
        .items
        .iter()
        .filter(|item| item.label == game.to_owned())
        .cloned()
        .collect();

    let temp_dir = tempdir()?;
    let new_playlist_file_path = temp_dir.path().join(format!("{}.lpl", playlist));
    let mut new_playlist_file = File::create(&new_playlist_file_path)?;
    new_playlist_file.write_all(
        serde_json::to_string_pretty(&new_playlist)
            .unwrap()
            .as_bytes(),
    )?;

    // Create all files needed for export
    let temp_playlist_file = File::open(new_playlist_file_path)?;
    let rom_file = File::open(playlist_item.path.to_owned())?;
    let boxart_file: File;
    let snap_file: File;
    let title_file: File;

    // Build the zip file and write to disk
    let mut files_and_paths = vec![
        (&temp_playlist_file, format!("playlists/{}.lpl", playlist)),
        (
            &rom_file,
            format!(
                "roms/{}/{}",
                playlist,
                get_file_name(playlist_item.path.as_str()).unwrap()
            ),
        ),
    ];

    if Path::new(&boxart_file_path).exists() {
        boxart_file = File::open(boxart_file_path)?;
        files_and_paths.push((
            &boxart_file,
            format!("thumbnails/{}/Named_Boxarts/{}.png", playlist, game),
        ));
    }

    if Path::new(&snap_file_path).exists() {
        snap_file = File::open(snap_file_path)?;
        files_and_paths.push((
            &snap_file,
            format!("thumbnails/{}/Named_Snaps/{}.png", playlist, game),
        ));
    }

    if Path::new(&title_file_path).exists() {
        title_file = File::open(title_file_path)?;
        files_and_paths.push((
            &title_file,
            format!("thumbnails/{}/Named_Titles/{}.png", playlist, game),
        ));
    }

    write_files_to_zip(&files_and_paths, destination)?;

    Ok(())
}

pub fn write_files_to_zip(files: &[(&File, String)], zip_path: &Path) -> Result<()> {
    let zip_file = File::create(zip_path)
        .with_context(|| format!("Failed to create zip archive at {:?}", zip_path))?;
    let mut zip = ZipWriter::new(zip_file);

    // Calculate total size for progress bar
    let total_size: u64 = files
        .iter()
        .map(|(file, _)| {
            file.metadata()
                .map(|metadata| metadata.len())
                .with_context(|| "Failed to get file metadata")
        })
        .collect::<Result<Vec<u64>>>()?
        .iter()
        .sum();

    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));
    progress_bar.set_message("Exporting game...");

    for (file, target_path) in files {
        // Rewind the file to start
        let mut file = file
            .try_clone()
            .with_context(|| format!("Failed to clone file for {:?}", target_path))?;
        file.seek(std::io::SeekFrom::Start(0))
            .with_context(|| format!("Failed to seek file {:?}", target_path))?;

        zip.start_file::<String, ()>(target_path.to_string(), FileOptions::default())
            .with_context(|| format!("Failed to start file {:?} in zip", target_path))?;

        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = file
                .read(&mut buffer)
                .with_context(|| format!("Failed to read from file {:?}", target_path))?;
            if bytes_read == 0 {
                break;
            }
            zip.write_all(&buffer[..bytes_read])
                .with_context(|| format!("Failed to write to zip for file {:?}", target_path))?;
            progress_bar.inc(bytes_read as u64);
        }
    }

    zip.finish()
        .with_context(|| "Failed to finish writing zip archive")?;
    progress_bar.finish();

    Ok(())
}
