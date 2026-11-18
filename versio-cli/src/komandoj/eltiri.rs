//! `vs eltiri`: restarigas la laborspacon al la arbo de komito.

use anyhow::Context;
use libversio::Eraro;
use libversio::deponejo::Deponejo;
use libversio::stokado::{HAKETA_LONGO, Haketo};

const PLENA_LONGO: usize = HAKETA_LONGO.saturating_mul(2);
const MINIMUMA_LONGO: usize = 4;

/// Plenumas la komandon kun la celata komito kaj la perforta opcio.
pub(crate) fn plenumi(haketo: &str, perforta: bool) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;
    let haketo = solvi_haketon(&deponejo, haketo)?;

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
    deponejo
        .skribi_head(&haketo)
        .context("malsukcesis ĝisdatigi HEAD")?;

    println!("Laborspaco restarigita al {haketo}");
    Ok(())
}

/// Solvas plenan aŭ unikan mallongan haketon en la objektstokejo.
fn solvi_haketon(deponejo: &Deponejo, teksto: &str) -> anyhow::Result<Haketo> {
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
            // Bloboj kaj arboj ne malseriigeblas kiel komitoj.
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
