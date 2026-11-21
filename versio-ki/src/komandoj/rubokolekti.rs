//! `vs rubokolekti`: forigas objektojn, kiujn neniu referenco atingas.

use anyhow::Context;
use libversio::deponejo::Deponejo;

/// Plenumas la komandon.
pub(crate) fn plenumi() -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let raporto = deponejo.rubokolekti().context("malsukcesis rubokolekti")?;
    println!(
        "Forigis {} {}, konservis {} {}.",
        raporto.forigitaj,
        objekto(raporto.forigitaj),
        raporto.konservitaj,
        objekto(raporto.konservitaj)
    );
    Ok(())
}

fn objekto(kvanto: usize) -> &'static str {
    if kvanto == 1 { "objekton" } else { "objektojn" }
}
