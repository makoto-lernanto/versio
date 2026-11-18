//! `vs kunfandi`: kunfandas linion en la nunan linion.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use libversio::deponejo::{Deponejo, Kunfando};

/// Plenumas la komandon por la donita linio.
pub(crate) fn plenumi(linio: &str) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let celo = deponejo
        .legi_branĉon(linio)
        .with_context(|| format!("malsukcesis legi la linion {linio}"))?
        .ok_or_else(|| anyhow::anyhow!("la linio {linio} ne ekzistas"))?;
    let nuna = deponejo
        .legi_kapon()
        .context("malsukcesis legi KAPO")?
        .ok_or_else(|| anyhow::anyhow!("neniu komito ankoraŭ; faru komiton antaŭ kunfandi"))?;

    let stato = deponejo
        .stato()
        .context("malsukcesis kalkuli la staton de la laborspaco")?;
    anyhow::ensure!(
        stato.estas_pura(),
        "la laborspaco enhavas nekomititajn ŝanĝojn; komitu ilin antaŭ kunfandi"
    );

    let kreita_je = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mesaĝo = format!("Kunfandis linion {linio}");
    let rezulto = deponejo
        .kunfandi(&nuna, &celo, &mesaĝo, kreita_je)
        .context("malsukcesis kunfandi")?;

    match rezulto {
        Kunfando::Nenio => println!("Nenio farenda: {linio} jam estas kunfandita"),
        Kunfando::Antauen(haketo) => {
            ĝisdatigi_referencojn(&deponejo, &haketo)?;
            println!("Antaŭen al {haketo} ({linio})");
        }
        Kunfando::Kunfandita(haketo) => {
            ĝisdatigi_referencojn(&deponejo, &haketo)?;
            println!("Kunfanda komito {haketo}");
        }
    }
    Ok(())
}

/// Movas la nunan linion kaj `KAPO` al la donita komito.
fn ĝisdatigi_referencojn(
    deponejo: &Deponejo,
    haketo: &libversio::stokado::Haketo,
) -> anyhow::Result<()> {
    if let Some(nomo) = deponejo
        .nuna_branĉo()
        .context("malsukcesis legi la nunan linion")?
    {
        deponejo
            .skribi_branĉon(&nomo, haketo)
            .with_context(|| format!("malsukcesis ĝisdatigi la linion {nomo}"))?;
    }
    deponejo
        .skribi_kapon(haketo)
        .context("malsukcesis ĝisdatigi KAPO")?;
    Ok(())
}
