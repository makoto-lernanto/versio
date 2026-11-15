//! La datummodeloj de Versio: blob, arbo kaj komito.

use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::eraroj::Eraro;
use crate::stokado::{Haketo, Konservilo, haketi};

/// Blob: la kruda enhavo de unu dosiero.
///
/// Bloboj estas konservataj sen kroma kapsegigo, do ilia haketo egalas al la
/// BLAKE3-haketo de la dosiera enhavo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dosiero {
    enhavo: Vec<u8>,
}

impl Dosiero {
    /// Kreas blobon el la donita enhavo.
    pub fn nova(enhavo: impl Into<Vec<u8>>) -> Self {
        Self {
            enhavo: enhavo.into(),
        }
    }

    /// La kruda enhavo de la blobo.
    pub fn enhavo(&self) -> &[u8] {
        &self.enhavo
    }

    /// La enhavadresita haketo de la blobo.
    pub fn haketo(&self) -> Haketo {
        haketi(&self.enhavo)
    }

    /// Konservas la blobon en la donita stokejo.
    pub fn konservi(&self, stokejo: &impl Konservilo) -> Result<Haketo, Eraro> {
        stokejo.konservi(&self.enhavo)
    }

    /// Legas blobon el la donita stokejo, kontrolante ĝian haketon.
    pub fn ricevi(stokejo: &impl Konservilo, haketo: &Haketo) -> Result<Self, Eraro> {
        Ok(Self {
            enhavo: stokejo.ricevi(haketo)?,
        })
    }
}

/// Arbo: ordigita listo de dosieroj kaj subarboj.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Arbo {
    /// La enskriboj, ordigitaj laŭ nomo.
    pub enskriboj: Vec<ArbaEnskribo>,
}

/// Unu enskribo en arbo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn nomo(&self) -> &str {
        match self {
            Self::Dosiero { nomo, .. } | Self::SubArbo { nomo, .. } => nomo,
        }
    }
}

impl Arbo {
    /// Seriigas la arbon per postcard.
    pub fn seriigi(&self) -> Result<Vec<u8>, Eraro> {
        postcard::to_allocvec(self).map_err(Eraro::Seriigo)
    }

    /// Malseriigas arbon el postcard-bajtoj.
    pub fn malseriigi(datumoj: &[u8]) -> Result<Self, Eraro> {
        postcard::from_bytes(datumoj).map_err(Eraro::Malseriigo)
    }
}

/// Komito: momentfoto de la arbo, ligita al sia patro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Seriigas la komiton per postcard.
    pub fn seriigi(&self) -> Result<Vec<u8>, Eraro> {
        postcard::to_allocvec(self).map_err(Eraro::Seriigo)
    }

    /// Malseriigas komiton el postcard-bajtoj.
    pub fn malseriigi(datumoj: &[u8]) -> Result<Self, Eraro> {
        postcard::from_bytes(datumoj).map_err(Eraro::Malseriigo)
    }
}

#[cfg(test)]
mod testoj {
    use super::*;

    fn haketo(nombro: u8) -> Haketo {
        Haketo::el_bajtoj([nombro; 32])
    }

    #[test]
    fn testo_arbo_seriigi_kaj_malseriigi() {
        let arbo = Arbo {
            enskriboj: alloc::vec![
                ArbaEnskribo::Dosiero {
                    nomo: "README.md".into(),
                    haketo: haketo(1),
                    grandeco: 128,
                },
                ArbaEnskribo::SubArbo {
                    nomo: "src".into(),
                    haketo: haketo(2),
                },
            ],
        };

        let bajtoj = arbo.seriigi().unwrap();
        assert_eq!(Arbo::malseriigi(&bajtoj).unwrap(), arbo);
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
    }

    #[test]
    fn testo_dosiero_haketo() {
        let dosiero = Dosiero::nova(alloc::vec![1u8, 2, 3]);
        assert_eq!(dosiero.enhavo(), &[1, 2, 3]);
        assert_eq!(dosiero.haketo(), haketi(dosiero.enhavo()));
    }
}
