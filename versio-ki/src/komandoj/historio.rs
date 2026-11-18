//! `vs historio`: montras la komito-ĉenon ekde `KAPO`.

use std::collections::HashSet;
use std::io::{BufWriter, Write};

use anyhow::Context;
use libversio::deponejo::Deponejo;

use super::haketo::mallongigi;

const SEKUNDOJ_EN_TAGO: u64 = 86_400;
const SEKUNDOJ_EN_HORO: u64 = 3_600;
const SEKUNDOJ_EN_MINUTO: u64 = 60;

/// Plenumas la komandon kun la donita limo da komitoj.
pub(crate) fn plenumi(nombro: Option<usize>) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let nuna = deponejo.legi_kapon().context("malsukcesis legi KAPO")?;
    let mut eligo = BufWriter::new(std::io::stdout().lock());
    let Some(nuna) = nuna else {
        writeln!(eligo, "Neniu komito ankoraŭ")?;
        eligo.flush()?;
        return Ok(());
    };

    let mut restantaj = nombro.unwrap_or(usize::MAX);
    let mut stako = vec![nuna];
    let mut vizititaj = HashSet::new();
    while let Some(haketo) = stako.pop() {
        if restantaj == 0 {
            break;
        }
        if !vizititaj.insert(haketo) {
            continue;
        }
        restantaj = restantaj.saturating_sub(1);

        let komito = deponejo
            .legi_komiton(&haketo)
            .with_context(|| format!("malsukcesis legi la komiton {haketo}"))?;
        writeln!(
            eligo,
            "{}  {}  {}",
            mallongigi(&haketo),
            formati_tempon(komito.kreita_je),
            unulinia(&komito.mesaĝo)
        )?;
        // La stako estas LIFO: la dua gepatro estas puŝita unue, do la unua
        // gepatro kaj ĝia ĉeno estas montritaj antaŭ la kunfandita linio.
        if let Some(dua) = komito.dua_gepatro {
            stako.push(dua);
        }
        stako.extend(komito.gepatro);
    }
    eligo.flush()?;
    Ok(())
}

/// Kunpremas la komitan mesaĝon al unu linio.
fn unulinia(mesaĝo: &str) -> String {
    mesaĝo.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Formats uniksan tempstampon kiel `YYYY-MM-DD HH:MM:SS UTC`.
fn formati_tempon(sekundoj: u64) -> String {
    let tagoj = sekundoj / SEKUNDOJ_EN_TAGO;
    let en_tago = sekundoj % SEKUNDOJ_EN_TAGO;
    let horo = en_tago / SEKUNDOJ_EN_HORO;
    let minuto = en_tago % SEKUNDOJ_EN_HORO / SEKUNDOJ_EN_MINUTO;
    let sekundo = en_tago % SEKUNDOJ_EN_MINUTO;
    let (jaro, monato, tago) = civila_dato(tagoj);
    format!("{jaro:04}-{monato:02}-{tago:02} {horo:02}:{minuto:02}:{sekundo:02} UTC")
}

/// Konvertas tagojn ekde 1970-01-01 al `(jaro, monato, tago)` laŭ UTC.
///
/// Uzita algoritmo: Howard Hinnant, «chrono-Compatible Low-Level Date
/// Algorithms».
fn civila_dato(tagoj: u64) -> (u64, u64, u64) {
    let z = tagoj.saturating_add(719_468);
    let epoko = z / 146_097;
    let tago_de_epoko = z % 146_097;
    let jaro_de_epoko = tago_de_epoko
        .saturating_sub(tago_de_epoko / 1_460)
        .saturating_add(tago_de_epoko / 36_524)
        .saturating_sub(tago_de_epoko / 146_096)
        / 365;
    let pasintaj = 365u64
        .saturating_mul(jaro_de_epoko)
        .saturating_add(jaro_de_epoko / 4)
        .saturating_sub(jaro_de_epoko / 100);
    let tago_de_jaro = tago_de_epoko.saturating_sub(pasintaj);
    let monata_indekso = (5u64.saturating_mul(tago_de_jaro).saturating_add(2)) / 153;
    let tago = tago_de_jaro
        .saturating_sub((153u64.saturating_mul(monata_indekso).saturating_add(2)) / 5)
        .saturating_add(1);
    let monata_numero = if monata_indekso < 10 {
        monata_indekso.saturating_add(3)
    } else {
        monata_indekso.saturating_sub(9)
    };
    let jaro = jaro_de_epoko
        .saturating_add(epoko.saturating_mul(400))
        .saturating_add(u64::from(monata_numero <= 2));
    (jaro, monata_numero, tago)
}

#[cfg(test)]
mod testoj {
    use super::super::haketo::mallongigi;
    use super::{formati_tempon, unulinia};
    use libversio::stokado::Haketo;

    #[test]
    fn testo_formati_tempon() {
        assert_eq!(formati_tempon(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(formati_tempon(86_399), "1970-01-01 23:59:59 UTC");
        assert_eq!(formati_tempon(951_782_400), "2000-02-29 00:00:00 UTC");
        assert_eq!(formati_tempon(1_700_000_000), "2023-11-14 22:13:20 UTC");
    }

    #[test]
    fn testo_mallongigi() {
        let haketo = Haketo::el_bajtoj([0xab; 32]);
        assert_eq!(mallongigi(&haketo), "abababababab");
    }

    #[test]
    fn testo_unulinia() {
        assert_eq!(unulinia("unua\n\n  dua\t tria "), "unua dua tria");
        assert_eq!(unulinia(""), "");
    }
}
