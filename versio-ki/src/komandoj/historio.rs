//! `vs historio`: montras la komito-ĉenon ekde la donita celo aŭ `KAPO`.

use std::collections::HashSet;
use std::io::{BufWriter, Write};

use anyhow::Context;
use libversio::deponejo::Deponejo;

use super::haketo;
use super::haketo::mallongigi;
use super::tempo::formati_tempon;

/// Plenumas la komandon kun la donita limo da komitoj kaj laŭvola celo.
pub(crate) fn plenumi(nombro: Option<usize>, celo: Option<&str>) -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let nuna = match celo {
        Some(teksto) => Some(haketo::solvi_referencon(&deponejo, teksto)?),
        None => deponejo.legi_kapon().context("malsukcesis legi KAPO")?,
    };
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
        // La stako estas LIFO (lasta eniro, unua eliro): la dua gepatro estas
        // puŝita unue, do la unua gepatro kaj ĝia ĉeno aperas antaŭ la
        // kunfandita branĉo.
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

#[cfg(test)]
mod testoj {
    use super::super::haketo::mallongigi;
    use super::unulinia;
    use libversio::stokado::Haketo;

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
