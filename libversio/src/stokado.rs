//! Enhavadresita stokado (EAS): `konservi` kaj `ricevi`.
//!
//! Ĉiu objekto estas identigita per la `BLAKE3`-haketo de siaj malkunpremitaj
//! datumoj. La trajto [`Konservilo`] kaj la enmemora [`MemoraKonservilo`]
//! funkcias sen `std`; la dosiersistema [`DosieraKonservilo`] estas ŝaltata
//! per la trajto `std`.

use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt;
use core::fmt::Write as _;
use core::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::eraroj::Eraro;

/// Longo de `BLAKE3`-haketo en bajtoj.
pub const HAKETA_LONGO: usize = 32;

const DEKSESUMO: &[u8; 16] = b"0123456789abcdef";

/// Enhavadresita identigilo: la `BLAKE3`-haketo de objekto.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Haketo([u8; HAKETA_LONGO]);

impl Haketo {
    /// Kreas haketon el krudaj bajtoj.
    #[must_use]
    pub const fn el_bajtoj(bajtoj: [u8; HAKETA_LONGO]) -> Self {
        Self(bajtoj)
    }

    /// Redonas la krudajn bajtojn de la haketo.
    #[must_use]
    pub const fn kiel_bajtoj(&self) -> &[u8; HAKETA_LONGO] {
        &self.0
    }
}

impl fmt::Display for Haketo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &bajto in &self.0 {
            for duono in [bajto.checked_shr(4).unwrap_or(0), bajto & 0x0f] {
                let indekso = usize::from(duono);
                let signo = DEKSESUMO.get(indekso).copied().unwrap_or(b'0');
                f.write_char(char::from(signo))?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Haketo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Haketo({self})")
    }
}

impl FromStr for Haketo {
    type Err = Eraro;

    /// Analizas deksesuman haketon.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaHaketo`], se la longo ne estas 64 aŭ se iu
    /// signo ne estas deksesuma cifero.
    fn from_str(teksto: &str) -> Result<Self, Self::Err> {
        let (paroj, resto) = teksto.as_bytes().as_chunks::<2>();
        if !resto.is_empty() || paroj.len() != HAKETA_LONGO {
            return Err(Eraro::NevalidaHaketo);
        }

        let mut bajtoj = [0u8; HAKETA_LONGO];
        for (bajto, &[alta, malalta]) in bajtoj.iter_mut().zip(paroj) {
            let alta = de_hex(alta).ok_or(Eraro::NevalidaHaketo)?;
            let malalta = de_hex(malalta).ok_or(Eraro::NevalidaHaketo)?;
            *bajto = alta.wrapping_shl(4) | malalta;
        }
        Ok(Self(bajtoj))
    }
}

const fn de_hex(signo: u8) -> Option<u8> {
    match signo {
        b'0'..=b'9' => signo.checked_sub(b'0'),
        b'a'..=b'f' => match signo.checked_sub(b'a') {
            Some(valoro) => valoro.checked_add(10),
            None => None,
        },
        b'A'..=b'F' => match signo.checked_sub(b'A') {
            Some(valoro) => valoro.checked_add(10),
            None => None,
        },
        _ => None,
    }
}

/// Kalkulas la `BLAKE3`-haketon de la donitaj datumoj.
#[must_use]
pub fn haketi(datumoj: &[u8]) -> Haketo {
    Haketo(*blake3::hash(datumoj).as_bytes())
}

/// Kontrolas, ĉu la datumoj kongruas kun la atendita haketo.
///
/// # Errors
///
/// Redonas [`Eraro::DatumoDifektita`], se la haketo de la datumoj malsamas
/// de la atendita.
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
    ///
    /// # Errors
    ///
    /// Redonas la eraron de la suba stokejo.
    fn konservi(&self, datumoj: &[u8]) -> Result<Haketo, Eraro>;

    /// Legas la objekton, kontrolas ĝian haketon kaj redonas la datumojn.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NeekzistantaObjekto`], se la objekto mankas, aŭ
    /// [`Eraro::DatumoDifektita`], se la enhavo ne kongruas kun la haketo.
    fn ricevi(&self, haketo: &Haketo) -> Result<Vec<u8>, Eraro>;
}

/// Konservilo, kiu tenas la objektojn en memoro.
///
/// Ĝi funkcias sen `std` kaj servas al testoj, pruvoj kaj al la estonta
/// enmemora reĝimo. La legado ĉiam kontrolas la haketon.
#[derive(Debug, Default)]
pub struct MemoraKonservilo {
    objektoj: RefCell<Vec<(Haketo, Vec<u8>)>>,
}

impl MemoraKonservilo {
    /// Kreas malplenan enmemoran konservilon.
    #[must_use]
    pub fn nova() -> Self {
        Self::default()
    }

    /// Enigas krudajn datumojn sub la donita haketo sen kontrolo.
    ///
    /// Tio ebligas simuli difektitan stokejon en testoj kaj pruvoj.
    pub fn enigi_krudajn(&self, haketo: Haketo, datumoj: Vec<u8>) {
        let mut objektoj = self.objektoj.borrow_mut();
        if let Some((_, ekzistanta)) = objektoj.iter_mut().find(|(h, _)| *h == haketo) {
            *ekzistanta = datumoj;
        } else {
            objektoj.push((haketo, datumoj));
        }
    }

    /// La nombro da konservitaj objektoj.
    #[must_use]
    pub fn n_objektoj(&self) -> usize {
        self.objektoj.borrow().len()
    }
}

impl Konservilo for MemoraKonservilo {
    fn konservi(&self, datumoj: &[u8]) -> Result<Haketo, Eraro> {
        let haketo = haketi(datumoj);
        self.enigi_krudajn(haketo, datumoj.to_vec());
        Ok(haketo)
    }

    fn ricevi(&self, haketo: &Haketo) -> Result<Vec<u8>, Eraro> {
        let objektoj = self.objektoj.borrow();
        let (_, datumoj) = objektoj
            .iter()
            .find(|(h, _)| h == haketo)
            .ok_or(Eraro::NeekzistantaObjekto { haketo: *haketo })?;
        kontroli(datumoj, haketo)?;
        Ok(datumoj.clone())
    }
}

#[cfg(feature = "std")]
mod dosiera {
    use super::{Eraro, Haketo, Konservilo, kontroli};
    use alloc::vec::Vec;
    use std::fs;
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    const LEGA_BLOKO: usize = 64 * 1024;

    static TEMP_NOMBRO: AtomicU64 = AtomicU64::new(0);

    /// Forigas la provizoran dosieron, kiam la operacio finiĝas aŭ malsukcesas.
    struct TempForigilo<'a>(&'a Path);

    impl Drop for TempForigilo<'_> {
        fn drop(&mut self) {
            drop(fs::remove_file(self.0));
        }
    }

    /// [`Konservilo`] bazita sur la loka dosiersistemo.
    ///
    /// Ĉiu objekto estas kunpremita per `lz4` kaj skribita al
    /// `<dosierujo>/<haketo>.lz4`. Skribo okazas al provizora dosiero,
    /// kiun oni poste renomas atome, por eviti duonfinajn objektojn.
    pub struct DosieraKonservilo {
        dosierujo: PathBuf,
    }

    impl DosieraKonservilo {
        /// Kreas konservilon direktitan al la donita dosierujo.
        #[must_use]
        pub fn nova(dosierujo: impl Into<PathBuf>) -> Self {
            Self {
                dosierujo: dosierujo.into(),
            }
        }

        /// La dosierujo, en kiu la objektoj estas konservataj.
        #[must_use]
        pub fn dosierujo(&self) -> &Path {
            &self.dosierujo
        }

        /// Ĉu la objekto kun la donita haketo jam ekzistas en la stokejo.
        #[must_use]
        pub fn havas(&self, haketo: &Haketo) -> bool {
            self.vojo(haketo).is_file()
        }

        /// Listigas la haketojn de ĉiuj konservitaj objektoj, ordigitajn.
        ///
        /// # Errors
        ///
        /// Redonas dosiersisteman eraron, se la objekta dosierujo ne legeblas.
        pub fn haketoj(&self) -> Result<Vec<Haketo>, Eraro> {
            let mut haketoj = Vec::new();
            for eniro in fs::read_dir(&self.dosierujo)? {
                let eniro = eniro?;
                let nomo = eniro.file_name();
                let Some(tigo) = nomo.to_str().and_then(|nomo| nomo.strip_suffix(".lz4")) else {
                    continue;
                };
                if let Ok(haketo) = tigo.parse() {
                    haketoj.push(haketo);
                }
            }
            haketoj.sort_unstable();
            Ok(haketoj)
        }

        fn vojo(&self, haketo: &Haketo) -> PathBuf {
            self.dosierujo.join(format!("{haketo}.lz4"))
        }

        fn temp_vojo(&self) -> PathBuf {
            let nombro = TEMP_NOMBRO.fetch_add(1, Ordering::Relaxed);
            self.dosierujo
                .join(format!(".{}.{}.tmp", std::process::id(), nombro))
        }

        /// Legas dosieron kontinuante, kunpremas ĝin kaj konservas ĝin.
        ///
        /// La enhavo estas legata po 64 KB, do grandaj dosieroj ne devas
        /// esti tute ŝarĝataj en memoro.
        ///
        /// # Errors
        ///
        /// Redonas dosiersisteman eraron, se la dosiero ne legeblas aŭ la
        /// stokejo ne skribeblas.
        pub fn konservi_dosieron(&self, vojo: &Path) -> Result<(Haketo, u64), Eraro> {
            let dosiero = fs::File::open(vojo)?;
            self.konservi_fluon(dosiero)
        }

        /// Konservas fluon de datumoj kaj redonas la haketon kun la grandeco.
        ///
        /// # Errors
        ///
        /// Redonas dosiersisteman eraron, se legado aŭ skribado malsukcesas.
        pub fn konservi_fluon<R: Read>(&self, mut legilo: R) -> Result<(Haketo, u64), Eraro> {
            fs::create_dir_all(&self.dosierujo)?;

            let provizora = self.temp_vojo();
            let forigilo = TempForigilo(&provizora);

            let dosiero = fs::File::create(&provizora)?;
            let mut kodilo = lz4_flex::frame::FrameEncoder::new(dosiero);
            let mut haketilo = blake3::Hasher::new();
            let mut bufujo = vec![0u8; LEGA_BLOKO];
            let mut grandeco: u64 = 0;

            loop {
                let legita = legilo.read(&mut bufujo)?;
                if legita == 0 {
                    break;
                }
                let ero = bufujo.get(..legita).ok_or(Eraro::NevalidaDatumo {
                    kialo: "legita longo superas la bufujon",
                })?;
                haketilo.update(ero);
                kodilo.write_all(ero)?;
                grandeco = grandeco.saturating_add(legita as u64);
            }

            kodilo.finish().map_err(std::io::Error::from)?.sync_all()?;
            let haketo = Haketo(*haketilo.finalize().as_bytes());
            let celo = self.vojo(&haketo);

            if celo.exists() {
                drop(forigilo);
                return Ok((haketo, grandeco));
            }
            fs::rename(&provizora, &celo)?;
            drop(forigilo);
            Ok((haketo, grandeco))
        }
    }

    /// Kalkulas la haketon kaj grandecon de dosiero sen konservi ĝin.
    ///
    /// La enhavo estas legata po 64 KB.
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman eraron, se la dosiero ne legeblas.
    pub fn haketi_dosieron(vojo: &Path) -> Result<(Haketo, u64), Eraro> {
        let mut dosiero = fs::File::open(vojo)?;
        let mut haketilo = blake3::Hasher::new();
        let mut bufujo = vec![0u8; LEGA_BLOKO];
        let mut grandeco: u64 = 0;
        loop {
            let legita = dosiero.read(&mut bufujo)?;
            if legita == 0 {
                break;
            }
            let ero = bufujo.get(..legita).ok_or(Eraro::NevalidaDatumo {
                kialo: "legita longo superas la bufujon",
            })?;
            haketilo.update(ero);
            grandeco = grandeco.saturating_add(legita as u64);
        }
        Ok((Haketo(*haketilo.finalize().as_bytes()), grandeco))
    }

    impl Konservilo for DosieraKonservilo {
        fn konservi(&self, datumoj: &[u8]) -> Result<Haketo, Eraro> {
            // La haketo estas kalkulata antaŭ la kunpremado: se la objekto jam
            // ekzistas, neniu provizora dosiero estas skribata nek sinkronigata.
            let haketo = super::haketi(datumoj);
            if self.havas(&haketo) {
                return Ok(haketo);
            }
            self.konservi_fluon(datumoj).map(|(haketo, _)| haketo)
        }

        fn ricevi(&self, haketo: &Haketo) -> Result<Vec<u8>, Eraro> {
            let dosiero = match fs::File::open(self.vojo(haketo)) {
                Ok(dosiero) => dosiero,
                Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => {
                    return Err(Eraro::NeekzistantaObjekto { haketo: *haketo });
                }
                Err(eraro) => return Err(eraro.into()),
            };

            let mut datumoj = Vec::new();
            lz4_flex::frame::FrameDecoder::new(dosiero).read_to_end(&mut datumoj)?;
            kontroli(&datumoj, haketo)?;
            Ok(datumoj)
        }
    }
}

#[cfg(feature = "std")]
pub use dosiera::{DosieraKonservilo, haketi_dosieron};

#[cfg(all(test, feature = "std"))]
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

    use super::{
        DosieraKonservilo, Eraro, HAKETA_LONGO, Haketo, Konservilo, MemoraKonservilo, haketi,
        haketi_dosieron,
    };
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
            drop(std::fs::remove_dir_all(&vojo));
            std::fs::create_dir_all(&vojo).expect("krei provan dosierujon");
            Self { vojo }
        }
    }

    impl Drop for ProvDosierujo {
        fn drop(&mut self) {
            drop(std::fs::remove_dir_all(&self.vojo));
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

        let dosiero = prov.vojo.join(format!("{haketo}.lz4"));
        assert!(dosiero.is_file(), "la kunpremita objekto devas ekzisti");

        let legita = stokejo.ricevi(&haketo).unwrap();
        assert_eq!(legita, datumoj, "legitaj datumoj devas esti identaj");
    }

    #[test]
    fn testo_ricevi_difektitan_objekton() {
        let prov = ProvDosierujo::nova("difektita");
        let stokejo = DosieraKonservilo::nova(&prov.vojo);
        let atendita = haketi(b"originalaj datumoj");
        let alia = stokejo.konservi(b"aliaj datumoj").unwrap();

        let fonto = prov.vojo.join(format!("{alia}.lz4"));
        let celo = prov.vojo.join(format!("{atendita}.lz4"));
        std::fs::rename(fonto, celo).unwrap();

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
        assert!(teksto.to_uppercase().parse::<Haketo>().is_ok());
    }

    #[test]
    fn testo_konservi_dosieron() {
        let prov = ProvDosierujo::nova("fluo");
        let stokejo = DosieraKonservilo::nova(&prov.vojo);
        let datumoj = hazardaj_bajtoj(2 << 20, 0x9e37_79b9_7f4a_7c15);
        let vojo = prov.vojo.join("granda.bin");
        std::fs::write(&vojo, &datumoj).unwrap();

        let (haketo, grandeco) = stokejo.konservi_dosieron(&vojo).unwrap();
        assert_eq!(grandeco, datumoj.len() as u64);
        assert_eq!(haketo, haketi(&datumoj));
        assert_eq!(stokejo.ricevi(&haketo).unwrap(), datumoj);
        assert_eq!(haketi_dosieron(&vojo).unwrap(), (haketo, grandeco));
    }

    #[test]
    fn testo_konservi_fluon_sen_duobligo() {
        let prov = ProvDosierujo::nova("duobligo");
        let stokejo = DosieraKonservilo::nova(&prov.vojo);
        let datumoj = b"ripetata enhavo";

        let unua = stokejo.konservi_fluon(datumoj.as_slice()).unwrap();
        let dua = stokejo.konservi_fluon(datumoj.as_slice()).unwrap();

        assert_eq!(unua, dua);
        assert_eq!(unua.1, datumoj.len() as u64);
        assert_eq!(stokejo.ricevi(&unua.0).unwrap(), datumoj);
    }

    #[test]
    fn testo_memora_konservilo() {
        let stokejo = MemoraKonservilo::nova();
        assert_eq!(stokejo.n_objektoj(), 0);

        let haketo = stokejo.konservi(b"enmemora").unwrap();
        assert_eq!(haketo, haketi(b"enmemora"));
        assert_eq!(stokejo.n_objektoj(), 1);
        assert_eq!(stokejo.ricevi(&haketo).unwrap(), b"enmemora");
        assert!(matches!(
            stokejo.ricevi(&haketi(b"mankanta")),
            Err(Eraro::NeekzistantaObjekto { .. })
        ));
    }
}
