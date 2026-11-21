//! `vs movhistorio`: montras la antaŭajn valorojn de `KAPO` kaj la branĉoj.

use std::io::{BufWriter, Write};

use anyhow::Context;
use libversio::deponejo::Deponejo;
use libversio::stokado::Haketo;

use super::haketo::mallongigi;
use super::tempo::formati_tempon;

/// Plenumas la komandon.
pub(crate) fn plenumi() -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let historio = deponejo
        .movhistorio()
        .context("malsukcesis legi la movhistorion")?;
    let mut eligo = BufWriter::new(std::io::stdout().lock());
    if historio.movoj().is_empty() {
        writeln!(eligo, "Neniu movo registrita")?;
    }
    for movo in historio.movoj().iter().rev() {
        writeln!(
            eligo,
            "{}  {}  {} -> {}  {}",
            formati_tempon(movo.tempo),
            movo.celo.nomo(),
            mallongigi_opcion(movo.malnova),
            mallongigi_opcion(movo.nova),
            movo.ago.priskribo()
        )?;
    }
    eligo.flush()?;
    Ok(())
}

fn mallongigi_opcion(haketo: Option<Haketo>) -> String {
    haketo.map_or_else(|| String::from("-"), |haketo| mallongigi(&haketo))
}
