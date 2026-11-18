//! Limigita kaj determinisma legado de la `postcard`-formato.
//!
//! [`Skribilo`] spegulas la legadon kaj skribas la komitan kadron sen
//! `serde`, tiel ke la formato estas eksplicita kaj pruvebla per Kani. La
//! legado malĉifras unuopan skalaron per `postcard` (sen memoro-rezervado) kaj
//! kontrolas ĉiun longoprefikson antaŭ ol rezervi memoraĵon. Tiel malica enigo
//! ne povas kaŭzi grandegan rezervadon.

use alloc::string::String;
use alloc::vec::Vec;

use crate::eraroj::Eraro;
use crate::stokado::{HAKETA_LONGO, Haketo};

/// Skribilo por la kadro, kiun [`Legilo`] malĉifras.
pub(crate) struct Skribilo {
    bajtoj: Vec<u8>,
}

impl Skribilo {
    /// Kreas malplenan skribilon.
    pub(crate) fn nova() -> Self {
        Self { bajtoj: Vec::new() }
    }

    /// Aldonas unu `u8`-valoron.
    pub(crate) fn u8(&mut self, valoro: u8) {
        self.bajtoj.push(valoro);
    }

    /// Aldonas `u32`-valoron kiel varintan nombron.
    pub(crate) fn u32(&mut self, valoro: u32) {
        self.u64(u64::from(valoro));
    }

    /// Aldonas `u64`-valoron kiel varintan nombron.
    ///
    /// JUSTIFIKO: la varinto estas konstruata en loka tabelo kaj aldonata per
    /// unu `extend_from_slice`. La diagnozaj pruvoj mezuris, ke `Vec::push`-oj
    /// ene de buklo kun simbola ripetnombro multekostigas la formulon (3654
    /// restantaj kontroloj), dum la sama kadro tra `postcard`, kiu uzas ĉi
    /// tiun strukturon, restas ĉe 984. La tabelo havas 10 bajtojn (10 × 7 ≥
    /// 64 bitoj), do `get_mut` neniam revenas `None`.
    pub(crate) fn u64(&mut self, valoro: u64) {
        let mut buf = [0u8; 10];
        let mut longo = 0usize;
        let mut restanta = valoro;
        while longo < 10 {
            let malalta = (restanta & 0x7f).to_le_bytes()[0];
            restanta >>= 7;
            let bajto = if restanta == 0 {
                malalta
            } else {
                malalta | 0x80
            };
            if let Some(selo) = buf.get_mut(longo) {
                *selo = bajto;
            }
            longo = longo.saturating_add(1);
            if restanta == 0 {
                break;
            }
        }
        if let Some(uzo) = buf.get(..longo) {
            self.bajtoj.extend_from_slice(uzo);
        }
    }

    /// Aldonas haketon kiel fiksajn [`HAKETA_LONGO`] bajtojn.
    pub(crate) fn haketo(&mut self, haketo: &Haketo) {
        self.bajtoj.extend_from_slice(haketo.kiel_bajtoj());
    }

    /// Aldonas krudajn bajtojn.
    pub(crate) fn bajtoj(&mut self, bajtoj: &[u8]) {
        self.bajtoj.extend_from_slice(bajtoj);
    }

    /// Redonas la konstruitajn bajtojn.
    pub(crate) fn fino(self) -> Vec<u8> {
        self.bajtoj
    }
}

/// Minimuma kvanto da bajtoj por unu arba enskribo: speco (1), nomolongo (1),
/// nomo (≥1) kaj haketo (32).
const MINIMA_ENSKRIBO: usize = 35;

/// Legilo super krudaj bajtoj kun eksplicitaj limoj.
pub(crate) struct Legilo<'a> {
    restantaj: &'a [u8],
}

impl<'a> Legilo<'a> {
    /// Kreas legilon super la donitaj bajtoj.
    pub(crate) const fn nova(datumoj: &'a [u8]) -> Self {
        Self { restantaj: datumoj }
    }

    /// La nombro da ankoraŭ nelegitaj bajtoj.
    pub(crate) const fn longo(&self) -> usize {
        self.restantaj.len()
    }

    /// Legas unu `u8`-valoron.
    pub(crate) fn u8(&mut self) -> Result<u8, Eraro> {
        let (valoro, restantaj) =
            postcard::take_from_bytes::<u8>(self.restantaj).map_err(Eraro::Malseriigo)?;
        self.restantaj = restantaj;
        Ok(valoro)
    }

    /// Legas unu `u32`-varintan valoron.
    pub(crate) fn u32(&mut self) -> Result<u32, Eraro> {
        let (valoro, restantaj) =
            postcard::take_from_bytes::<u32>(self.restantaj).map_err(Eraro::Malseriigo)?;
        self.restantaj = restantaj;
        Ok(valoro)
    }

    /// Legas unu `u64`-varintan valoron.
    pub(crate) fn u64(&mut self) -> Result<u64, Eraro> {
        let (valoro, restantaj) =
            postcard::take_from_bytes::<u64>(self.restantaj).map_err(Eraro::Malseriigo)?;
        self.restantaj = restantaj;
        Ok(valoro)
    }

    /// Legas ekzakte `longo` bajtojn.
    pub(crate) fn bajtoj(&mut self, longo: usize) -> Result<&'a [u8], Eraro> {
        if self.restantaj.len() < longo {
            return Err(Eraro::NevalidaDatumo {
                kialo: "mankantaj bajtoj",
            });
        }
        let (unua, restantaj) = self.restantaj.split_at(longo);
        self.restantaj = restantaj;
        Ok(unua)
    }

    /// Legas unu haketon (32 fiksaj bajtoj).
    pub(crate) fn haketo(&mut self) -> Result<Haketo, Eraro> {
        let bajtoj = self.bajtoj(HAKETA_LONGO)?;
        let arangxo: [u8; HAKETA_LONGO] = bajtoj.try_into().map_err(|_| Eraro::NevalidaDatumo {
            kialo: "haketa longo maltrafas",
        })?;
        Ok(Haketo::el_bajtoj(arangxo))
    }

    /// Legas signoĉenon kun eksplicita supra limo por la longo.
    pub(crate) fn limigita_string(&mut self, maksimumo: usize) -> Result<String, Eraro> {
        let longo = self.u32()?;
        let longo = usize::try_from(longo).map_err(|_| Eraro::TrolongaEnigo {
            maksimumo,
            akirita: usize::MAX,
        })?;
        if longo > maksimumo {
            return Err(Eraro::TrolongaEnigo {
                maksimumo,
                akirita: longo,
            });
        }
        let bajtoj = self.bajtoj(longo)?;
        String::from_utf8(bajtoj.to_vec()).map_err(|_| Eraro::NevalidaDatumo {
            kialo: "nevalida UTF-8",
        })
    }

    /// Certigas, ke neniu bajto restas post la atendita enhavo.
    pub(crate) fn fino(&self) -> Result<(), Eraro> {
        if self.restantaj.is_empty() {
            Ok(())
        } else {
            Err(Eraro::NevalidaDatumo {
                kialo: "postrestaj bajtoj",
            })
        }
    }
}

/// Validigas la deklaritajn kvantojn antaŭ ol rezervi memoraĵon.
///
/// Laŭ la formato ĉiu enskribo konsumas almenaŭ [`MINIMA_ENSKRIBO`] bajtojn,
/// do la deklarita kvanto devas kongrui kun la restanta eniga longo.
pub(crate) fn limigi_kvanton(
    kvanto: u32,
    restantaj: usize,
    maksimumo: usize,
) -> Result<usize, Eraro> {
    let kvanto = usize::try_from(kvanto).map_err(|_| Eraro::TrolongaEnigo {
        maksimumo,
        akirita: usize::MAX,
    })?;
    if kvanto > maksimumo {
        return Err(Eraro::TrolongaEnigo {
            maksimumo,
            akirita: kvanto,
        });
    }
    if kvanto > restantaj / MINIMA_ENSKRIBO {
        return Err(Eraro::NevalidaDatumo {
            kialo: "la deklarita kvanto superas la restantajn bajtojn",
        });
    }
    Ok(kvanto)
}
