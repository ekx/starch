mod import_export;
mod update_cores;

use crate::import_export::export::export;
use crate::import_export::import::import;
use crate::update_cores::update_cores;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use ini::Ini;
use steamlocate::SteamDir;

#[derive(Parser)]
#[command(version, about, propagate_version = false, subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Updates all cores (even non Steam cores)")]
    UpdateCores {
        #[arg(
            help = "Version of cores to download",
            default_value = "nightly",
            required = false
        )]
        version: String,

        #[arg(
            short,
            long,
            help = "Manually override RetroArch path (Will be queried from Steam otherwise)"
        )]
        retro_arch_path: Option<PathBuf>,
    },

    #[command(about = "Exports a game from a RetroArch playlist to a archive file")]
    Export {
        #[arg(help = "Playlist to export from", required = true)]
        playlist: String,

        #[arg(help = "Game to be exported", required = true)]
        game: String,

        #[arg(help = "Export destination path")]
        destination: PathBuf,

        #[arg(
            short,
            long,
            help = "Manually override RetroArch path (Will be queried from Steam otherwise)"
        )]
        retro_arch_path: Option<PathBuf>,
    },

    #[command(about = "Imports a game from a archive file to a RetroArch playlist")]
    Import {
        #[arg(help = "Import origin path", required = true)]
        origin: PathBuf,

        #[arg(
            help = "Game to be exported",
            required = false,
            help = "Where to put the imported game. Only gets used if playlist doesn't already exist. Roms folder in the home directory will be used if unset"
        )]
        destination: Option<PathBuf>,

        #[arg(
            short,
            long,
            help = "Manually override RetroArch path (Will be queried from Steam otherwise)"
        )]
        retro_arch_path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::UpdateCores {
            version,
            retro_arch_path,
        }) => {
            update_cores(version.to_owned(), retro_arch_path.to_owned()).await?;
        }
        Some(Commands::Export {
            playlist,
            game,
            destination,
            retro_arch_path,
        }) => {
            export(playlist, game, destination, retro_arch_path.to_owned())?;
        }
        Some(Commands::Import {
            origin,
            destination,
            retro_arch_path,
        }) => {
            import(origin, destination.to_owned(), retro_arch_path.to_owned())?;
        }
        None => {}
    }

    Ok(())
}

fn get_retro_arch_config(retro_arch_path: Option<PathBuf>) -> Result<(Ini, PathBuf)> {
    let retro_arch_path = retro_arch_path
        .or_else(|| {
            let steam_dir = SteamDir::locate().expect("Steam not found");
            let (app, library) = steam_dir.find_app(1118310).expect("RetroArch not found")?;

            Some(library.resolve_app_dir(&app))
        })
        .expect("RetroArch not installed in Steam");

    let config_file_path = retro_arch_path.join("retroarch.cfg");
    Ok((Ini::load_from_file(config_file_path)?, retro_arch_path))
}

fn get_path_from_config(config: &Ini, key: &str, retro_arch_path: &PathBuf) -> Result<PathBuf> {
    let path = config
        .get_from(None::<String>, key)
        .expect(&format!("Key {key} not found in RetroArch config"));

    let result = if path.starts_with(":") {
        retro_arch_path.join(path.replace(":/", "./").replace(":", ""))
    } else {
        PathBuf::from(path)
    };

    Ok(result)
}
