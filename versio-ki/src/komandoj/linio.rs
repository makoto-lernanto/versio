//! `vs linio`: kreas branĉon ĉe `KAPO` aŭ ĉe la donita komito.

use anyhow::Context;
use libversio::deponejo::{Ago, Deponejo};

use super::haketo;

/// Plenumas la komandon kun la branĉa nomo kaj laŭvola baza komito.
pub(crate) fn plenumi(nomo: &str, bazo: Option<&str>) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    if deponejo
        .legi_branĉon(nomo)
        .context("malsukcesis legi la branĉon")?
        .is_some()
    {
        anyhow::bail!("la branĉo «{nomo}» jam ekzistas");
    }

    let celo = match bazo {
        Some(teksto) => haketo::solvi_referencon(&deponejo, teksto)?,
        None => deponejo
            .legi_kapon()
            .context("malsukcesis legi KAPO")?
            .context("ankoraŭ ne ekzistas komito por branĉi")?,
    };
    deponejo
        .legi_komiton(&celo)
        .with_context(|| format!("la komito {celo} ne legeblas"))?;
    deponejo
        .movi_branĉon(nomo, &celo, Ago::Krei)
        .context("malsukcesis skribi la branĉon")?;

    println!("Kreita branĉo {nomo} ĉe {celo}");
    Ok(())
}
