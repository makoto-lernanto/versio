//! `vs iniciati`: kreas la deponejan strukturon en la nuna dosierujo.

use anyhow::Context;
use libversio::deponejo::{Deponejo, VS_DOSIERUJO};

/// Plenumas la komandon.
pub fn plenumi() -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let jam = radiko.join(VS_DOSIERUJO).is_dir();
    let deponejo = Deponejo::iniciati(radiko).context("malsukcesis iniciati la deponejon")?;

    if jam {
        println!("Deponejo jam ekzistas ĉe {}", deponejo.radiko().display());
    } else {
        println!("Iniciatita deponejo ĉe {}", deponejo.radiko().display());
    }

    Ok(())
}
