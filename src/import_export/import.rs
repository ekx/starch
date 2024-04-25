use crate::import_export::dummy_print;

use std::path::PathBuf;

pub(crate) fn import(
    playlist: &String,
    game: &String,
    origin: &Option<PathBuf>,
    retro_arch_path: &Option<PathBuf>,
) -> anyhow::Result<()> {
    dummy_print(playlist, game, origin, retro_arch_path);

    Ok(())
}
