//! `vs forigi`: forigas branĉon.

use anyhow::Context;
use libversio::deponejo::Deponejo;

/// Plenumas la komandon kun la branĉa nomo.
pub(crate) fn plenumi(nomo: &str) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let estis_nuna = deponejo
        .nuna_branĉo()
        .context("malsukcesis legi la nunan branĉon")?
        .as_deref()
        == Some(nomo);
    deponejo
        .forigi_branĉon(nomo)
        .with_context(|| format!("malsukcesis forigi la branĉon {nomo}"))?;
    if estis_nuna {
        println!("Forigis branĉon {nomo} (KAPO nun estas detaĉita)");
    } else {
        println!("Forigis branĉon {nomo}");
    }
    Ok(())
}
