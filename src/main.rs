mod import_export;
mod update_cores;

use crate::import_export::export::export;
use crate::import_export::import::import;
use crate::update_cores::update_cores;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

    #[command(about = "Exports a game from RetroArch to a removable media")]
    Export {
        #[arg(help = "Playlist to export from", required = true)]
        playlist: String,

        #[arg(help = "Game to be exported", required = true)]
        game: String,

        #[arg(
            help = "Export destination path [default: first available removable media]",
            required = false
        )]
        destination: Option<PathBuf>,

        #[arg(
            short,
            long,
            help = "Manually override RetroArch path (Will be queried from Steam otherwise)"
        )]
        retro_arch_path: Option<PathBuf>,
    },

    #[command(about = "Imports a game from a removable media to RetroArch")]
    Import {
        #[arg(help = "Playlist to import into", required = true)]
        playlist: String,

        #[arg(help = "Game to be imported", required = true)]
        game: String,

        #[arg(
            help = "Import origin path [default: first available removable media]",
            required = false
        )]
        origin: Option<PathBuf>,

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
            export(playlist, game, destination, retro_arch_path)?;
        }
        Some(Commands::Import {
            playlist,
            game,
            origin,
            retro_arch_path,
        }) => {
            import(playlist, game, origin, retro_arch_path)?;
        }
        None => {}
    }

    Ok(())
}
