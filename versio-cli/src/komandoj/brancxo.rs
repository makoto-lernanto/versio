//! `vs brancxo`: kreas branĉon ĉe `HEAD` aŭ ĉe la donita komito.

use anyhow::Context;
use libversio::deponejo::Deponejo;

use super::haketo;

/// Plenumas la komandon kun la branĉa nomo kaj laŭvola baza komito.
pub(crate) fn plenumi(nomo: &str, bazo: Option<&str>) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    if deponejo
        .legi_brancxon(nomo)
        .context("malsukcesis legi la branĉon")?
        .is_some()
    {
        anyhow::bail!("la branĉo «{nomo}» jam ekzistas");
    }

    let celo = match bazo {
        Some(teksto) => haketo::solvi(&deponejo, teksto)?,
        None => deponejo
            .legi_head()
            .context("malsukcesis legi HEAD")?
            .context("ankoraŭ ne ekzistas komito por branĉi")?,
    };
    deponejo
        .legi_komiton(&celo)
        .with_context(|| format!("la komito {celo} ne legeblas"))?;
    deponejo
        .skribi_brancxon(nomo, &celo)
        .context("malsukcesis skribi la branĉon")?;

    println!("Kreita branĉo {nomo} ĉe {celo}");
    Ok(())
}
