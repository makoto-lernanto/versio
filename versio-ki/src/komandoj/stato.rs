//! `vs stato`: montras la ŝanĝojn de la laborspaco rilate al `KAPO`.

use std::io::{BufWriter, Write};

use anyhow::Context;
use libversio::deponejo::{Deponejo, Stato};

/// Plenumas la komandon.
pub(crate) fn plenumi() -> anyhow::Result<()> {
    let radiko = std::env::current_dir().context("ne povas determini la nunan dosierujon")?;
    let deponejo =
        Deponejo::malfermi(radiko).context("la nuna dosierujo ne estas Versio-deponejo")?;
    let stato = deponejo
        .stato()
        .context("malsukcesis kalkuli la staton de la laborspaco")?;

    let mut eligo = BufWriter::new(std::io::stdout().lock());
    skribi(&mut eligo, &stato)?;
    eligo.flush()?;
    Ok(())
}

/// Skribas nur la nemalplenajn sekciojn; pura laborspaco donas unu linion.
fn skribi(eligo: &mut impl Write, stato: &Stato) -> std::io::Result<()> {
    if stato.estas_pura() {
        return writeln!(eligo, "Neniu ŝanĝo");
    }
    for (titolo, vojoj) in [
        ("Modifitaj", &stato.modifitaj),
        ("Novaj", &stato.novaj),
        ("Forigitaj", &stato.forigitaj),
    ] {
        if vojoj.is_empty() {
            continue;
        }
        writeln!(eligo, "{titolo}:")?;
        for vojo in vojoj {
            writeln!(eligo, "  {vojo}")?;
        }
    }
    Ok(())
}
