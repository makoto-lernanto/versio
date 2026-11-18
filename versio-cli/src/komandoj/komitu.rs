//! `vs komitu`: kaptas la laborspacon kaj registras novan komiton.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use libversio::deponejo::Deponejo;
use libversio::objektoj::Komito;

/// Plenumas la komandon kun la donita noto.
pub(crate) fn plenumi(noto: &str) -> anyhow::Result<()> {
    let mesaĝo = noto.trim();
    anyhow::ensure!(
        !mesaĝo.is_empty(),
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
        mesaĝo: mesaĝo.to_owned(),
        kreita_je,
        dua_gepatro: None,
    };
    let haketo = deponejo
        .konservi_komiton(&komito)
        .context("malsukcesis konservi la komiton")?;
    if let Some(nomo) = deponejo
        .nuna_branĉo()
        .context("malsukcesis legi la nunan linion")?
    {
        deponejo
            .skribi_branĉon(&nomo, &haketo)
            .with_context(|| format!("malsukcesis ĝisdatigi la linion {nomo}"))?;
    }
    deponejo
        .skribi_head(&haketo)
        .context("malsukcesis ĝisdatigi HEAD")?;

    println!("Komito {haketo}");
    Ok(())
}
