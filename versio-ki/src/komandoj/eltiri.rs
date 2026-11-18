//! `vs eltiri`: restarigas la laborspacon al komito aŭ linio.

use anyhow::Context;
use libversio::Eraro;
use libversio::deponejo::Deponejo;
use libversio::stokado::Haketo;

use super::haketo;

/// Plenumas la komandon kun la celata komito aŭ linio kaj la perforta opcio.
pub(crate) fn plenumi(celo: &str, perforta: bool) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;
    let (haketo, linio) = solvi_celon(&deponejo, celo)?;

    if !perforta {
        let stato = deponejo
            .stato()
            .context("malsukcesis kalkuli la staton de la laborspaco")?;
        anyhow::ensure!(
            stato.estas_pura(),
            "la laborspaco enhavas nekomititajn ŝanĝojn; komitu ilin aŭ uzu --perforta"
        );
    }

    deponejo
        .eltiri(&haketo)
        .context("malsukcesis restarigi la laborspacon")?;
    if let Some(nomo) = &linio {
        deponejo
            .agordi_nunan_branĉon(nomo)
            .context("malsukcesis agordi la nunan linion")?;
    } else {
        deponejo
            .forigi_nunan_branĉon()
            .context("malsukcesis forigi la nunan linion")?;
    }
    deponejo
        .skribi_kapon(&haketo)
        .context("malsukcesis ĝisdatigi KAPO")?;

    if let Some(nomo) = linio {
        println!("Sur linio {nomo} ({haketo})");
    } else {
        println!("Laborspaco restarigita al {haketo}");
    }
    Ok(())
}

/// Solvas linian nomon aŭ komitan haketon; linio havas prioritaton.
fn solvi_celon(deponejo: &Deponejo, celo: &str) -> anyhow::Result<(Haketo, Option<String>)> {
    let nomo = celo.trim();
    match deponejo.legi_branĉon(nomo) {
        Ok(Some(haketo)) => return Ok((haketo, Some(nomo.to_owned()))),
        // Ne ekzistanta aŭ nevalida linia nomo signifas, ke la celo estas haketo.
        Ok(None) | Err(Eraro::NevalidaVojaKomponanto) => {}
        Err(eraro) => return Err(eraro.into()),
    }
    Ok((haketo::solvi(deponejo, nomo)?, None))
}
