//! Deponeja metadatumaro kaj DAG-administrado (nur kun `std`).

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;

use crate::eraroj::Eraro;
use crate::objektoj::{ArbaEnskribo, Arbo, Komito};
use crate::stokado::{DosieraKonservilo, Haketo, Konservilo};

/// Nomo de la metadatumuja dosierujo.
pub const VS_DOSIERUJO: &str = ".vs";

/// Nomo de la objekta dosierujo.
pub const OBJEKTUJO: &str = "objektoj";

/// Nomo de la HEAD-dosiero.
pub const HEAD_DOSIERO: &str = "HEAD";

static TEMP_NOMBRO: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
enum Speco {
    Dosierujo,
    Dosiero,
    Alia,
}

struct Ero {
    nomo: String,
    vojo: PathBuf,
    speco: Speco,
}

/// Versio-deponejo en la loka dosiersistemo.
pub struct Deponejo {
    radiko: PathBuf,
    stokejo: DosieraKonservilo,
}

impl Deponejo {
    /// Iniciatas deponejon en la donita radika dosierujo.
    ///
    /// La operacio estas idempotenta: ekzistanta `.vs`-strukturo ne estas
    /// forviŝita, kaj nur mankantaj partoj estas kreataj.
    pub fn iniciati(radiko: impl Into<PathBuf>) -> Result<Self, Eraro> {
        let radiko = radiko.into();
        let metadatumoj = radiko.join(VS_DOSIERUJO);
        fs::create_dir_all(metadatumoj.join(OBJEKTUJO))?;

        let head = metadatumoj.join(HEAD_DOSIERO);
        if !head.exists() {
            skribi_atome(&head, b"")?;
        }

        Ok(Self {
            stokejo: DosieraKonservilo::nova(metadatumoj.join(OBJEKTUJO)),
            radiko,
        })
    }

    /// Malfermas ekzistantan deponejon.
    pub fn malfermi(radiko: impl Into<PathBuf>) -> Result<Self, Eraro> {
        let radiko = radiko.into();
        let metadatumoj = radiko.join(VS_DOSIERUJO);
        if !metadatumoj.is_dir() || !metadatumoj.join(HEAD_DOSIERO).is_file() {
            return Err(Eraro::Nedeponejo);
        }
        fs::create_dir_all(metadatumoj.join(OBJEKTUJO))?;

        Ok(Self {
            stokejo: DosieraKonservilo::nova(metadatumoj.join(OBJEKTUJO)),
            radiko,
        })
    }

    /// La radika dosierujo de la deponejo.
    pub fn radiko(&self) -> &Path {
        &self.radiko
    }

    /// La objektstokejo de la deponejo.
    pub fn stokejo(&self) -> &DosieraKonservilo {
        &self.stokejo
    }

    fn metadatumoj(&self) -> PathBuf {
        self.radiko.join(VS_DOSIERUJO)
    }

    /// Legas la haketon en `HEAD`; `None` signifas, ke ankoraŭ ne ekzistas komito.
    pub fn legi_head(&self) -> Result<Option<Haketo>, Eraro> {
        let teksto = fs::read_to_string(self.metadatumoj().join(HEAD_DOSIERO))?;
        let teksto = teksto.trim();
        if teksto.is_empty() {
            Ok(None)
        } else {
            Ok(Some(teksto.parse()?))
        }
    }

    /// Skribas la donitan komiton kiel la novan `HEAD`.
    pub fn skribi_head(&self, haketo: &Haketo) -> Result<(), Eraro> {
        skribi_atome(
            &self.metadatumoj().join(HEAD_DOSIERO),
            haketo.to_string().as_bytes(),
        )
    }

    /// Konservas arbon kaj redonas ĝian haketon.
    pub fn konservi_arbon(&self, arbo: &Arbo) -> Result<Haketo, Eraro> {
        self.stokejo.konservi(&arbo.seriigi()?)
    }

    /// Legas arbon el la stokejo.
    pub fn legi_arbon(&self, haketo: &Haketo) -> Result<Arbo, Eraro> {
        Arbo::malseriigi(&self.stokejo.ricevi(haketo)?)
    }

    /// Konservas komiton kaj redonas ĝian haketon.
    pub fn konservi_komiton(&self, komito: &Komito) -> Result<Haketo, Eraro> {
        self.stokejo.konservi(&komito.seriigi()?)
    }

    /// Legas komiton el la stokejo.
    pub fn legi_komiton(&self, haketo: &Haketo) -> Result<Komito, Eraro> {
        Komito::malseriigi(&self.stokejo.ricevi(haketo)?)
    }

    /// Kaptas la nunan laborspacon kaj redonas la haketon de la radika arbo.
    pub fn kapti_arbon(&self) -> Result<Haketo, Eraro> {
        let arbo = self.konstrui_arbon(&self.radiko)?;
        self.konservi_arbon(&arbo)
    }

    fn konstrui_arbon(&self, dosierujo: &Path) -> Result<Arbo, Eraro> {
        let mut eroj = Vec::new();
        for eniro in fs::read_dir(dosierujo)? {
            let eniro = eniro?;
            let nomo = eniro
                .file_name()
                .into_string()
                .map_err(|_| Eraro::NevalidaDosiernomo)?;
            if nomo == VS_DOSIERUJO {
                continue;
            }

            let speco = speco_de(eniro.file_type()?);
            eroj.push(Ero {
                nomo,
                vojo: eniro.path(),
                speco,
            });
        }

        let mut enskriboj = eroj
            .par_iter()
            .map(|ero| self.enskribi(ero))
            .collect::<Result<Vec<_>, Eraro>>()?;
        enskriboj.sort_by(|a, b| a.nomo().cmp(b.nomo()));

        Ok(Arbo { enskriboj })
    }

    fn enskribi(&self, ero: &Ero) -> Result<ArbaEnskribo, Eraro> {
        match ero.speco {
            Speco::Dosierujo => {
                let subarbo = self.konstrui_arbon(&ero.vojo)?;
                let haketo = self.konservi_arbon(&subarbo)?;
                Ok(ArbaEnskribo::SubArbo {
                    nomo: ero.nomo.clone(),
                    haketo,
                })
            }
            Speco::Dosiero => {
                let (haketo, grandeco) = self.stokejo.konservi_dosieron(&ero.vojo)?;
                Ok(ArbaEnskribo::Dosiero {
                    nomo: ero.nomo.clone(),
                    haketo,
                    grandeco,
                })
            }
            Speco::Alia => Err(Eraro::NesubtenitaDosiertipo {
                vojo: ero.vojo.display().to_string(),
            }),
        }
    }
}

fn speco_de(speco: fs::FileType) -> Speco {
    if speco.is_dir() {
        Speco::Dosierujo
    } else if speco.is_file() {
        Speco::Dosiero
    } else {
        Speco::Alia
    }
}

fn skribi_atome(vojo: &Path, datumoj: &[u8]) -> Result<(), Eraro> {
    let dosierujo = vojo.parent().unwrap_or_else(|| Path::new("."));
    let nomo = vojo
        .file_name()
        .map(|nomo| nomo.to_string_lossy())
        .unwrap_or_default();
    let nombro = TEMP_NOMBRO.fetch_add(1, Ordering::Relaxed);
    let provizora = dosierujo.join(format!(".{nomo}.{}.{}.tmp", std::process::id(), nombro));

    fs::write(&provizora, datumoj)?;
    if let Err(eraro) = fs::rename(&provizora, vojo) {
        let _ = fs::remove_file(&provizora);
        return Err(eraro.into());
    }
    Ok(())
}

#[cfg(test)]
mod testoj {
    use super::*;

    static PROVOJ: AtomicU64 = AtomicU64::new(0);

    struct ProvDeponejo {
        radiko: PathBuf,
    }

    impl ProvDeponejo {
        fn nova(etikedo: &str) -> Self {
            let nombro = PROVOJ.fetch_add(1, Ordering::Relaxed);
            let radiko = std::env::temp_dir().join(format!(
                "versio-deponejo-{etikedo}-{}-{nombro}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&radiko);
            fs::create_dir_all(&radiko).expect("krei provan dosierujon");
            Self { radiko }
        }
    }

    impl Drop for ProvDeponejo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.radiko);
        }
    }

    #[test]
    fn testo_iniciati_kreas_metadatumojn() {
        let prov = ProvDeponejo::nova("iniciati");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();

        assert!(prov.radiko.join(VS_DOSIERUJO).is_dir());
        assert!(prov.radiko.join(VS_DOSIERUJO).join(OBJEKTUJO).is_dir());
        assert!(prov.radiko.join(VS_DOSIERUJO).join(HEAD_DOSIERO).is_file());
        assert_eq!(deponejo.legi_head().unwrap(), None);

        // Ripeta iniciato konservas la ekzistantan strukturon.
        let denove = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        assert_eq!(denove.legi_head().unwrap(), None);
    }

    #[test]
    fn testo_kapti_arbon_kaj_legi_blobojn() {
        let prov = ProvDeponejo::nova("arbo");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();

        fs::write(prov.radiko.join("testo.txt"), b"saluton\n").unwrap();
        fs::create_dir(prov.radiko.join("sub")).unwrap();
        fs::write(prov.radiko.join("sub").join("dosiero.txt"), b"mondo\n").unwrap();

        let haketo = deponejo.kapti_arbon().unwrap();
        let arbo = deponejo.legi_arbon(&haketo).unwrap();

        assert_eq!(arbo.enskriboj.len(), 2);
        assert_eq!(arbo.enskriboj[0].nomo(), "sub");
        assert_eq!(arbo.enskriboj[1].nomo(), "testo.txt");
        assert!(arbo.enskriboj.iter().all(|ero| ero.nomo() != VS_DOSIERUJO));

        let ArbaEnskribo::SubArbo {
            haketo: sub_haketo, ..
        } = &arbo.enskriboj[0]
        else {
            panic!("atendita subarbo");
        };
        let subarbo = deponejo.legi_arbon(sub_haketo).unwrap();
        assert_eq!(subarbo.enskriboj.len(), 1);

        let ArbaEnskribo::Dosiero {
            haketo: blob_haketo,
            grandeco,
            ..
        } = &subarbo.enskriboj[0]
        else {
            panic!("atendita dosiero");
        };
        assert_eq!(*grandeco, 6);
        assert_eq!(
            deponejo.stokejo().ricevi(blob_haketo).unwrap(),
            b"mondo\n".to_vec()
        );
    }

    #[test]
    fn testo_komitoj_ligighas_al_gepatro() {
        let prov = ProvDeponejo::nova("komito");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("testo.txt"), b"unua").unwrap();

        let arbo1 = deponejo.kapti_arbon().unwrap();
        let komito1 = Komito {
            arbo: arbo1,
            gepatro: deponejo.legi_head().unwrap(),
            mesagxo: "Unua".into(),
            kreita_je: 1,
        };
        let h1 = deponejo.konservi_komiton(&komito1).unwrap();
        deponejo.skribi_head(&h1).unwrap();
        assert_eq!(deponejo.legi_head().unwrap(), Some(h1));

        fs::write(prov.radiko.join("testo.txt"), b"dua").unwrap();
        let arbo2 = deponejo.kapti_arbon().unwrap();
        let komito2 = Komito {
            arbo: arbo2,
            gepatro: deponejo.legi_head().unwrap(),
            mesagxo: "Dua".into(),
            kreita_je: 2,
        };
        let h2 = deponejo.konservi_komiton(&komito2).unwrap();
        deponejo.skribi_head(&h2).unwrap();

        let legita = deponejo.legi_komiton(&h2).unwrap();
        assert_eq!(legita.gepatro, Some(h1));
        assert_eq!(legita.mesagxo, "Dua");

        let denove = Deponejo::malfermi(prov.radiko.clone()).unwrap();
        assert_eq!(denove.legi_head().unwrap(), Some(h2));
    }

    #[test]
    fn testo_malfermi_neekzistantan_deponejon() {
        let prov = ProvDeponejo::nova("mankanta");
        assert!(matches!(
            Deponejo::malfermi(prov.radiko.clone()),
            Err(Eraro::Nedeponejo)
        ));
    }
}
