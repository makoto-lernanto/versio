//! `vs kunfandi`: kunfandas branĉon en la nunan branĉon.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use libversio::deponejo::{Deponejo, Kunfando};

/// Plenumas la komandon por la donita branĉo.
pub(crate) fn plenumi(branĉo: &str) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let celo = deponejo
        .legi_branĉon(branĉo)
        .with_context(|| format!("malsukcesis legi la branĉon {branĉo}"))?
        .ok_or_else(|| anyhow::anyhow!("la branĉo {branĉo} ne ekzistas"))?;
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
    let mesaĝo = format!("Kunfandis branĉon {branĉo}");
    let rezulto = deponejo
        .kunfandi(&nuna, &celo, &mesaĝo, kreita_je)
        .context("malsukcesis kunfandi")?;

    match rezulto {
        Kunfando::Nenio => println!("Nenio farenda: {branĉo} jam estas kunfandita"),
        Kunfando::Antauen(haketo) => {
            ĝisdatigi_referencojn(&deponejo, &haketo)?;
            println!("Antaŭen al {haketo} ({branĉo})");
        }
        Kunfando::Kunfandita(haketo) => {
            ĝisdatigi_referencojn(&deponejo, &haketo)?;
            println!("Kunfanda komito {haketo}");
        }
    }
    Ok(())
}

/// Movas la nunan branĉon kaj `KAPO` al la donita komito.
fn ĝisdatigi_referencojn(
    deponejo: &Deponejo,
    haketo: &libversio::stokado::Haketo,
) -> anyhow::Result<()> {
    if let Some(nomo) = deponejo
        .nuna_branĉo()
        .context("malsukcesis legi la nunan branĉon")?
    {
        deponejo
            .skribi_branĉon(&nomo, haketo)
            .with_context(|| format!("malsukcesis ĝisdatigi la branĉon {nomo}"))?;
    }
    deponejo
        .skribi_kapon(haketo)
        .context("malsukcesis ĝisdatigi KAPO")?;
    Ok(())
}
