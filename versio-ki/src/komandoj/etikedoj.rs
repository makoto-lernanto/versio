//! `vs etikedoj`: listigas la etikedojn kun iliaj celoj.

use std::io::{BufWriter, Write};

use anyhow::Context;
use libversio::deponejo::{Deponejo, EtikedaCelo};

use super::haketo::mallongigi;

/// Plenumas la komandon.
pub(crate) fn plenumi() -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;

    let etikedoj = deponejo
        .etikedoj()
        .context("malsukcesis listigi la etikedojn")?;
    let mut eligo = BufWriter::new(std::io::stdout().lock());
    if etikedoj.is_empty() {
        writeln!(eligo, "Neniu etikedo")?;
    }
    for etikedo in etikedoj {
        let haketo = deponejo
            .solvi_etikedon(&etikedo.nomo)
            .with_context(|| format!("malsukcesis solvi la etikedon {}", etikedo.nomo))?
            .ok_or_else(|| anyhow::anyhow!("la etikedo «{}» montras al nenio", etikedo.nomo))?;
        match &etikedo.celo {
            EtikedaCelo::Komito(_) => {
                writeln!(eligo, "{}  {}", etikedo.nomo, mallongigi(&haketo))?;
            }
            EtikedaCelo::Alias(aliaso) => {
                writeln!(
                    eligo,
                    "{}  {}  -> {aliaso}",
                    etikedo.nomo,
                    mallongigi(&haketo)
                )?;
            }
        }
    }
    eligo.flush()?;
    Ok(())
}
