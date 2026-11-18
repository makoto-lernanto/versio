//! La datummodeloj de Versio: blob, arbo kaj komito.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Serialize;

use crate::eraroj::Eraro;
use crate::kodigo::{Legilo, Skribilo, limigi_kvanton};
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

/// Komito: momentfoto de la arbo, ligita al siaj gepatroj.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Komito {
    /// La haketo de la radika arbo de la momentfoto.
    pub arbo: Haketo,
    /// La haketo de la unua patra komito, se ekzistas.
    pub gepatro: Option<Haketo>,
    /// La mesaĝo de la komito.
    pub mesagxo: String,
    /// Uniksa tempstampo de la kreo, en sekundoj.
    pub kreita_je: u64,
    /// La dua patro de kunfanda komito; `None` por ordinaraj komitoj.
    pub dua_gepatro: Option<Haketo>,
}

impl Komito {
    /// Seriigas la komiton laŭ la sama kadro, kiun [`Komito::malseriigi`] legas.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::TrolongaEnigo`], se la mesaĝo ne kongruas kun `u32`.
    pub fn seriigi(&self) -> Result<Vec<u8>, Eraro> {
        let mut skribilo = Skribilo::nova();
        skribilo.haketo(&self.arbo);
        skribi_gepatron(&mut skribilo, self.gepatro.as_ref());
        let longo = u32::try_from(self.mesagxo.len()).map_err(|_| Eraro::TrolongaEnigo {
            maksimumo: MAKSIMA_MESAGXO,
            akirita: self.mesagxo.len(),
        })?;
        skribilo.u32(longo);
        skribilo.bajtoj(self.mesagxo.as_bytes());
        skribilo.u64(self.kreita_je);
        skribi_gepatron(&mut skribilo, self.dua_gepatro.as_ref());
        Ok(skribilo.fino())
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
        let dua_gepatro = match legilo.u8()? {
            0 => None,
            1 => Some(legilo.haketo()?),
            _ => {
                return Err(Eraro::NevalidaDatumo {
                    kialo: "nevalida dua-gepatro-etikedo",
                });
            }
        };
        if dua_gepatro.is_some() && gepatro.is_none() {
            return Err(Eraro::NevalidaDatumo {
                kialo: "dua gepatro sen unua",
            });
        }
        legilo.fino()?;
        Ok(Self {
            arbo,
            gepatro,
            mesagxo,
            kreita_je,
            dua_gepatro,
        })
    }
}

/// Skribas la etikedon kaj la bajtojn de laŭvola patro.
fn skribi_gepatron(skribilo: &mut Skribilo, gepatro: Option<&Haketo>) {
    match gepatro {
        None => skribilo.u8(0),
        Some(haketo) => {
            skribilo.u8(1);
            skribilo.haketo(haketo);
        }
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
    use alloc::vec::Vec;

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
            dua_gepatro: None,
        };

        let bajtoj = komito.seriigi().unwrap();
        assert_eq!(Komito::malseriigi(&bajtoj).unwrap(), komito);

        let kunfanda = Komito {
            dua_gepatro: Some(haketo(5)),
            ..komito.clone()
        };
        let bajtoj = kunfanda.seriigi().unwrap();
        assert_eq!(Komito::malseriigi(&bajtoj).unwrap(), kunfanda);

        let sen_patro = Komito {
            gepatro: None,
            dua_gepatro: None,
            ..komito
        };
        let bajtoj = sen_patro.seriigi().unwrap();
        assert_eq!(Komito::malseriigi(&bajtoj).unwrap(), sen_patro);
    }

    #[test]
    fn testo_komito_forpelas_duan_gepatron_sen_unua() {
        let mut bajtoj = Vec::new();
        bajtoj.extend_from_slice(&[7u8; 32]);
        bajtoj.extend_from_slice(&[0, 1, b'a', 0x80, 0x01]);
        bajtoj.push(1);
        bajtoj.extend_from_slice(&[8u8; 32]);
        assert!(matches!(
            Komito::malseriigi(&bajtoj),
            Err(Eraro::NevalidaDatumo { .. })
        ));
    }

    #[test]
    fn testo_komito_rifuzas_tro_longan_mesaĝon() {
        let komito = Komito {
            arbo: haketo(3),
            gepatro: None,
            mesagxo: "a".repeat(MAKSIMA_MESAGXO + 1),
            kreita_je: 1,
            dua_gepatro: None,
        };
        let bajtoj = komito.seriigi().unwrap();
        assert!(matches!(
            Komito::malseriigi(&bajtoj),
            Err(Eraro::TrolongaEnigo { .. })
        ));
    }

    #[test]
    fn testo_komito_rondiro_limoj() {
        for gepatro in [None, Some(haketo(4))] {
            for dua_gepatro in [None, gepatro.map(|_| haketo(5))] {
                for mesagxo in [
                    String::new(),
                    String::from("a\0é𐐀"),
                    "a".repeat(MAKSIMA_MESAGXO),
                ] {
                    for kreita_je in [0, 127, 128, u64::MAX] {
                        let komito = Komito {
                            arbo: haketo(3),
                            gepatro,
                            mesagxo: mesagxo.clone(),
                            kreita_je,
                            dua_gepatro,
                        };
                        let bajtoj = komito.seriigi().unwrap();
                        assert_eq!(Komito::malseriigi(&bajtoj).unwrap(), komito);
                    }
                }
            }
        }
    }

    #[test]
    fn testo_komito_rifuzas_trancxojn_kaj_postrestojn() {
        for gepatro in [None, Some(haketo(4))] {
            for dua_gepatro in [None, gepatro.map(|_| haketo(5))] {
                let komito = Komito {
                    arbo: haketo(3),
                    gepatro,
                    mesagxo: String::from("mesaĝo"),
                    kreita_je: u64::MAX,
                    dua_gepatro,
                };
                let mut bajtoj = komito.seriigi().unwrap();
                for longo in 0..bajtoj.len() {
                    assert!(Komito::malseriigi(&bajtoj[..longo]).is_err());
                }
                assert_eq!(Komito::malseriigi(&bajtoj).unwrap(), komito);
                bajtoj.push(0);
                assert!(Komito::malseriigi(&bajtoj).is_err());
            }
        }
    }

    #[test]
    fn testo_komito_rifuzas_nevalidajn_etikedojn() {
        let komito = Komito {
            arbo: haketo(3),
            gepatro: None,
            mesagxo: String::from("a"),
            kreita_je: 128,
            dua_gepatro: None,
        };
        let mut bajtoj = komito.seriigi().unwrap();
        for etikedo in 0..=u8::MAX {
            bajtoj[crate::stokado::HAKETA_LONGO] = etikedo;
            for longo in 0..=bajtoj.len() {
                assert_eq!(
                    Komito::malseriigi(&bajtoj[..longo]).is_ok(),
                    etikedo == 0 && longo == bajtoj.len()
                );
            }
        }
    }

    #[test]
    fn testo_komito_rifuzas_nevalidan_duan_etikedon() {
        let komito = Komito {
            arbo: haketo(3),
            gepatro: Some(haketo(4)),
            mesagxo: String::from("a"),
            kreita_je: 128,
            dua_gepatro: None,
        };
        let mut bajtoj = komito.seriigi().unwrap();
        let lasta = bajtoj.len() - 1;
        bajtoj[lasta] = 2;
        assert!(matches!(
            Komito::malseriigi(&bajtoj),
            Err(Eraro::NevalidaDatumo { .. })
        ));
    }

    #[test]
    fn testo_komito_fiksa_kodajo() {
        let komito = Komito {
            arbo: haketo(7),
            gepatro: None,
            mesagxo: String::from("a"),
            kreita_je: 128,
            dua_gepatro: None,
        };
        let mut atendita = Vec::new();
        atendita.extend_from_slice(&[7u8; 32]);
        atendita.extend_from_slice(&[0, 1, b'a', 0x80, 0x01, 0]);
        assert_eq!(komito.seriigi().unwrap(), atendita);
    }

    #[test]
    fn testo_komito_fiksa_kunfanda_kodajo() {
        let komito = Komito {
            arbo: haketo(7),
            gepatro: Some(haketo(8)),
            mesagxo: String::new(),
            kreita_je: 0,
            dua_gepatro: Some(haketo(9)),
        };
        let mut atendita = Vec::new();
        atendita.extend_from_slice(&[7u8; 32]);
        atendita.extend_from_slice(&[1]);
        atendita.extend_from_slice(&[8u8; 32]);
        atendita.extend_from_slice(&[0, 0]); // malplena mesaĝo, tempstampo 0
        atendita.push(1);
        atendita.extend_from_slice(&[9u8; 32]);
        assert_eq!(komito.seriigi().unwrap(), atendita);
    }

    #[test]
    fn testo_komito_akceptas_fiksan_tempstampan_varinton() {
        for kreita_je in [0, 1, 127, 128, u64::MAX] {
            let mut bajtoj = Vec::new();
            bajtoj.extend_from_slice(&[7u8; 32]);
            bajtoj.extend_from_slice(&[0, 1, b'a']);
            for indekso in 0..9 {
                bajtoj.push((kreita_je >> (7 * indekso)) as u8 & 0x7f | 0x80);
            }
            bajtoj.push((kreita_je >> 63) as u8 & 0x01);
            bajtoj.push(0);

            let komito = Komito::malseriigi(&bajtoj).unwrap();
            assert_eq!(komito.kreita_je, kreita_je);
            assert_eq!(komito.mesagxo, "a");
            assert_eq!(komito.arbo, haketo(7));
            assert_eq!(komito.gepatro, None);
        }
    }

    #[test]
    fn testo_dosiero_haketo() {
        let dosiero = Dosiero::nova(vec![1u8, 2, 3]);
        assert_eq!(dosiero.enhavo(), &[1, 2, 3]);
        assert_eq!(dosiero.haketo(), haketi(dosiero.enhavo()));
    }
}
