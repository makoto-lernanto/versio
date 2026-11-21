//! `vs etikedi`: markas komiton aŭ alian etikedon per nomo.

use anyhow::Context;
use libversio::Eraro;
use libversio::deponejo::{Deponejo, EtikedaCelo};

use super::haketo;

/// Plenumas la komandon kun la etikeda nomo kaj laŭvola celo.
pub(crate) fn plenumi(nomo: &str, celo: Option<&str>) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let ekzistas = match deponejo.legi_etikedon(nomo) {
        Ok(celo) => celo.is_some(),
        Err(Eraro::NevalidaVojaKomponanto) => anyhow::bail!("nevalida etikeda nomo: «{nomo}»"),
        Err(eraro) => return Err(eraro.into()),
    };
    anyhow::ensure!(!ekzistas, "la etikedo «{nomo}» jam ekzistas");

    let (etikeda_celo, pinto) = if let Some(teksto) = celo {
        let celo_estas_etikedo = match deponejo.legi_etikedon(teksto) {
            Ok(celo) => celo.is_some(),
            Err(Eraro::NevalidaVojaKomponanto) => false,
            Err(eraro) => return Err(eraro.into()),
        };
        if celo_estas_etikedo {
            let haketo = deponejo
                .solvi_etikedon(teksto)
                .with_context(|| format!("malsukcesis solvi la etikedon {teksto}"))?
                .ok_or_else(|| anyhow::anyhow!("la etikedo «{teksto}» montras al nenio"))?;
            (EtikedaCelo::Alias(teksto.to_owned()), haketo)
        } else {
            let haketo = haketo::solvi(&deponejo, teksto)?;
            deponejo
                .legi_komiton(&haketo)
                .with_context(|| format!("la komito {haketo} ne legeblas"))?;
            (EtikedaCelo::Komito(haketo), haketo)
        }
    } else {
        let haketo = deponejo
            .legi_kapon()
            .context("malsukcesis legi KAPO")?
            .context("ankoraŭ ne ekzistas komito por etikedi")?;
        (EtikedaCelo::Komito(haketo), haketo)
    };

    deponejo
        .skribi_etikedon(nomo, &etikeda_celo)
        .context("malsukcesis skribi la etikedon")?;
    println!("Etikedis {nomo} ĉe {pinto}");
    Ok(())
}
