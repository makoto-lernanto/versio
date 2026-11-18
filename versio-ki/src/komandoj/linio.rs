//! `vs linio`: kreas linion ĉe `KAPO` aŭ ĉe la donita komito.

use anyhow::Context;
use libversio::deponejo::Deponejo;

use super::haketo;

/// Plenumas la komandon kun la linia nomo kaj laŭvola baza komito.
pub(crate) fn plenumi(nomo: &str, bazo: Option<&str>) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    if deponejo
        .legi_branĉon(nomo)
        .context("malsukcesis legi la linion")?
        .is_some()
    {
        anyhow::bail!("la linio «{nomo}» jam ekzistas");
    }

    let celo = match bazo {
        Some(teksto) => haketo::solvi(&deponejo, teksto)?,
        None => deponejo
            .legi_kapon()
            .context("malsukcesis legi KAPO")?
            .context("ankoraŭ ne ekzistas komito por branĉi")?,
    };
    deponejo
        .legi_komiton(&celo)
        .with_context(|| format!("la komito {celo} ne legeblas"))?;
    deponejo
        .skribi_branĉon(nomo, &celo)
        .context("malsukcesis skribi la linion")?;

    println!("Kreita linio {nomo} ĉe {celo}");
    Ok(())
}
