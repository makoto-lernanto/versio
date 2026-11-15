//! `vs komitu`: kaptas la laborspacon kaj registras novan komiton.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use libversio::deponejo::Deponejo;
use libversio::objektoj::Komito;

/// Plenumas la komandon kun la donita mesaĝo.
pub(crate) fn plenumi(mesagxo: &str) -> anyhow::Result<()> {
    let mesagxo = mesagxo.trim();
    anyhow::ensure!(
        !mesagxo.is_empty(),
        "la komita mesaĝo ne rajtas esti malplena"
    );

    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let arbo = deponejo
        .kapti_arbon()
        .context("malsukcesis legi la laborspacon")?;
    let gepatro = deponejo.legi_head().context("malsukcesis legi HEAD")?;
    let kreita_je = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let komito = Komito {
        arbo,
        gepatro,
        mesagxo: mesagxo.to_owned(),
        kreita_je,
    };
    let haketo = deponejo
        .konservi_komiton(&komito)
        .context("malsukcesis konservi la komiton")?;
    deponejo
        .skribi_head(&haketo)
        .context("malsukcesis ĝisdatigi HEAD")?;

    println!("Komito {haketo}");
    Ok(())
}
