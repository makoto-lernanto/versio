//! `vs renomi`: renomas branĉon.

use anyhow::Context;
use libversio::deponejo::Deponejo;

/// Plenumas la komandon kun la malnova kaj la nova branĉaj nomoj.
pub(crate) fn plenumi(malnova: &str, nova: &str) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    deponejo
        .renomi_branĉon(malnova, nova)
        .with_context(|| format!("malsukcesis renomi la branĉon {malnova}"))?;
    println!("Renomis branĉon {malnova} al {nova}");
    Ok(())
}
