use crate::{get_path_from_config, get_retro_arch_config};

use std::env::consts;
use std::fs::{File, remove_file};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use sevenz_rust::{Password, SevenZReader};
use zip::ZipArchive;

pub(crate) async fn update_cores(version: String, retro_arch_path: Option<PathBuf>) -> Result<()> {
    // Get RetroArch config and load the necessary paths from it
    let (config, retro_arch_path) = get_retro_arch_config(retro_arch_path)?;

    let core_path = get_path_from_config(&config, "libretro_directory", &retro_arch_path)?;
    let info_path = get_path_from_config(&config, "libretro_info_path", &retro_arch_path)?;

    // Build download URL for RetroArch cores and download and extract them
    let release_type = if version != "nightly" {
        format!("stable/{}", version)
    } else {
        version
    };

    let core_download_url = format!(
        "http://buildbot.libretro.com/{}/{}/{}/RetroArch_cores.7z",
        release_type,
        consts::OS,
        consts::ARCH
    );
    let core_download_file_path = core_path.join("cores.7z");

    download_file(
        &Client::new(),
        &core_download_url,
        &core_download_file_path,
        "Downloading cores...",
    )
    .await?;

    extract_7zip_file(&core_download_file_path, &core_path, "Extracting cores...")?;

    remove_file(core_download_file_path)?;

    // Build download URL for RetroArch info files and download and extract them
    let info_download_url = "https://buildbot.libretro.com/assets/frontend/info.zip";
    let info_download_file_path = info_path.join("info.zip");

    download_file(
        &Client::new(),
        &info_download_url,
        &info_download_file_path,
        "Downloading info files...",
    )
    .await?;

    extract_zip_file(
        &info_download_file_path,
        &info_path,
        "Extracting info files...",
    )?;

    remove_file(info_download_file_path)?;

    println!("Cores successfully updated.");
    Ok(())
}

async fn download_file(
    client: &Client,
    url: &str,
    path: &PathBuf,
    message: &'static str,
) -> Result<()> {
    // Reqwest setup
    let response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    // Indicatif setup
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_message(message);

    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));

    // Download file
    let mut file = File::create(path)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;

        downloaded += chunk.len() as u64;
        progress_bar.set_position(downloaded);
    }

    progress_bar.finish();
    Ok(())
}

fn extract_zip_file(file: &PathBuf, destination: &PathBuf, message: &'static str) -> Result<()> {
    // Zip setup
    let zip_file = File::open(file)?;
    let mut archive = ZipArchive::new(zip_file)?;

    let mut total_size = 0;
    for index in 0..archive.len() {
        let file = archive.by_index(index)?;

        if file.is_file() {
            total_size += file.size();
        }
    }

    // Indicatif setup
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_message(message);

    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));

    let mut decompressed_size: u64 = 0;

    // Extract archive
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;

        if !file.is_file() {
            continue;
        }

        let mut buffer = [0u8; 1024];
        let path = destination.join(file.name());

        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut extracted_file = File::create(path)?;

        loop {
            let read_size = file.read(&mut buffer)?;

            if read_size == 0 {
                break;
            }

            extracted_file.write_all(&buffer[..read_size])?;

            decompressed_size += read_size as u64;
            progress_bar.set_position(decompressed_size);
        }
    }

    progress_bar.finish();
    Ok(())
}

fn extract_7zip_file(file: &PathBuf, destination: &PathBuf, message: &'static str) -> Result<()> {
    // SevenZ setup
    let mut sz = SevenZReader::open(file, Password::empty())?;

    let total_size: u64 = sz
        .archive()
        .files
        .iter()
        .filter(|e| e.has_stream())
        .map(|e| e.size())
        .sum();

    // Indicatif setup
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_message(message);

    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));

    let mut decompressed_size: u64 = 0;

    // Extract archive
    sz.for_each_entries(|entry, reader| {
        if !entry.has_stream {
            return Ok(true);
        }

        let mut buffer = [0u8; 1024];
        let path = destination.join(Path::new(entry.name()).file_name().unwrap());

        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut file = File::create(path)?;

        loop {
            let read_size = reader.read(&mut buffer)?;

            if read_size == 0 {
                break Ok(true);
            }

            file.write_all(&buffer[..read_size])?;

            decompressed_size += read_size as u64;
            progress_bar.set_position(decompressed_size);
        }
    })?;

    progress_bar.finish();
    Ok(())
}
