use crate::import_export::dummy_print;

use std::path::PathBuf;

pub(crate) fn export(
    playlist: &String,
    game: &String,
    destination: &Option<PathBuf>,
    retro_arch_path: &Option<PathBuf>,
) -> anyhow::Result<()> {
    dummy_print(playlist, game, destination, retro_arch_path);

    Ok(())
}
