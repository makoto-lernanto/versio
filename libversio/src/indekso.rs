//! Metadatuma kaŝmemoro de la laborspaco (la indekso).
//!
//! Por ĉiu dosiero la indekso memoras ĝian modifotempon, grandecon kaj
//! `BLAKE3`-haketon. Se la modifotempo kaj la grandeco ne ŝanĝiĝis, `stato`
//! kaj `kapti_arbon` reuzas la memoritan haketon anstataŭ relegi la dosieron.
//!
//! Formato: versia bajto, `postcard`-seriigitaj enskriboj ordigitaj laŭ vojo,
//! kaj fine la `BLAKE3`-haketo de ĉio antaŭa. La indekso estas nur
//! kaŝmemoro: difektita indekso estas rifuzata kaj rekonstruata.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Serialize;

use crate::eraroj::Eraro;
use crate::kodigo::{Legilo, limigi_kvanton};
use crate::stokado::{HAKETA_LONGO, Haketo, haketi};
use crate::vojo::{MAKSIMA_VOJO, validi_vojon};

/// La versio de la indeksa formato.
pub const INDEKSA_VERSIO: u8 = 1;

/// Maksimuma nombro da enskriboj en la indekso.
pub const MAKSIMAJ_INDEKSAJ_ENSKRIBOJ: usize = 1 << 22;

/// La memoritaj metadatumoj de unu dosiero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndeksEnskribo {
    /// La relativa vojo, kun komponantoj apartigitaj per `/`.
    pub vojo: String,
    /// La modifotempo en nanosekundoj ekde la Uniksa epoko.
    pub mtempo: u64,
    /// La grandeco en bajtoj.
    pub grandeco: u64,
    /// La `BLAKE3`-haketo de la enhavo.
    pub haketo: Haketo,
}

/// Indekso: strikte laŭ vojo ordigita listo de [`IndeksEnskribo`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Indekso {
    enskriboj: Vec<IndeksEnskribo>,
}

impl Indekso {
    /// Kreas indekson el la donitaj enskriboj en ajna ordo.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::TrolongaEnigo`], se estas pli ol
    /// [`MAKSIMAJ_INDEKSAJ_ENSKRIBOJ`] enskriboj,
    /// [`Eraro::NevalidaVojaKomponanto`] por malvalida vojo, aŭ
    /// [`Eraro::DuplikataVojo`], se vojo ripetiĝas.
    pub fn el_enskriboj(mut enskriboj: Vec<IndeksEnskribo>) -> Result<Self, Eraro> {
        if enskriboj.len() > MAKSIMAJ_INDEKSAJ_ENSKRIBOJ {
            return Err(Eraro::TrolongaEnigo {
                maksimumo: MAKSIMAJ_INDEKSAJ_ENSKRIBOJ,
                akirita: enskriboj.len(),
            });
        }
        for enskribo in &enskriboj {
            validi_vojon(&enskribo.vojo)?;
        }
        enskriboj.sort_unstable_by(|unu, du| unu.vojo.cmp(&du.vojo));
        kontroli_ordon(&enskriboj)?;
        Ok(Self { enskriboj })
    }

    /// La enskriboj, ordigitaj laŭ vojo.
    #[must_use]
    pub fn enskriboj(&self) -> &[IndeksEnskribo] {
        &self.enskriboj
    }

    /// Trovas la enskribon de la donita vojo.
    #[must_use]
    pub fn trovi(&self, vojo: &str) -> Option<&IndeksEnskribo> {
        let indekso = self
            .enskriboj
            .binary_search_by(|enskribo| enskribo.vojo.as_str().cmp(vojo))
            .ok()?;
        self.enskriboj.get(indekso)
    }

    /// Redonas la memoritan haketon, se la metadatumoj ne ŝanĝiĝis.
    #[must_use]
    pub fn kaŝita_haketo(&self, vojo: &str, mtempo: u64, grandeco: u64) -> Option<Haketo> {
        self.trovi(vojo)
            .filter(|enskribo| enskribo.mtempo == mtempo && enskribo.grandeco == grandeco)
            .map(|enskribo| enskribo.haketo)
    }

    /// Seriigas la indekson kun versia bajto kaj kontrola haketo.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::Seriigo`], se la seriigado malsukcesas.
    pub fn seriigi(&self) -> Result<Vec<u8>, Eraro> {
        let korpo = postcard::to_allocvec(self).map_err(Eraro::Seriigo)?;
        let mut bajtoj = Vec::with_capacity(korpo.len().saturating_add(1 + HAKETA_LONGO));
        bajtoj.push(INDEKSA_VERSIO);
        bajtoj.extend_from_slice(&korpo);
        let kontrolo = haketi(&bajtoj);
        bajtoj.extend_from_slice(kontrolo.kiel_bajtoj());
        Ok(bajtoj)
    }

    /// Malseriigas indekson, kontrolante la haketon, la version kaj ĉiujn limojn.
    ///
    /// La vojoj ne estas validigataj komponanto post komponanto: ili servas nur
    /// kiel serĉŝlosiloj kaj neniam por malfermi dosierojn, do malvalida vojo
    /// simple neniam kongruas. La longolimo kaj la strikta ordo tamen estas
    /// devigaj, ĉar la duuma serĉo dependas de ili.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::DatumoDifektita`], se la kontrola haketo malkongruas,
    /// aŭ [`Eraro::NevalidaDatumo`], [`Eraro::Malseriigo`],
    /// [`Eraro::TrolongaEnigo`] kaj [`Eraro::DuplikataVojo`] por malvalida
    /// enhavo.
    pub fn malseriigi(datumoj: &[u8]) -> Result<Self, Eraro> {
        let mezo = datumoj
            .len()
            .checked_sub(HAKETA_LONGO)
            .ok_or(Eraro::NevalidaDatumo {
                kialo: "tro mallonga indekso",
            })?;
        let (enhavo, kontrolo) = datumoj
            .split_at_checked(mezo)
            .ok_or(Eraro::NevalidaDatumo {
                kialo: "tro mallonga indekso",
            })?;
        let atendita = Legilo::nova(kontrolo).haketo()?;
        let akirita = haketi(enhavo);
        if akirita != atendita {
            return Err(Eraro::DatumoDifektita { atendita, akirita });
        }

        let mut legilo = Legilo::nova(enhavo);
        if legilo.u8()? != INDEKSA_VERSIO {
            return Err(Eraro::NevalidaDatumo {
                kialo: "nekonata indeksa versio",
            });
        }
        // Ĉiu enskribo okupas almenaŭ 36 bajtojn (longo, nomo, du varintoj kaj
        // haketo), do la arba minimumo de `limigi_kvanton` restas valida limo.
        let deklarita = legilo.u32()?;
        let kvanto = limigi_kvanton(deklarita, legilo.longo(), MAKSIMAJ_INDEKSAJ_ENSKRIBOJ)?;
        let mut enskriboj = Vec::with_capacity(kvanto);
        for _ in 0..kvanto {
            enskriboj.push(IndeksEnskribo {
                vojo: legilo.limigita_string(MAKSIMA_VOJO)?,
                mtempo: legilo.u64()?,
                grandeco: legilo.u64()?,
                haketo: legilo.haketo()?,
            });
        }
        legilo.fino()?;
        kontroli_ordon(&enskriboj)?;
        Ok(Self { enskriboj })
    }
}

fn kontroli_ordon(enskriboj: &[IndeksEnskribo]) -> Result<(), Eraro> {
    for paro in enskriboj.windows(2) {
        if let [unu, du] = paro
            && unu.vojo >= du.vojo
        {
            return Err(if unu.vojo == du.vojo {
                Eraro::DuplikataVojo {
                    nomo: unu.vojo.clone(),
                }
            } else {
                Eraro::NevalidaDatumo {
                    kialo: "neordigitaj indeksaj enskriboj",
                }
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod testoj {
    // PERMESO-JUSTIFIKO: nur testa kodo; panikigaj konstruaĵoj estas akcepteblaj.
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        reason = "testa kodo rajtas uzi panikigajn konstrukciojn"
    )]

    use super::{IndeksEnskribo, Indekso};
    use crate::eraroj::Eraro;
    use crate::stokado::haketi;
    use alloc::string::String;
    use alloc::vec;

    fn enskribo(vojo: &str, mtempo: u64) -> IndeksEnskribo {
        IndeksEnskribo {
            vojo: String::from(vojo),
            mtempo,
            grandeco: 7,
            haketo: haketi(vojo.as_bytes()),
        }
    }

    fn ekzemplo() -> Indekso {
        Indekso::el_enskriboj(vec![
            enskribo("src/lib.rs", 3),
            enskribo("README.md", 1),
            enskribo("src/a.rs", 2),
        ])
        .unwrap()
    }

    #[test]
    fn testo_indekso_ordigita_kaj_trovebla() {
        let indekso = ekzemplo();
        let vojoj: vec::Vec<&str> = indekso
            .enskriboj()
            .iter()
            .map(|e| e.vojo.as_str())
            .collect();
        assert_eq!(vojoj, ["README.md", "src/a.rs", "src/lib.rs"]);
        assert_eq!(indekso.trovi("src/a.rs").unwrap().mtempo, 2);
        assert!(indekso.trovi("mankanta").is_none());
    }

    #[test]
    fn testo_kaŝita_haketo_postulas_samajn_metadatumojn() {
        let indekso = ekzemplo();
        let haketo = haketi(b"src/a.rs");
        assert_eq!(indekso.kaŝita_haketo("src/a.rs", 2, 7), Some(haketo));
        assert_eq!(indekso.kaŝita_haketo("src/a.rs", 3, 7), None);
        assert_eq!(indekso.kaŝita_haketo("src/a.rs", 2, 8), None);
        assert_eq!(indekso.kaŝita_haketo("mankanta", 2, 7), None);
    }

    #[test]
    fn testo_indekso_seriigi_kaj_malseriigi() {
        let indekso = ekzemplo();
        let bajtoj = indekso.seriigi().unwrap();
        assert_eq!(Indekso::malseriigi(&bajtoj).unwrap(), indekso);

        let malplena = Indekso::default().seriigi().unwrap();
        assert_eq!(Indekso::malseriigi(&malplena).unwrap(), Indekso::default());
    }

    #[test]
    fn testo_indekso_rifuzas_difekton() {
        let mut bajtoj = ekzemplo().seriigi().unwrap();
        bajtoj[5] ^= 1;
        assert!(matches!(
            Indekso::malseriigi(&bajtoj),
            Err(Eraro::DatumoDifektita { .. })
        ));
        assert!(Indekso::malseriigi(&[]).is_err());
        assert!(Indekso::malseriigi(&bajtoj[..10]).is_err());
    }

    #[test]
    fn testo_indekso_rifuzas_malvalidajn_vojojn() {
        assert!(matches!(
            Indekso::el_enskriboj(vec![enskribo("a", 1), enskribo("a", 2)]),
            Err(Eraro::DuplikataVojo { .. })
        ));
        assert!(matches!(
            Indekso::el_enskriboj(vec![enskribo("../a", 1)]),
            Err(Eraro::NevalidaVojaKomponanto)
        ));
    }
}
