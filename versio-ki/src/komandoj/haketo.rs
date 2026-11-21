//! Komunaj utilaĵoj por haketoj en la komandlinia tavolo.

use libversio::Eraro;
use libversio::deponejo::Deponejo;
use libversio::stokado::{HAKETA_LONGO, Haketo};

const PLENA_LONGO: usize = HAKETA_LONGO.saturating_mul(2);
const MINIMUMA_LONGO: usize = 4;
const MALLONGA_LONGO: usize = 12;

/// La unuaj dek du deksesumaj signoj sufiĉas por la homa legado.
pub(crate) fn mallongigi(haketo: &Haketo) -> String {
    haketo.to_string().chars().take(MALLONGA_LONGO).collect()
}

/// Solvas referencon: etikedo, branĉo aŭ haketo, laŭ tiu prioritato.
pub(crate) fn solvi_referencon(deponejo: &Deponejo, teksto: &str) -> anyhow::Result<Haketo> {
    let teksto = teksto.trim();
    match deponejo.solvi_etikedon(teksto) {
        Ok(Some(haketo)) => return Ok(haketo),
        Ok(None) | Err(Eraro::NevalidaVojaKomponanto) => {}
        Err(eraro) => return Err(eraro.into()),
    }
    match deponejo.legi_branĉon(teksto) {
        Ok(Some(haketo)) => return Ok(haketo),
        Ok(None) | Err(Eraro::NevalidaVojaKomponanto) => {}
        Err(eraro) => return Err(eraro.into()),
    }
    if teksto.bytes().all(|bajto| bajto.is_ascii_hexdigit()) {
        solvi(deponejo, teksto)
    } else {
        anyhow::bail!("neniu etikedo nek branĉo nomiĝas «{teksto}»")
    }
}

/// Solvas plenan aŭ unikan mallongan haketon en la objektstokejo.
///
/// Nur objektoj, kiuj malseriigeblas kiel komito, estas konsiderataj; tiel
/// dosieraj objektoj kaj arboj ne kaŭzas falsajn kongruojn.
pub(crate) fn solvi(deponejo: &Deponejo, teksto: &str) -> anyhow::Result<Haketo> {
    let teksto = teksto.trim().to_ascii_lowercase();
    anyhow::ensure!(
        (MINIMUMA_LONGO..=PLENA_LONGO).contains(&teksto.len())
            && teksto.bytes().all(|bajto| bajto.is_ascii_hexdigit()),
        "nevalida haketo: «{teksto}»"
    );
    if teksto.len() == PLENA_LONGO {
        return Ok(teksto.parse::<Haketo>()?);
    }

    let mut trovitaj = Vec::new();
    for haketo in deponejo.stokejo().haketoj()? {
        let plena = haketo.to_string();
        if plena
            .get(..teksto.len())
            .is_none_or(|prefikso| prefikso != teksto.as_str())
        {
            continue;
        }
        match deponejo.legi_komiton(&haketo) {
            Ok(_) => trovitaj.push(haketo),
            // Dosieraj objektoj kaj arboj ne malseriigeblas kiel komitoj.
            Err(
                Eraro::Malseriigo(_) | Eraro::NevalidaDatumo { .. } | Eraro::TrolongaEnigo { .. },
            ) => {}
            Err(eraro) => return Err(eraro.into()),
        }
    }

    match trovitaj.as_slice() {
        [] => anyhow::bail!("neniu komito kongruas kun «{teksto}»"),
        [ununura] => Ok(*ununura),
        _ => anyhow::bail!("la mallonga haketo «{teksto}» estas ambigua"),
    }
}
