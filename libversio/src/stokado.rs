//! Enhavadresita stokado (CAS): `konservi` kaj `ricevi`.
//!
//! Ĉiu objekto estas identigita per la BLAKE3-haketo de siaj malkunpremitaj
//! datumoj. La dosiersistema realigo (`DosieraKonservilo`) estas ŝaltata per
//! la trajto `std`; la haketa logiko mem restas `no_std`-kongrueca.

use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::eraroj::Eraro;

/// Longo de BLAKE3-haketo en bajtoj.
pub const HAKETA_LONGO: usize = 32;

const DEKSESUMO: &[u8; 16] = b"0123456789abcdef";

/// Enhavadresita identigilo: la BLAKE3-haketo de objekto.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Haketo([u8; HAKETA_LONGO]);

impl Haketo {
    /// Kreas haketon el krudaj bajtoj.
    pub const fn el_bajtoj(bajtoj: [u8; HAKETA_LONGO]) -> Self {
        Self(bajtoj)
    }

    /// Redonas la krudajn bajtojn de la haketo.
    pub const fn kiel_bajtoj(&self) -> &[u8; HAKETA_LONGO] {
        &self.0
    }
}

impl fmt::Display for Haketo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut teksto = [0u8; HAKETA_LONGO * 2];
        for (indekso, bajto) in self.0.iter().enumerate() {
            teksto[2 * indekso] = DEKSESUMO[(bajto >> 4) as usize];
            teksto[2 * indekso + 1] = DEKSESUMO[(bajto & 0x0f) as usize];
        }
        f.write_str(core::str::from_utf8(&teksto).expect("deksesumaj bajtoj estas UTF-8"))
    }
}

impl fmt::Debug for Haketo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Haketo({self})")
    }
}

impl FromStr for Haketo {
    type Err = Eraro;

    fn from_str(teksto: &str) -> Result<Self, Self::Err> {
        let signoj = teksto.as_bytes();
        if signoj.len() != HAKETA_LONGO * 2 {
            return Err(Eraro::NevalidaHaketo);
        }

        let mut bajtoj = [0u8; HAKETA_LONGO];
        for (indekso, bajto) in bajtoj.iter_mut().enumerate() {
            let alta = de_hex(signoj[2 * indekso]).ok_or(Eraro::NevalidaHaketo)?;
            let malalta = de_hex(signoj[2 * indekso + 1]).ok_or(Eraro::NevalidaHaketo)?;
            *bajto = (alta << 4) | malalta;
        }
        Ok(Self(bajtoj))
    }
}

const fn de_hex(signo: u8) -> Option<u8> {
    match signo {
        b'0'..=b'9' => Some(signo - b'0'),
        b'a'..=b'f' => Some(signo - b'a' + 10),
        b'A'..=b'F' => Some(signo - b'A' + 10),
        _ => None,
    }
}

/// Kalkulas la BLAKE3-haketon de la donitaj datumoj.
pub fn haketi(datumoj: &[u8]) -> Haketo {
    Haketo(*blake3::hash(datumoj).as_bytes())
}

/// Kontrolas, ĉu la datumoj kongruas kun la atendita haketo.
pub fn kontroli(datumoj: &[u8], atendita: &Haketo) -> Result<(), Eraro> {
    let akirita = haketi(datumoj);
    if akirita == *atendita {
        Ok(())
    } else {
        Err(Eraro::DatumoDifektita {
            atendita: *atendita,
            akirita,
        })
    }
}

/// Abstraktaĵo por enhavadresita stokado.
pub trait Konservilo {
    /// Konservas la datumojn kaj redonas ilian haketon.
    fn konservi(&self, datumoj: &[u8]) -> Result<Haketo, Eraro>;

    /// Legas la objekton, kontrolas ĝian haketon kaj redonas la datumojn.
    fn ricevi(&self, haketo: &Haketo) -> Result<Vec<u8>, Eraro>;
}

#[cfg(feature = "std")]
mod dosiera {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    const KUNPREMA_NIVELO: i32 = 3;

    static TEMP_NOMBRO: AtomicU64 = AtomicU64::new(0);

    /// `Konservilo` bazita sur la loka dosiersistemo.
    ///
    /// Ĉiu objekto estas kunpremita per zstd kaj skribita al
    /// `<dosierujo>/<haketo>.zst`. Skribo okazas al provizora dosiero,
    /// kiun oni poste renomas atome, por eviti duonfinajn objektojn.
    pub struct DosieraKonservilo {
        dosierujo: PathBuf,
    }

    impl DosieraKonservilo {
        /// Kreas konservilon direktitan al la donita dosierujo.
        pub fn nova(dosierujo: impl Into<PathBuf>) -> Self {
            Self {
                dosierujo: dosierujo.into(),
            }
        }

        /// La dosierujo, en kiu la objektoj estas konservataj.
        pub fn dosierujo(&self) -> &Path {
            &self.dosierujo
        }

        fn vojo(&self, haketo: &Haketo) -> PathBuf {
            self.dosierujo.join(format!("{haketo}.zst"))
        }

        fn temp_vojo(&self, haketo: &Haketo) -> PathBuf {
            let nombro = TEMP_NOMBRO.fetch_add(1, Ordering::Relaxed);
            self.dosierujo
                .join(format!(".{haketo}.{}.{nombro}.tmp", std::process::id()))
        }
    }

    impl Konservilo for DosieraKonservilo {
        fn konservi(&self, datumoj: &[u8]) -> Result<Haketo, Eraro> {
            let haketo = haketi(datumoj);
            let celo = self.vojo(&haketo);
            if celo.exists() {
                return Ok(haketo);
            }

            fs::create_dir_all(&self.dosierujo)?;
            let kunpremita = zstd::stream::encode_all(datumoj, KUNPREMA_NIVELO)?;

            let provizora = self.temp_vojo(&haketo);
            fs::write(&provizora, kunpremita)?;
            if let Err(eraro) = fs::rename(&provizora, &celo) {
                let _ = fs::remove_file(&provizora);
                return Err(eraro.into());
            }
            Ok(haketo)
        }

        fn ricevi(&self, haketo: &Haketo) -> Result<Vec<u8>, Eraro> {
            let kunpremita = match fs::read(self.vojo(haketo)) {
                Ok(datumoj) => datumoj,
                Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => {
                    return Err(Eraro::NeekzistantaObjekto { haketo: *haketo });
                }
                Err(eraro) => return Err(eraro.into()),
            };

            let datumoj = zstd::stream::decode_all(kunpremita.as_slice())?;
            kontroli(&datumoj, haketo)?;
            Ok(datumoj)
        }
    }
}

#[cfg(feature = "std")]
pub use dosiera::DosieraKonservilo;

#[cfg(all(test, feature = "std"))]
mod testoj {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static PROVOJ: AtomicU64 = AtomicU64::new(0);

    struct ProvDosierujo {
        vojo: PathBuf,
    }

    impl ProvDosierujo {
        fn nova(etikedo: &str) -> Self {
            let nombro = PROVOJ.fetch_add(1, Ordering::Relaxed);
            let vojo = std::env::temp_dir().join(format!(
                "versio-testo-{etikedo}-{}-{nombro}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&vojo);
            std::fs::create_dir_all(&vojo).expect("krei provan dosierujon");
            Self { vojo }
        }
    }

    impl Drop for ProvDosierujo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.vojo);
        }
    }

    fn hazardaj_bajtoj(longo: usize, semo: u64) -> Vec<u8> {
        let mut stato = semo | 1;
        (0..longo)
            .map(|_| {
                stato ^= stato << 13;
                stato ^= stato >> 7;
                stato ^= stato << 17;
                (stato & 0xff) as u8
            })
            .collect()
    }

    #[test]
    fn testo_konservi_kaj_ricevi() {
        let prov = ProvDosierujo::nova("konservi");
        let stokejo = DosieraKonservilo::nova(&prov.vojo);
        let datumoj = hazardaj_bajtoj(1 << 20, 0x5eed_1234_9abc_def1);

        let haketo = stokejo.konservi(&datumoj).unwrap();
        assert_eq!(haketo, haketi(&datumoj));
        assert_eq!(haketo.to_string().len(), HAKETA_LONGO * 2);

        let dosiero = prov.vojo.join(format!("{haketo}.zst"));
        assert!(dosiero.is_file(), "la kunpremita objekto devas ekzisti");

        let legita = stokejo.ricevi(&haketo).unwrap();
        assert_eq!(legita, datumoj, "legitaj datumoj devas esti identaj");
    }

    #[test]
    fn testo_ricevi_difektitan_objekton() {
        let prov = ProvDosierujo::nova("difektita");
        let stokejo = DosieraKonservilo::nova(&prov.vojo);
        let atendita = haketi(b"originalaj datumoj");

        let kunpremita = zstd::stream::encode_all(&b"aliaj datumoj"[..], 3).unwrap();
        std::fs::write(prov.vojo.join(format!("{atendita}.zst")), kunpremita).unwrap();

        match stokejo.ricevi(&atendita) {
            Err(Eraro::DatumoDifektita {
                atendita: a,
                akirita,
            }) => {
                assert_eq!(a, atendita);
                assert_eq!(akirita, haketi(b"aliaj datumoj"));
            }
            alia => panic!("atendita DatumoDifektita, ricevita: {alia:?}"),
        }
    }

    #[test]
    fn testo_ricevi_neekzistantan() {
        let prov = ProvDosierujo::nova("mankanta");
        let stokejo = DosieraKonservilo::nova(&prov.vojo);
        let haketo = haketi(b"nenio");

        assert!(matches!(
            stokejo.ricevi(&haketo),
            Err(Eraro::NeekzistantaObjekto { haketo: h }) if h == haketo
        ));
    }

    #[test]
    fn testo_haketo_teksto() {
        let haketo = haketi(b"saluton, mondo");
        let teksto = haketo.to_string();

        assert_eq!(teksto.len(), HAKETA_LONGO * 2);
        assert_eq!(teksto.parse::<Haketo>().unwrap(), haketo);
        assert!("nevalida".parse::<Haketo>().is_err());
        assert!("g".repeat(64).parse::<Haketo>().is_err());
    }
}
