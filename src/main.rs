mod update_cores;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::UpdateCores {
            version,
            retro_arch_path,
        }) => {
            update_cores::update_cores(version.to_owned(), retro_arch_path.to_owned()).await?;
        }
        None => {}
    }

    Ok(())
}
