//! Abstraktaĵoj por la laborspaco kaj la horloĝo.
//!
//! La deponeja motoro ne rekte uzas `std`: ĝi legas, skribas, listigas kaj
//! forigas nur per la trajto [`Laborspaco`], kaj petas la nunan tempon per
//! [`Horlogo`]. Tiel la sama motoro funkcias sur la dosiersistemo, en memoro
//! kaj en la estonta no_std-medio de `MiKerno`.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str;

use crate::eraroj::Eraro;

/// La speco de ero en la laborspaco.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speco {
    /// Ordinara dosiero.
    Dosiero,
    /// Dosierujo.
    Dosierujo,
    /// Simbola ligilo, fenestro aŭ alia neordinara ero.
    Alia,
}

/// La metadatumoj de unu ero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metadatumoj {
    /// La speco de la ero.
    pub speco: Speco,
    /// La grandeco en bajtoj (0 por dosierujoj).
    pub grandeco: u64,
    /// La modifotempo en nanosekundoj ekde la Uniksa epoko, se konata.
    pub mtempo: Option<u64>,
}

/// Unu ero listigita en dosierujo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ero {
    /// La nomo de la ero, sen la patra vojo.
    pub nomo: String,
    /// La speco de la ero.
    pub speco: Speco,
}

/// Operaciaj kapabloj, kiujn la motoro bezonas de la laborspaco.
///
/// Ĉiuj vojoj estas relativaj al la radiko kaj uzas `/` kiel apartigilon. La
/// malplena teksto signifas la radikon mem.
pub trait Laborspaco {
    /// La homa radiko de la laborspaco.
    fn radiko(&self) -> &str;

    /// Listigas la rektajn erojn de dosierujo. Mankanta dosierujo estas
    /// malplena listo.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, se la dosierujo ne legeblas.
    fn listi(&self, vojo: &str) -> Result<Vec<Ero>, Eraro>;

    /// Legas la tutan enhavon de dosiero.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NeekzistantaVojo`], se la dosiero mankas, aŭ alian
    /// laborspacan eraron.
    fn legi(&self, vojo: &str) -> Result<Vec<u8>, Eraro>;

    /// Skribas dosieron atome: provizora dosiero, poste renomo.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, se la dosiero ne skribeblas.
    fn skribi_atome(&self, vojo: &str, datumoj: &[u8]) -> Result<(), Eraro>;

    /// Forigas dosieron; mankanta dosiero ne estas eraro.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, se la dosiero ne forigeblas.
    fn forigi(&self, vojo: &str) -> Result<(), Eraro>;

    /// Forigas dosierujon; mankanta dosierujo ne estas eraro.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, se la dosierujo ne forigeblas.
    fn forigi_dosierujon(&self, vojo: &str) -> Result<(), Eraro>;

    /// Renomas aŭ movas eron.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NeekzistantaVojo`], se la fonto mankas, aŭ alian
    /// laborspacan eraron.
    fn renomi(&self, malnova: &str, nova: &str) -> Result<(), Eraro>;

    /// Kreas dosierujon kaj ĉiujn mankantajn gepatrojn (idempotente).
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, se la dosierujo ne kreeblas.
    fn krei_dosierujon(&self, vojo: &str) -> Result<(), Eraro>;

    /// Legas la metadatumojn de ero.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NeekzistantaVojo`], se la ero mankas, aŭ alian
    /// laborspacan eraron.
    fn metadatumoj(&self, vojo: &str) -> Result<Metadatumoj, Eraro>;

    /// Ĉu la ero ekzistas.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, se la stato ne determineblas.
    fn ekzistas(&self, vojo: &str) -> Result<bool, Eraro>;

    /// Legas UTF-8-dosieron; mankanta dosiero redonas `None`.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaDatumo`], se la enhavo ne estas UTF-8, aŭ
    /// laborspacan eraron.
    fn legi_tekston(&self, vojo: &str) -> Result<Option<String>, Eraro> {
        match self.legi(vojo) {
            Ok(bajtoj) => str::from_utf8(&bajtoj)
                .map(|teksto| Some(teksto.to_string()))
                .map_err(|_| Eraro::NevalidaDatumo {
                    kialo: "la teksta dosiero ne estas UTF-8",
                }),
            Err(Eraro::NeekzistantaVojo { .. }) => Ok(None),
            Err(eraro) => Err(eraro),
        }
    }
}

/// Provizanto de la nuna tempo.
pub trait Horlogo {
    /// La nuna tempo en nanosekundoj ekde la Uniksa epoko, se konata.
    fn nunaj_nanosekundoj(&self) -> Option<u64>;

    /// La nuna tempo en sekundoj ekde la Uniksa epoko.
    #[must_use]
    fn nunaj_sekundoj(&self) -> u64 {
        self.nunaj_nanosekundoj()
            .map_or(0, |nanosekundoj| nanosekundoj / 1_000_000_000)
    }
}

/// Fiksa horloĝo por testoj kaj pruvoj.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FiksaHorlogo {
    /// La tempo, kiun la horloĝo ĉiam redonas, en nanosekundoj.
    pub nunaj: u64,
}

impl FiksaHorlogo {
    /// Kreas fiksan horloĝon.
    #[must_use]
    pub const fn nova(nunaj: u64) -> Self {
        Self { nunaj }
    }
}

impl Horlogo for FiksaHorlogo {
    fn nunaj_nanosekundoj(&self) -> Option<u64> {
        Some(self.nunaj)
    }
}

/// Enmemora laborspaco por testoj, pruvoj kaj la estonta enmemora reĝimo.
///
/// Ĝi funkcias sen `std` kaj sen dosiersistema aliro. La radiko estas `/`, sed
/// ĉiuj vojoj restas relativaj kaj malplena vojo signifas la radikon.
#[derive(Debug, Default)]
pub struct Memspaco {
    eroj: core::cell::RefCell<alloc::collections::BTreeMap<String, Memero>>,
    mtempo: core::cell::Cell<u64>,
}

#[derive(Debug, Clone)]
struct Memero {
    speco: Speco,
    datumoj: Vec<u8>,
    mtempo: Option<u64>,
}

impl Memspaco {
    /// Kreas malplenan enmemoran laborspacon.
    #[must_use]
    pub fn nova() -> Self {
        Self::default()
    }

    fn sekva_mtempo(&self) -> u64 {
        let nuna = self.mtempo.get();
        self.mtempo.set(nuna.saturating_add(1_000_000_000));
        nuna
    }

    fn certigi_patrojn(eroj: &mut alloc::collections::BTreeMap<String, Memero>, vojo: &str) {
        let mut prefikso = String::new();
        for parto in vojo.split('/') {
            if !prefikso.is_empty() {
                prefikso.push('/');
            }
            prefikso.push_str(parto);
            if prefikso == vojo {
                break;
            }
            eroj.entry(prefikso.clone()).or_insert_with(|| Memero {
                speco: Speco::Dosierujo,
                datumoj: Vec::new(),
                mtempo: None,
            });
        }
    }
}

impl Laborspaco for Memspaco {
    fn radiko(&self) -> &'static str {
        "/"
    }

    fn listi(&self, vojo: &str) -> Result<Vec<Ero>, Eraro> {
        let eroj = self.eroj.borrow();
        if !vojo.is_empty() {
            let ero = eroj.get(vojo).ok_or_else(|| Eraro::NeekzistantaVojo {
                vojo: String::from(vojo),
            })?;
            if ero.speco != Speco::Dosierujo {
                return Err(Eraro::NeekzistantaVojo {
                    vojo: String::from(vojo),
                });
            }
        }
        let prefikso = if vojo.is_empty() {
            String::new()
        } else {
            alloc::format!("{vojo}/")
        };
        let mut videblaj = alloc::collections::BTreeMap::new();
        for (pado, ero) in &*eroj {
            let Some(resto) = pado.get(prefikso.len()..) else {
                continue;
            };
            if !pado.starts_with(&prefikso) || resto.is_empty() {
                continue;
            }
            let Some(nomo) = resto.split('/').next() else {
                continue;
            };
            let speco = if resto.contains('/') {
                Speco::Dosierujo
            } else {
                ero.speco
            };
            videblaj.insert(String::from(nomo), speco);
        }
        Ok(videblaj
            .into_iter()
            .map(|(nomo, speco)| Ero { nomo, speco })
            .collect())
    }

    fn legi(&self, vojo: &str) -> Result<Vec<u8>, Eraro> {
        let eroj = self.eroj.borrow();
        let ero = eroj.get(vojo).ok_or_else(|| Eraro::NeekzistantaVojo {
            vojo: String::from(vojo),
        })?;
        if ero.speco != Speco::Dosiero {
            return Err(Eraro::NeekzistantaVojo {
                vojo: String::from(vojo),
            });
        }
        Ok(ero.datumoj.clone())
    }

    fn skribi_atome(&self, vojo: &str, datumoj: &[u8]) -> Result<(), Eraro> {
        let mut eroj = self.eroj.borrow_mut();
        Self::certigi_patrojn(&mut eroj, vojo);
        let mtempo = self.sekva_mtempo();
        eroj.insert(
            String::from(vojo),
            Memero {
                speco: Speco::Dosiero,
                datumoj: datumoj.to_vec(),
                mtempo: Some(mtempo),
            },
        );
        Ok(())
    }

    fn forigi(&self, vojo: &str) -> Result<(), Eraro> {
        self.eroj.borrow_mut().remove(vojo);
        Ok(())
    }

    fn forigi_dosierujon(&self, vojo: &str) -> Result<(), Eraro> {
        let prefikso = alloc::format!("{vojo}/");
        self.eroj
            .borrow_mut()
            .retain(|pado, _| pado != vojo && !pado.starts_with(&prefikso));
        Ok(())
    }

    fn renomi(&self, malnova: &str, nova: &str) -> Result<(), Eraro> {
        let mut eroj = self.eroj.borrow_mut();
        let ero = eroj
            .remove(malnova)
            .ok_or_else(|| Eraro::NeekzistantaVojo {
                vojo: String::from(malnova),
            })?;
        let prefikso = alloc::format!("{malnova}/");
        let mut idoj = Vec::new();
        for pado in eroj.keys() {
            if let Some(resto) = pado.strip_prefix(&prefikso) {
                idoj.push((String::from(resto), pado.clone()));
            }
        }
        let mut movitaj = Vec::new();
        for (resto, pado) in idoj {
            if let Some(ero) = eroj.remove(&pado) {
                movitaj.push((alloc::format!("{nova}/{resto}"), ero));
            }
        }
        Self::certigi_patrojn(&mut eroj, nova);
        eroj.insert(String::from(nova), ero);
        for (pado, ero) in movitaj {
            eroj.insert(pado, ero);
        }
        Ok(())
    }

    fn krei_dosierujon(&self, vojo: &str) -> Result<(), Eraro> {
        if vojo.is_empty() {
            return Ok(());
        }
        let mut eroj = self.eroj.borrow_mut();
        Self::certigi_patrojn(&mut eroj, vojo);
        eroj.insert(
            String::from(vojo),
            Memero {
                speco: Speco::Dosierujo,
                datumoj: Vec::new(),
                mtempo: None,
            },
        );
        Ok(())
    }

    fn metadatumoj(&self, vojo: &str) -> Result<Metadatumoj, Eraro> {
        if vojo.is_empty() {
            return Ok(Metadatumoj {
                speco: Speco::Dosierujo,
                grandeco: 0,
                mtempo: None,
            });
        }
        let eroj = self.eroj.borrow();
        let ero = eroj.get(vojo).ok_or_else(|| Eraro::NeekzistantaVojo {
            vojo: String::from(vojo),
        })?;
        Ok(Metadatumoj {
            speco: ero.speco,
            grandeco: u64::try_from(ero.datumoj.len()).unwrap_or(u64::MAX),
            mtempo: ero.mtempo,
        })
    }

    fn ekzistas(&self, vojo: &str) -> Result<bool, Eraro> {
        Ok(vojo.is_empty() || self.eroj.borrow().contains_key(vojo))
    }
}

#[cfg(feature = "std")]
mod dosiera {
    use super::{Ero, Horlogo, Laborspaco, Metadatumoj, Speco};
    use crate::eraroj::Eraro;
    use alloc::format;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use std::fs;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static PROVIZORA_NOMBRO: AtomicU64 = AtomicU64::new(0);

    /// [`Laborspaco`] bazita sur la loka dosiersistemo.
    #[derive(Debug, Clone)]
    pub struct Dosierspaco {
        radiko: PathBuf,
        teksto: String,
    }

    impl Dosierspaco {
        /// Kreas laborspacon direktitan al la donita radiko.
        #[must_use]
        pub fn nova(radiko: impl Into<PathBuf>) -> Self {
            let radiko = radiko.into();
            let teksto = radiko.to_string_lossy().into_owned();
            Self { radiko, teksto }
        }

        /// La absoluta vojo de relativa ero.
        #[must_use]
        pub fn absoluta(&self, vojo: &str) -> PathBuf {
            if vojo.is_empty() {
                self.radiko.clone()
            } else {
                self.radiko.join(vojo)
            }
        }

        fn eraro(&self, vojo: &str, eraro: &std::io::Error) -> Eraro {
            if eraro.kind() == ErrorKind::NotFound {
                Eraro::NeekzistantaVojo {
                    vojo: self.absoluta(vojo).display().to_string(),
                }
            } else {
                Eraro::Laborspaco(format!("{}: {eraro}", self.absoluta(vojo).display()))
            }
        }
    }

    impl Laborspaco for Dosierspaco {
        fn radiko(&self) -> &str {
            &self.teksto
        }

        fn listi(&self, vojo: &str) -> Result<Vec<Ero>, Eraro> {
            let dosierujo = self.absoluta(vojo);
            let eniroj = match fs::read_dir(&dosierujo) {
                Ok(eniroj) => eniroj,
                Err(eraro) if eraro.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
                Err(eraro) => return Err(self.eraro(vojo, &eraro)),
            };
            let mut eroj = Vec::new();
            for eniro in eniroj {
                let eniro = eniro.map_err(|eraro| self.eraro(vojo, &eraro))?;
                let nomo = eniro
                    .file_name()
                    .into_string()
                    .map_err(|_| Eraro::NevalidaDosiernomo)?;
                let speco = match eniro.file_type() {
                    Ok(speco) if speco.is_dir() => Speco::Dosierujo,
                    Ok(speco) if speco.is_file() => Speco::Dosiero,
                    Ok(_) => Speco::Alia,
                    Err(eraro) => return Err(self.eraro(vojo, &eraro)),
                };
                eroj.push(Ero { nomo, speco });
            }
            eroj.sort_unstable_by(|unu, du| unu.nomo.cmp(&du.nomo));
            Ok(eroj)
        }

        fn legi(&self, vojo: &str) -> Result<Vec<u8>, Eraro> {
            fs::read(self.absoluta(vojo)).map_err(|eraro| self.eraro(vojo, &eraro))
        }

        fn skribi_atome(&self, vojo: &str, datumoj: &[u8]) -> Result<(), Eraro> {
            let celo = self.absoluta(vojo);
            let dosierujo = celo.parent().unwrap_or_else(|| Path::new("."));
            let nomo = celo
                .file_name()
                .map_or_else(String::new, |nomo| nomo.to_string_lossy().into_owned());
            let nombro = PROVIZORA_NOMBRO.fetch_add(1, Ordering::Relaxed);
            let provizora =
                dosierujo.join(format!(".{nomo}.{}.{}.prov", std::process::id(), nombro));
            fs::write(&provizora, datumoj).map_err(|eraro| self.eraro(vojo, &eraro))?;
            if let Err(eraro) = fs::rename(&provizora, &celo) {
                drop(fs::remove_file(&provizora));
                return Err(self.eraro(vojo, &eraro));
            }
            Ok(())
        }

        fn forigi(&self, vojo: &str) -> Result<(), Eraro> {
            match fs::remove_file(self.absoluta(vojo)) {
                Ok(()) => Ok(()),
                Err(eraro) if eraro.kind() == ErrorKind::NotFound => Ok(()),
                Err(eraro) => Err(self.eraro(vojo, &eraro)),
            }
        }

        fn forigi_dosierujon(&self, vojo: &str) -> Result<(), Eraro> {
            match fs::remove_dir(self.absoluta(vojo)) {
                Ok(()) => Ok(()),
                Err(eraro) if eraro.kind() == ErrorKind::NotFound => Ok(()),
                Err(eraro) => Err(self.eraro(vojo, &eraro)),
            }
        }

        fn renomi(&self, malnova: &str, nova: &str) -> Result<(), Eraro> {
            fs::rename(self.absoluta(malnova), self.absoluta(nova))
                .map_err(|eraro| self.eraro(malnova, &eraro))
        }

        fn krei_dosierujon(&self, vojo: &str) -> Result<(), Eraro> {
            fs::create_dir_all(self.absoluta(vojo)).map_err(|eraro| self.eraro(vojo, &eraro))
        }

        fn metadatumoj(&self, vojo: &str) -> Result<Metadatumoj, Eraro> {
            let metadatumoj = fs::symlink_metadata(self.absoluta(vojo))
                .map_err(|eraro| self.eraro(vojo, &eraro))?;
            let speco = if metadatumoj.is_dir() {
                Speco::Dosierujo
            } else if metadatumoj.is_file() {
                Speco::Dosiero
            } else {
                Speco::Alia
            };
            let mtempo = metadatumoj
                .modified()
                .ok()
                .and_then(|tempo| tempo.duration_since(UNIX_EPOCH).ok())
                .and_then(|daŭro| u64::try_from(daŭro.as_nanos()).ok());
            Ok(Metadatumoj {
                speco,
                grandeco: metadatumoj.len(),
                mtempo,
            })
        }

        fn ekzistas(&self, vojo: &str) -> Result<bool, Eraro> {
            self.absoluta(vojo)
                .try_exists()
                .map_err(|eraro| self.eraro(vojo, &eraro))
        }
    }

    /// [`Horlogo`] bazita sur la sistema horloĝo.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct StdaHorlogo;

    impl Horlogo for StdaHorlogo {
        fn nunaj_nanosekundoj(&self) -> Option<u64> {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .and_then(|daŭro| u64::try_from(daŭro.as_nanos()).ok())
        }
    }
}

#[cfg(feature = "std")]
pub use dosiera::{Dosierspaco, StdaHorlogo};
