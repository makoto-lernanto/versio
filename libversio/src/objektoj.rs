//! La datummodeloj de Versio: blob, arbo kaj komito.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Serialize;

use crate::eraroj::Eraro;
use crate::kodigo::{Legilo, limigi_kvanton};
use crate::stokado::{Haketo, Konservilo, haketi};
use crate::vojo::{MAKSIMA_KOMPONANTO, validi_komponanton};

/// Maksimuma nombro da enskriboj en unu arbo.
pub const MAKSIMAJ_ENSKRIBOJ: usize = 4096;

/// Maksimuma longo de komita mesaĝo en bajtoj.
pub const MAKSIMA_MESAGXO: usize = 64 * 1024;

/// Blob: la kruda enhavo de unu dosiero.
///
/// Bloboj estas konservataj sen kroma kapsegigo, do ilia haketo egalas al la
/// `BLAKE3`-haketo de la dosiera enhavo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dosiero {
    enhavo: Vec<u8>,
}

impl Dosiero {
    /// Kreas blobon el la donita enhavo.
    #[must_use]
    pub fn nova(enhavo: impl Into<Vec<u8>>) -> Self {
        Self {
            enhavo: enhavo.into(),
        }
    }

    /// La kruda enhavo de la blobo.
    #[must_use]
    pub fn enhavo(&self) -> &[u8] {
        &self.enhavo
    }

    /// La enhavadresita haketo de la blobo.
    #[must_use]
    pub fn haketo(&self) -> Haketo {
        haketi(&self.enhavo)
    }

    /// Konservas la blobon en la donita stokejo.
    ///
    /// # Errors
    ///
    /// Redonas la eraron de la stokejo.
    pub fn konservi(&self, stokejo: &impl Konservilo) -> Result<Haketo, Eraro> {
        stokejo.konservi(&self.enhavo)
    }

    /// Legas blobon el la donita stokejo, kontrolante ĝian haketon.
    ///
    /// # Errors
    ///
    /// Redonas la eraron de la stokejo.
    pub fn ricevi(stokejo: &impl Konservilo, haketo: &Haketo) -> Result<Self, Eraro> {
        Ok(Self {
            enhavo: stokejo.ricevi(haketo)?,
        })
    }
}

/// Arbo: kanonike ordigita listo de dosieroj kaj subarboj.
///
/// La invarianto estas: enskriboj estas ordigitaj laŭ nomo, ĉiu nomo estas
/// valida voja komponanto, kaj neniu nomo ripetiĝas. Nur
/// [`Arbo::el_enskriboj`] kaj [`Arbo::malseriigi`] kreas arbojn.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Arbo {
    enskriboj: Vec<ArbaEnskribo>,
}

/// Unu enskribo en arbo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ArbaEnskribo {
    /// Regula dosiero, kies enhavo estas blobo en la stokejo.
    Dosiero {
        /// La nomo de la dosiero ene de la patra dosierujo.
        nomo: String,
        /// La haketo de la blobo.
        haketo: Haketo,
        /// La grandeco de la malkunpremita enhavo en bajtoj.
        grandeco: u64,
    },
    /// Subarbo, kies propra arbo-objekto estas en la stokejo.
    SubArbo {
        /// La nomo de la dosierujo.
        nomo: String,
        /// La haketo de la subarbo.
        haketo: Haketo,
    },
}

impl ArbaEnskribo {
    /// La nomo de la enskribo.
    #[must_use]
    pub fn nomo(&self) -> &str {
        match self {
            Self::Dosiero { nomo, .. } | Self::SubArbo { nomo, .. } => nomo,
        }
    }
}

impl Arbo {
    /// Kreas malplenan arbon.
    #[must_use]
    pub fn malplena() -> Self {
        Self::default()
    }

    /// Kreas arbon el la donitaj enskriboj.
    ///
    /// La enskriboj estas ordigitaj laŭ nomo; tiel la seriigita arbo estas
    /// sendependa de la eniga ordo.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::TrolongaEnigo`], se estas pli ol
    /// [`MAKSIMAJ_ENSKRIBOJ`] enskriboj, [`Eraro::NevalidaVojaKomponanto`],
    /// se iu nomo estas malvalida, aŭ [`Eraro::DuplikataVojo`], se du nomoj
    /// ripetiĝas.
    pub fn el_enskriboj(mut enskriboj: Vec<ArbaEnskribo>) -> Result<Self, Eraro> {
        if enskriboj.len() > MAKSIMAJ_ENSKRIBOJ {
            return Err(Eraro::TrolongaEnigo {
                maksimumo: MAKSIMAJ_ENSKRIBOJ,
                akirita: enskriboj.len(),
            });
        }
        for enskribo in &enskriboj {
            validi_komponanton(enskribo.nomo())?;
        }
        // La ordo inter egalaj nomoj ne gravas, ĉar egalaj nomoj estas rifuzataj
        // tuj poste; tial la stabila ordigo ne necesas.
        enskriboj.sort_unstable_by(|unu, du| unu.nomo().cmp(du.nomo()));
        for paro in enskriboj.windows(2) {
            if let [unu, du] = paro
                && unu.nomo() == du.nomo()
            {
                return Err(Eraro::DuplikataVojo {
                    nomo: String::from(unu.nomo()),
                });
            }
        }
        Ok(Self { enskriboj })
    }

    /// La kanonike ordigitaj enskriboj.
    #[must_use]
    pub fn enskriboj(&self) -> &[ArbaEnskribo] {
        &self.enskriboj
    }

    /// Seriigas la arbon per `postcard`.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::Seriigo`], se la seriigado malsukcesas.
    pub fn seriigi(&self) -> Result<Vec<u8>, Eraro> {
        postcard::to_allocvec(self).map_err(Eraro::Seriigo)
    }

    /// Malseriigas arbon el `postcard`-bajtoj.
    ///
    /// Ĉiu longoprefikso estas kontrolata antaŭ ol rezervi memoraĵon.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::Malseriigo`], [`Eraro::TrolongaEnigo`] aŭ
    /// [`Eraro::NevalidaDatumo`], se la bajtoj estas malvalidaj aŭ tro longaj.
    pub fn malseriigi(datumoj: &[u8]) -> Result<Self, Eraro> {
        let mut legilo = Legilo::nova(datumoj);
        let deklarita = legilo.u32()?;
        let kvanto = limigi_kvanton(deklarita, legilo.longo(), MAKSIMAJ_ENSKRIBOJ)?;
        let mut enskriboj = Vec::new();
        for _ in 0..kvanto {
            enskriboj.push(malseriigi_enskribon(&mut legilo)?);
        }
        legilo.fino()?;
        Self::el_enskriboj(enskriboj)
    }
}

fn malseriigi_enskribon(legilo: &mut Legilo<'_>) -> Result<ArbaEnskribo, Eraro> {
    match legilo.u32()? {
        0 => {
            let nomo = legilo.limigita_string(MAKSIMA_KOMPONANTO)?;
            let haketo = legilo.haketo()?;
            let grandeco = legilo.u64()?;
            Ok(ArbaEnskribo::Dosiero {
                nomo,
                haketo,
                grandeco,
            })
        }
        1 => {
            let nomo = legilo.limigita_string(MAKSIMA_KOMPONANTO)?;
            let haketo = legilo.haketo()?;
            Ok(ArbaEnskribo::SubArbo { nomo, haketo })
        }
        _ => Err(Eraro::NevalidaDatumo {
            kialo: "nekonata enskriba speco",
        }),
    }
}

/// Komito: momentfoto de la arbo, ligita al sia patro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Komito {
    /// La haketo de la radika arbo de la momentfoto.
    pub arbo: Haketo,
    /// La haketo de la patra komito, se ekzistas.
    pub gepatro: Option<Haketo>,
    /// La mesaĝo de la komito.
    pub mesagxo: String,
    /// Uniksa tempstampo de la kreo, en sekundoj.
    pub kreita_je: u64,
}

impl Komito {
    /// Seriigas la komiton per `postcard`.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::Seriigo`], se la seriigado malsukcesas.
    pub fn seriigi(&self) -> Result<Vec<u8>, Eraro> {
        postcard::to_allocvec(self).map_err(Eraro::Seriigo)
    }

    /// Malseriigas komiton el `postcard`-bajtoj.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::Malseriigo`], [`Eraro::TrolongaEnigo`] aŭ
    /// [`Eraro::NevalidaDatumo`], se la bajtoj estas malvalidaj aŭ tro longaj.
    pub fn malseriigi(datumoj: &[u8]) -> Result<Self, Eraro> {
        let mut legilo = Legilo::nova(datumoj);
        let arbo = legilo.haketo()?;
        let gepatro = match legilo.u8()? {
            0 => None,
            1 => Some(legilo.haketo()?),
            _ => {
                return Err(Eraro::NevalidaDatumo {
                    kialo: "nevalida gepatro-etikedo",
                });
            }
        };
        let mesagxo = legilo.limigita_string(MAKSIMA_MESAGXO)?;
        let kreita_je = legilo.u64()?;
        legilo.fino()?;
        Ok(Self {
            arbo,
            gepatro,
            mesagxo,
            kreita_je,
        })
    }
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

    use super::{ArbaEnskribo, Arbo, Dosiero, Komito, MAKSIMA_MESAGXO, MAKSIMAJ_ENSKRIBOJ};
    use crate::eraroj::Eraro;
    use crate::stokado::{Haketo, haketi};
    use alloc::string::String;
    use alloc::vec;

    fn haketo(nombro: u8) -> Haketo {
        Haketo::el_bajtoj([nombro; 32])
    }

    fn dosiero(nomo: &str, nombro: u8) -> ArbaEnskribo {
        ArbaEnskribo::Dosiero {
            nomo: String::from(nomo),
            haketo: haketo(nombro),
            grandeco: 128,
        }
    }

    #[test]
    fn testo_arbo_seriigi_kaj_malseriigi() {
        let arbo = Arbo::el_enskriboj(vec![
            dosiero("README.md", 1),
            ArbaEnskribo::SubArbo {
                nomo: "src".into(),
                haketo: haketo(2),
            },
        ])
        .unwrap();

        let bajtoj = arbo.seriigi().unwrap();
        assert_eq!(Arbo::malseriigi(&bajtoj).unwrap(), arbo);
    }

    #[test]
    fn testo_arbo_kanonika_sendepende_de_ordo() {
        let unua = Arbo::el_enskriboj(vec![dosiero("b", 2), dosiero("a", 1)]).unwrap();
        let dua = Arbo::el_enskriboj(vec![dosiero("a", 1), dosiero("b", 2)]).unwrap();

        assert_eq!(unua.seriigi().unwrap(), dua.seriigi().unwrap());
        assert_eq!(unua.enskriboj()[0].nomo(), "a");
    }

    #[test]
    fn testo_arbo_rifuzas_duplikatojn() {
        assert!(matches!(
            Arbo::el_enskriboj(vec![dosiero("a", 1), dosiero("a", 2)]),
            Err(Eraro::DuplikataVojo { .. })
        ));
    }

    #[test]
    fn testo_arbo_rifuzas_malvalidan_nomon() {
        assert!(matches!(
            Arbo::el_enskriboj(vec![dosiero("../fuĝo", 1)]),
            Err(Eraro::NevalidaVojaKomponanto)
        ));
    }

    #[test]
    fn testo_malseriigi_limigas_kvanton() {
        // Longa varint-prefikso por kvanto 1000, sekvata de neniom da enskriboj.
        let mut bajtoj = postcard::to_allocvec(&1000u32).unwrap();
        bajtoj.push(0);
        assert!(matches!(
            Arbo::malseriigi(&bajtoj),
            Err(Eraro::NevalidaDatumo { .. })
        ));

        // Tro granda kvanto por la difinita maksimumo.
        let troa =
            postcard::to_allocvec(&(u32::try_from(MAKSIMAJ_ENSKRIBOJ).unwrap() + 1)).unwrap();
        assert!(matches!(
            Arbo::malseriigi(&troa),
            Err(Eraro::TrolongaEnigo { .. })
        ));
    }

    #[test]
    fn testo_malseriigi_malplenan_arbon() {
        let arbo = Arbo::malseriigi(&[0]).unwrap();
        assert_eq!(arbo.enskriboj().len(), 0);
    }

    #[test]
    fn testo_malseriigi_rifuzas_postrestajn_bajtojn() {
        assert!(Arbo::malseriigi(&[0, 1]).is_err());
    }

    #[test]
    fn testo_komito_seriigi_kaj_malseriigi() {
        let komito = Komito {
            arbo: haketo(3),
            gepatro: Some(haketo(4)),
            mesagxo: "Unua komito".into(),
            kreita_je: 1_700_000_000,
        };

        let bajtoj = komito.seriigi().unwrap();
        assert_eq!(Komito::malseriigi(&bajtoj).unwrap(), komito);

        let sen_patro = Komito {
            gepatro: None,
            ..komito
        };
        let bajtoj = sen_patro.seriigi().unwrap();
        assert_eq!(Komito::malseriigi(&bajtoj).unwrap(), sen_patro);
    }

    #[test]
    fn testo_komito_rifuzas_tro_longan_mesaĝon() {
        let komito = Komito {
            arbo: haketo(3),
            gepatro: None,
            mesagxo: "a".repeat(MAKSIMA_MESAGXO + 1),
            kreita_je: 1,
        };
        let bajtoj = komito.seriigi().unwrap();
        assert!(matches!(
            Komito::malseriigi(&bajtoj),
            Err(Eraro::TrolongaEnigo { .. })
        ));
    }

    #[test]
    fn testo_dosiero_haketo() {
        let dosiero = Dosiero::nova(vec![1u8, 2, 3]);
        assert_eq!(dosiero.enhavo(), &[1, 2, 3]);
        assert_eq!(dosiero.haketo(), haketi(dosiero.enhavo()));
    }
}
