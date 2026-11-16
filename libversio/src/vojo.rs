//! Validigo de vojkonsisteroj.
//!
//! La kanonikiga strategio de Versio estas: nomoj estas konservataj kiel
//! validaj UTF-8-signoĉenoj, ordigitaj laŭ siaj bajtoj, kaj unu arbo ne
//! rajtas enhavi du enskribojn kun identa nomo. Unicode-normaligo (NFC/NFD)
//! estos aldonita aparte; la testoj de tiu ĉi modulo fiksas la nunan
//! konduton por ke la normo ne ŝanĝiĝu silente.

use crate::eraroj::Eraro;

/// Maksimuma longo de unu voja komponanto en bajtoj.
pub const MAKSIMA_KOMPONANTO: usize = 255;

/// Maksimuma longo de relativa vojo (komponantoj kunigitaj per `/`) en bajtoj.
pub const MAKSIMA_VOJO: usize = 4096;

/// Validigas relativan vojon el komponantoj apartigitaj per `/`.
///
/// # Errors
///
/// Redonas [`Eraro::NevalidaVojaKomponanto`], se la vojo estas pli longa ol
/// [`MAKSIMA_VOJO`] aŭ se iu komponanto ne trapasas [`validi_komponanton`].
pub fn validi_vojon(vojo: &str) -> Result<(), Eraro> {
    if vojo.len() > MAKSIMA_VOJO {
        return Err(Eraro::NevalidaVojaKomponanto);
    }
    for komponanto in vojo.split('/') {
        validi_komponanton(komponanto)?;
    }
    Ok(())
}

/// Validigas unu vojan komponanton (dosier- aŭ dosierujnomon).
///
/// # Errors
///
/// Redonas [`Eraro::NevalidaVojaKomponanto`], se la komponanto estas malplena,
/// pli longa ol [`MAKSIMA_KOMPONANTO`], estas `.` aŭ `..`, aŭ enhavas `/`,
/// `\` aŭ NUL-signon.
pub fn validi_komponanton(komponanto: &str) -> Result<(), Eraro> {
    validi_bajtojn(komponanto.as_bytes())
}

/// Validigas vojan komponanton donitan kiel krudaj bajtoj.
///
/// # Errors
///
/// Redonas [`Eraro::NevalidaVojaKomponanto`] laŭ la samaj reguloj kiel
/// [`validi_komponanton`].
pub fn validi_bajtojn(bajtoj: &[u8]) -> Result<(), Eraro> {
    if bajtoj.is_empty() || bajtoj.len() > MAKSIMA_KOMPONANTO {
        return Err(Eraro::NevalidaVojaKomponanto);
    }
    if bajtoj == b"." || bajtoj == b".." {
        return Err(Eraro::NevalidaVojaKomponanto);
    }
    for &bajto in bajtoj {
        if matches!(bajto, b'/' | b'\\' | 0) {
            return Err(Eraro::NevalidaVojaKomponanto);
        }
    }
    Ok(())
}

#[cfg(test)]
mod testoj {
    use super::{MAKSIMA_KOMPONANTO, MAKSIMA_VOJO, validi_komponanton, validi_vojon};

    #[test]
    fn testo_validi_vojon() {
        for vojo in ["a", "src/lib.rs", "a/b/c"] {
            assert!(validi_vojon(vojo).is_ok(), "{vojo}");
        }
        for vojo in ["", "/a", "a/", "a//b", "a/../b", "./a"] {
            assert!(validi_vojon(vojo).is_err(), "{vojo:?}");
        }
        let longa = "a/".repeat(MAKSIMA_VOJO / 2) + "a";
        assert!(validi_vojon(&longa).is_err());
    }

    #[test]
    fn testo_akceptas_normalajn_nomojn() {
        for nomo in ["README.md", "src", "a", "ünïkodo"] {
            assert!(validi_komponanton(nomo).is_ok(), "{nomo}");
        }
    }

    #[test]
    fn testo_rifuzas_danĝerajn_nomojn() {
        for nomo in ["", ".", "..", "a/b", "a\\b", "a\0b"] {
            assert!(validi_komponanton(nomo).is_err(), "{nomo:?}");
        }
        let longa = "a".repeat(MAKSIMA_KOMPONANTO + 1);
        assert!(validi_komponanton(&longa).is_err());
    }
}
