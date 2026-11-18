//! `vs kunfandi`: kunfandas branĉon en la nunan branĉon.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use libversio::deponejo::{Deponejo, Kunfando};

/// Plenumas la komandon por la donita branĉo.
pub(crate) fn plenumi(brancxo: &str) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let celo = deponejo
        .legi_brancxon(brancxo)
        .with_context(|| format!("malsukcesis legi la branĉon {brancxo}"))?
        .ok_or_else(|| anyhow::anyhow!("la branĉo {brancxo} ne ekzistas"))?;
    let nuna = deponejo
        .legi_head()
        .context("malsukcesis legi HEAD")?
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
    let mesagxo = format!("Kunfandis branĉon {brancxo}");
    let rezulto = deponejo
        .kunfandi(&nuna, &celo, &mesagxo, kreita_je)
        .context("malsukcesis kunfandi")?;

    match rezulto {
        Kunfando::Nenio => println!("Nenio farenda: {brancxo} jam estas kunfandita"),
        Kunfando::Antauen(haketo) => {
            gxisdatigi_refs(&deponejo, &haketo)?;
            println!("Antaŭen al {haketo} ({brancxo})");
        }
        Kunfando::Kunfandita(haketo) => {
            gxisdatigi_refs(&deponejo, &haketo)?;
            println!("Kunfanda komito {haketo}");
        }
    }
    Ok(())
}

/// Movas la nunan branĉon kaj `HEAD` al la donita komito.
fn gxisdatigi_refs(deponejo: &Deponejo, haketo: &libversio::stokado::Haketo) -> anyhow::Result<()> {
    if let Some(nomo) = deponejo
        .nuna_brancxo()
        .context("malsukcesis legi la nunan branĉon")?
    {
        deponejo
            .skribi_brancxon(&nomo, haketo)
            .with_context(|| format!("malsukcesis ĝisdatigi la branĉon {nomo}"))?;
    }
    deponejo
        .skribi_head(haketo)
        .context("malsukcesis ĝisdatigi HEAD")?;
    Ok(())
}
