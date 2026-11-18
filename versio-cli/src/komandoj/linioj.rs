//! `vs linioj`: listigas la liniojn kaj markas la nunan.

use std::io::{BufWriter, Write};

use anyhow::Context;
use libversio::deponejo::Deponejo;

use super::haketo::mallongigi;

/// Plenumas la komandon.
pub(crate) fn plenumi() -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;
    let nuna = deponejo
        .nuna_branĉo()
        .context("malsukcesis legi la nunan linion")?;
    let linioj = deponejo
        .branĉoj()
        .context("malsukcesis listigi la liniojn")?;

    let mut eligo = BufWriter::new(std::io::stdout().lock());
    if linioj.is_empty() {
        writeln!(eligo, "Neniu linio")?;
    } else {
        for (nomo, haketo) in linioj {
            let marko = if nuna.as_deref() == Some(nomo.as_str()) {
                "*"
            } else {
                " "
            };
            writeln!(eligo, "{marko} {nomo}  {}", mallongigi(&haketo))?;
        }
    }
    eligo.flush()?;
    Ok(())
}
