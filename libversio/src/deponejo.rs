//! Deponeja metadatumaro kaj DAG-administrado (nur kun `std`).

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rayon::prelude::*;

use crate::eraroj::Eraro;
use crate::indekso::{IndeksEnskribo, Indekso};
use crate::objektoj::{ArbaEnskribo, Arbo, Komito};
use crate::stokado::{DosieraKonservilo, Haketo, Konservilo, haketi_dosieron};
use crate::vojo::{MAKSIMA_VOJO, validi_komponanton};

/// Nomo de la metadatumuja dosierujo.
pub const VS_DOSIERUJO: &str = ".vs";

/// Nomo de la objekta dosierujo.
pub const OBJEKTUJO: &str = "objektoj";

/// Nomo de la HEAD-dosiero.
pub const HEAD_DOSIERO: &str = "HEAD";

/// Nomo de la indeksa dosiero (la metadatuma kaŝmemoro).
pub const INDEKSO_DOSIERO: &str = "indekso";

/// Dosieroj modifitaj malpli ol tiom antaŭ la skanado ne estas kaŝmemorataj.
///
/// Racy-git-problemo: dosiersistemaj tempstampoj havas krudan precizecon, do
/// dua skribo en la sama tempero povus konservi la saman mtime kaj grandecon.
/// Nur sufiĉe malnovaj tempstampoj garantias, ke posta ŝanĝo videblas.
const FRESXA_MARGENO: Duration = Duration::from_secs(2);

static TEMP_NOMBRO: AtomicU64 = AtomicU64::new(0);

/// La ŝanĝoj de la laborspaco rilate al la arbo de `HEAD`.
///
/// Ĉiu listo enhavas relativajn vojojn kun `/`, ordigitajn laŭ bajtoj.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stato {
    /// Dosieroj, kies enhavo diferencas de `HEAD`.
    pub modifitaj: Vec<String>,
    /// Dosieroj, kiuj ne ekzistas en `HEAD`.
    pub novaj: Vec<String>,
    /// Dosieroj de `HEAD`, kiuj ne plu ekzistas en la laborspaco.
    pub forigitaj: Vec<String>,
}

impl Stato {
    /// Ĉu la laborspaco kongruas kun `HEAD`.
    #[must_use]
    pub fn estas_pura(&self) -> bool {
        self.modifitaj.is_empty() && self.novaj.is_empty() && self.forigitaj.is_empty()
    }
}

struct LaborDosiero {
    vojo: String,
    absoluta: PathBuf,
    grandeco: u64,
    mtempo: Option<u64>,
}

/// Dosieruja nodo por konstrui la arbaron el plata listo de dosieroj.
#[derive(Default)]
struct Nodo {
    dosieroj: Vec<(String, Haketo, u64)>,
    subarboj: HashMap<String, Nodo>,
}

struct HaketitaDosiero {
    vojo: String,
    haketo: Haketo,
    grandeco: u64,
    /// `(mtempo, grandeco)`, se la dosiero estas sufiĉe malnova por la kaŝmemoro.
    kasxebla: Option<(u64, u64)>,
}

#[derive(Clone, Copy)]
enum Speco {
    Dosierujo,
    Dosiero,
    Alia,
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
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman eraron, se la strukturo ne kreeblas.
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
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::Nedeponejo`], se la dosierujo ne enhavas `.vs`, aŭ
    /// dosiersisteman eraron.
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
    #[must_use]
    pub fn radiko(&self) -> &Path {
        &self.radiko
    }

    /// La objektstokejo de la deponejo.
    #[must_use]
    pub fn stokejo(&self) -> &DosieraKonservilo {
        &self.stokejo
    }

    fn metadatumoj(&self) -> PathBuf {
        self.radiko.join(VS_DOSIERUJO)
    }

    /// Legas la haketon en `HEAD`; `None` signifas, ke ankoraŭ ne ekzistas komito.
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman eraron aŭ [`Eraro::NevalidaHaketo`], se `HEAD`
    /// enhavas nevalidan tekston.
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
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman eraron.
    pub fn skribi_head(&self, haketo: &Haketo) -> Result<(), Eraro> {
        skribi_atome(
            &self.metadatumoj().join(HEAD_DOSIERO),
            haketo.to_string().as_bytes(),
        )
    }

    /// Konservas arbon kaj redonas ĝian haketon.
    ///
    /// # Errors
    ///
    /// Redonas seriigan aŭ stokejan eraron.
    pub fn konservi_arbon(&self, arbo: &Arbo) -> Result<Haketo, Eraro> {
        self.stokejo.konservi(&arbo.seriigi()?)
    }

    /// Legas arbon el la stokejo.
    ///
    /// # Errors
    ///
    /// Redonas stokejan aŭ malseriigan eraron.
    pub fn legi_arbon(&self, haketo: &Haketo) -> Result<Arbo, Eraro> {
        Arbo::malseriigi(&self.stokejo.ricevi(haketo)?)
    }

    /// Konservas komiton kaj redonas ĝian haketon.
    ///
    /// # Errors
    ///
    /// Redonas seriigan aŭ stokejan eraron.
    pub fn konservi_komiton(&self, komito: &Komito) -> Result<Haketo, Eraro> {
        self.stokejo.konservi(&komito.seriigi()?)
    }

    /// Legas komiton el la stokejo.
    ///
    /// # Errors
    ///
    /// Redonas stokejan aŭ malseriigan eraron.
    pub fn legi_komiton(&self, haketo: &Haketo) -> Result<Komito, Eraro> {
        Komito::malseriigi(&self.stokejo.ricevi(haketo)?)
    }

    /// Kaptas la nunan laborspacon kaj redonas la haketon de la radika arbo.
    ///
    /// La skanado, la haketado kaj la arba konstruo okazas paralele. Dosieroj,
    /// kies mtime kaj grandeco kongruas kun la indekso kaj kies objekto jam
    /// ekzistas en la stokejo, estas reuzataj sen relegado; nur ŝanĝitaj aŭ
    /// novaj dosieroj estas relegataj kaj kunpremataj. Post la kaptado la
    /// indekso estas ĝisdatigita, same kiel ĉe [`Deponejo::stato`].
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman eraron, [`Eraro::NevalidaDosiernomo`],
    /// [`Eraro::NevalidaVojaKomponanto`] aŭ [`Eraro::NesubtenitaDosiertipo`]
    /// por la laborspaco, aŭ stokejan eraron.
    pub fn kapti_arbon(&self) -> Result<Haketo, Eraro> {
        let limo = SystemTime::now()
            .checked_sub(FRESXA_MARGENO)
            .and_then(nanosekundoj);

        let malnova = self.legi_indekson()?;
        let haketitaj = skani(&self.radiko, "")?
            .into_par_iter()
            .map(|dosiero| self.konservi_laboran(dosiero, &malnova, limo))
            .collect::<Result<Vec<_>, Eraro>>()?;
        self.gxisdatigi_indekson(&malnova, &haketitaj)?;

        self.konservi_nodon(&nodo_el_dosieroj(&haketitaj))
    }

    /// Reuzas la indeksan haketon, se ĝi kongruas kaj la objekto ekzistas;
    /// alie relegas la dosieron kaj konservas ĝin en la stokejo.
    fn konservi_laboran(
        &self,
        dosiero: LaborDosiero,
        indekso: &Indekso,
        limo: Option<u64>,
    ) -> Result<HaketitaDosiero, Eraro> {
        let mtempo = dosiero
            .mtempo
            .filter(|mtempo| limo.is_some_and(|limo| *mtempo < limo));

        if let Some(mtempo) = mtempo
            && let Some(haketo) = indekso.kasxita_haketo(&dosiero.vojo, mtempo, dosiero.grandeco)
            && self.stokejo.havas(&haketo)
        {
            return Ok(HaketitaDosiero {
                vojo: dosiero.vojo,
                haketo,
                grandeco: dosiero.grandeco,
                kasxebla: Some((mtempo, dosiero.grandeco)),
            });
        }

        let (haketo, grandeco) = self.stokejo.konservi_dosieron(&dosiero.absoluta)?;
        let kasxebla = mtempo
            .filter(|_| grandeco == dosiero.grandeco)
            .map(|mtempo| (mtempo, grandeco));
        Ok(HaketitaDosiero {
            vojo: dosiero.vojo,
            haketo,
            grandeco,
            kasxebla,
        })
    }

    /// Konservas la arbon de unu dosieruja nodo kaj ĝiajn subarbojn paralele.
    fn konservi_nodon(&self, nodo: &Nodo) -> Result<Haketo, Eraro> {
        let mut enskriboj =
            Vec::with_capacity(nodo.dosieroj.len().saturating_add(nodo.subarboj.len()));
        for (nomo, haketo, grandeco) in &nodo.dosieroj {
            enskriboj.push(ArbaEnskribo::Dosiero {
                nomo: nomo.clone(),
                haketo: *haketo,
                grandeco: *grandeco,
            });
        }
        let subaj = nodo
            .subarboj
            .par_iter()
            .map(|(nomo, sub)| {
                Ok(ArbaEnskribo::SubArbo {
                    nomo: nomo.clone(),
                    haketo: self.konservi_nodon(sub)?,
                })
            })
            .collect::<Result<Vec<_>, Eraro>>()?;
        enskriboj.extend(subaj);
        self.konservi_arbon(&Arbo::el_enskriboj(enskriboj)?)
    }

    /// Komparas la laborspacon kun la arbo de `HEAD`.
    ///
    /// Haketo estas rekalkulata nur por dosieroj, kies mtime aŭ grandeco
    /// diferencas de la indekso; la ceteraj reuzas la memoritan haketon. La
    /// skanado, la haketado kaj la komparo okazas paralele. Se la kaŝmemoro
    /// ŝanĝiĝis, la indekso estas atome reskribita.
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman eraron, [`Eraro::NevalidaDosiernomo`],
    /// [`Eraro::NevalidaVojaKomponanto`] aŭ [`Eraro::NesubtenitaDosiertipo`]
    /// por la laborspaco, aŭ stokejan eraron por la arbo de `HEAD`.
    pub fn stato(&self) -> Result<Stato, Eraro> {
        let limo = SystemTime::now()
            .checked_sub(FRESXA_MARGENO)
            .and_then(nanosekundoj);

        let (haketitaj, head) = rayon::join(
            || -> Result<Vec<HaketitaDosiero>, Eraro> {
                let malnova = self.legi_indekson()?;
                let haketitaj = skani(&self.radiko, "")?
                    .into_par_iter()
                    .map(|dosiero| haketi_laboran(dosiero, &malnova, limo))
                    .collect::<Result<Vec<_>, Eraro>>()?;
                self.gxisdatigi_indekson(&malnova, &haketitaj)?;
                Ok(haketitaj)
            },
            || -> Result<HashMap<String, Haketo>, Eraro> {
                match self.legi_head()? {
                    Some(komito) => {
                        let arbo = self.legi_komiton(&komito)?.arbo;
                        Ok(self.platigi_arbon(&arbo, "")?.into_iter().collect())
                    }
                    None => Ok(HashMap::new()),
                }
            },
        );
        Ok(kompari(&head?, &haketitaj?))
    }

    /// Reskribas la indekson, se la kaŝeblaj enskriboj ŝanĝiĝis.
    fn gxisdatigi_indekson(
        &self,
        malnova: &Indekso,
        haketitaj: &[HaketitaDosiero],
    ) -> Result<(), Eraro> {
        // La vojoj estas unikaj, do egala nombro kaj kongruo de ĉiu kaŝebla
        // dosiero signifas egalan indekson; tiel la kutima kazo evitas rekonstruon.
        let kasxeblaj = haketitaj
            .par_iter()
            .filter(|dosiero| dosiero.kasxebla.is_some())
            .count();
        let sama = kasxeblaj == malnova.enskriboj().len()
            && haketitaj.par_iter().all(|dosiero| {
                dosiero.kasxebla.is_none_or(|(mtempo, grandeco)| {
                    malnova.trovi(&dosiero.vojo).is_some_and(|enskribo| {
                        enskribo.mtempo == mtempo
                            && enskribo.grandeco == grandeco
                            && enskribo.haketo == dosiero.haketo
                    })
                })
            });
        if sama {
            return Ok(());
        }

        let nova = Indekso::el_enskriboj(
            haketitaj
                .iter()
                .filter_map(|dosiero| {
                    dosiero.kasxebla.map(|(mtempo, grandeco)| IndeksEnskribo {
                        vojo: dosiero.vojo.clone(),
                        mtempo,
                        grandeco,
                        haketo: dosiero.haketo,
                    })
                })
                .collect(),
        )?;
        skribi_atome(&self.metadatumoj().join(INDEKSO_DOSIERO), &nova.seriigi()?)
    }

    /// Legas la indekson; manka aŭ difektita indekso estas malplena kaŝmemoro.
    fn legi_indekson(&self) -> Result<Indekso, Eraro> {
        match fs::read(self.metadatumoj().join(INDEKSO_DOSIERO)) {
            // La indekso estas nur kaŝmemoro: difektita indekso estas rekonstruata
            // per plena haketado, do ĝia eraro ne devas haltigi la komandon.
            Ok(bajtoj) => Ok(Indekso::malseriigi(&bajtoj).unwrap_or_default()),
            Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => Ok(Indekso::default()),
            Err(eraro) => Err(eraro.into()),
        }
    }

    /// Paralele legas la arbon kaj ĝiajn subarbojn en platan liston de dosieroj.
    fn platigi_arbon(
        &self,
        haketo: &Haketo,
        prefikso: &str,
    ) -> Result<Vec<(String, Haketo)>, Eraro> {
        let arbo = self.legi_arbon(haketo)?;
        let mut dosieroj = Vec::new();
        let mut subarboj = Vec::new();
        for enskribo in arbo.enskriboj() {
            let vojo = kunigi(prefikso, enskribo.nomo())?;
            match enskribo {
                ArbaEnskribo::Dosiero { haketo, .. } => dosieroj.push((vojo, *haketo)),
                ArbaEnskribo::SubArbo { haketo, .. } => subarboj.push((vojo, *haketo)),
            }
        }
        let subaj = subarboj
            .par_iter()
            .map(|(vojo, haketo)| self.platigi_arbon(haketo, vojo))
            .collect::<Result<Vec<_>, Eraro>>()?;
        dosieroj.extend(subaj.into_iter().flatten());
        Ok(dosieroj)
    }
}

/// Kunigas relativan vojon kun plia komponanto, respektante [`MAKSIMA_VOJO`].
fn kunigi(prefikso: &str, nomo: &str) -> Result<String, Eraro> {
    let vojo = if prefikso.is_empty() {
        String::from(nomo)
    } else {
        format!("{prefikso}/{nomo}")
    };
    if vojo.len() > MAKSIMA_VOJO {
        return Err(Eraro::TrolongaEnigo {
            maksimumo: MAKSIMA_VOJO,
            akirita: vojo.len(),
        });
    }
    Ok(vojo)
}

/// Konstruas la dosierujan hierarkion el plata listo de relativaj vojoj.
fn nodo_el_dosieroj(dosieroj: &[HaketitaDosiero]) -> Nodo {
    let mut radiko = Nodo::default();
    for dosiero in dosieroj {
        let mut nodo = &mut radiko;
        let mut partoj = dosiero.vojo.split('/').peekable();
        while let Some(parto) = partoj.next() {
            if partoj.peek().is_some() {
                nodo = nodo.subarboj.entry(parto.to_owned()).or_default();
            } else {
                nodo.dosieroj
                    .push((parto.to_owned(), dosiero.haketo, dosiero.grandeco));
            }
        }
    }
    radiko
}

fn nanosekundoj(tempo: SystemTime) -> Option<u64> {
    let dauxro = tempo.duration_since(UNIX_EPOCH).ok()?;
    u64::try_from(dauxro.as_nanos()).ok()
}

/// Rekursie kaj paralele listigas la regulajn dosierojn sub `dosierujo`.
fn skani(dosierujo: &Path, prefikso: &str) -> Result<Vec<LaborDosiero>, Eraro> {
    let mut dosieroj = Vec::new();
    let mut subdosierujoj = Vec::new();
    for eniro in fs::read_dir(dosierujo)? {
        let eniro = eniro?;
        let nomo = eniro
            .file_name()
            .into_string()
            .map_err(|_| Eraro::NevalidaDosiernomo)?;
        if nomo == VS_DOSIERUJO {
            continue;
        }
        validi_komponanton(&nomo)?;
        let vojo = kunigi(prefikso, &nomo)?;
        match speco_de(eniro.file_type()?) {
            Speco::Dosierujo => subdosierujoj.push((eniro.path(), vojo)),
            Speco::Dosiero => dosieroj.push((eniro, vojo)),
            Speco::Alia => {
                return Err(Eraro::NesubtenitaDosiertipo {
                    vojo: eniro.path().display().to_string(),
                });
            }
        }
    }

    let mut rezulto = dosieroj
        .into_par_iter()
        .map(|(eniro, vojo)| {
            let metadatumoj = eniro.metadata()?;
            Ok(LaborDosiero {
                vojo,
                absoluta: eniro.path(),
                grandeco: metadatumoj.len(),
                mtempo: metadatumoj.modified().ok().and_then(nanosekundoj),
            })
        })
        .collect::<Result<Vec<_>, Eraro>>()?;
    let subaj = subdosierujoj
        .par_iter()
        .map(|(absoluta, vojo)| skani(absoluta, vojo))
        .collect::<Result<Vec<_>, Eraro>>()?;
    rezulto.extend(subaj.into_iter().flatten());
    Ok(rezulto)
}

/// Prenas la haketon el la indekso, se la metadatumoj kongruas, alie relegas.
fn haketi_laboran(
    dosiero: LaborDosiero,
    indekso: &Indekso,
    limo: Option<u64>,
) -> Result<HaketitaDosiero, Eraro> {
    let mtempo = dosiero
        .mtempo
        .filter(|mtempo| limo.is_some_and(|limo| *mtempo < limo));

    if let Some(mtempo) = mtempo
        && let Some(haketo) = indekso.kasxita_haketo(&dosiero.vojo, mtempo, dosiero.grandeco)
    {
        return Ok(HaketitaDosiero {
            vojo: dosiero.vojo,
            haketo,
            grandeco: dosiero.grandeco,
            kasxebla: Some((mtempo, dosiero.grandeco)),
        });
    }

    let (haketo, grandeco) = haketi_dosieron(&dosiero.absoluta)?;
    // Se la grandeco ŝanĝiĝis inter `stat` kaj legado, la dosiero estas
    // ankoraŭ skribata; ĝi ne estas kaŝmemorata.
    let kasxebla = mtempo
        .filter(|_| grandeco == dosiero.grandeco)
        .map(|mtempo| (mtempo, grandeco));
    Ok(HaketitaDosiero {
        vojo: dosiero.vojo,
        haketo,
        grandeco,
        kasxebla,
    })
}

/// Paralele klasas la laborspacajn dosierojn rilate al la plata `HEAD`-arbo.
fn kompari(head: &HashMap<String, Haketo>, laboraj: &[HaketitaDosiero]) -> Stato {
    let mut modifitaj: Vec<String> = laboraj
        .par_iter()
        .filter(|dosiero| {
            head.get(&dosiero.vojo)
                .is_some_and(|haketo| *haketo != dosiero.haketo)
        })
        .map(|dosiero| dosiero.vojo.clone())
        .collect();
    let mut novaj: Vec<String> = laboraj
        .par_iter()
        .filter(|dosiero| !head.contains_key(&dosiero.vojo))
        .map(|dosiero| dosiero.vojo.clone())
        .collect();

    // La vojoj estas unikaj ambaŭflanke: se ĉiu dosiero de `HEAD` havas
    // laborspacan paron, nenio estis forigita kaj la aro ne necesas.
    let kongruaj = laboraj.len().saturating_sub(novaj.len());
    let mut forigitaj: Vec<String> = if kongruaj == head.len() {
        Vec::new()
    } else {
        let ekzistantaj: HashSet<&str> = laboraj
            .iter()
            .map(|dosiero| dosiero.vojo.as_str())
            .collect();
        head.par_iter()
            .filter(|(vojo, _)| !ekzistantaj.contains(vojo.as_str()))
            .map(|(vojo, _)| vojo.clone())
            .collect()
    };

    modifitaj.par_sort_unstable();
    novaj.par_sort_unstable();
    forigitaj.par_sort_unstable();
    Stato {
        modifitaj,
        novaj,
        forigitaj,
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
        .map_or_else(String::new, |nomo| nomo.to_string_lossy().into_owned());
    let nombro = TEMP_NOMBRO.fetch_add(1, Ordering::Relaxed);
    let provizora = dosierujo.join(format!(".{nomo}.{}.{}.tmp", std::process::id(), nombro));

    fs::write(&provizora, datumoj)?;
    if let Err(eraro) = fs::rename(&provizora, vojo) {
        drop(fs::remove_file(&provizora));
        return Err(eraro.into());
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

    use super::{Deponejo, Eraro, HEAD_DOSIERO, INDEKSO_DOSIERO, OBJEKTUJO, Stato, VS_DOSIERUJO};
    use crate::objektoj::{ArbaEnskribo, Komito};
    use crate::stokado::Konservilo;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, SystemTime};

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
            drop(fs::remove_dir_all(&radiko));
            fs::create_dir_all(&radiko).expect("krei provan dosierujon");
            Self { radiko }
        }
    }

    impl Drop for ProvDeponejo {
        fn drop(&mut self) {
            drop(fs::remove_dir_all(&self.radiko));
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

        assert_eq!(arbo.enskriboj().len(), 2);
        assert_eq!(arbo.enskriboj()[0].nomo(), "sub");
        assert_eq!(arbo.enskriboj()[1].nomo(), "testo.txt");
        assert!(
            arbo.enskriboj()
                .iter()
                .all(|ero| ero.nomo() != VS_DOSIERUJO)
        );

        let ArbaEnskribo::SubArbo {
            haketo: sub_haketo, ..
        } = &arbo.enskriboj()[0]
        else {
            panic!("atendita subarbo");
        };
        let subarbo = deponejo.legi_arbon(sub_haketo).unwrap();
        assert_eq!(subarbo.enskriboj().len(), 1);

        let ArbaEnskribo::Dosiero {
            haketo: blob_haketo,
            grandeco,
            ..
        } = &subarbo.enskriboj()[0]
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

    fn komiti(deponejo: &Deponejo) {
        let komito = Komito {
            arbo: deponejo.kapti_arbon().unwrap(),
            gepatro: deponejo.legi_head().unwrap(),
            mesagxo: "testo".into(),
            kreita_je: 1,
        };
        let haketo = deponejo.konservi_komiton(&komito).unwrap();
        deponejo.skribi_head(&haketo).unwrap();
    }

    fn malnovigi(vojo: &Path) -> SystemTime {
        let tempo = SystemTime::now() - Duration::from_secs(3600);
        fs::File::options()
            .write(true)
            .open(vojo)
            .unwrap()
            .set_modified(tempo)
            .unwrap();
        tempo
    }

    #[test]
    fn testo_stato_sen_komito() {
        let prov = ProvDeponejo::nova("stato-nova");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("b.txt"), b"b").unwrap();
        fs::create_dir(prov.radiko.join("sub")).unwrap();
        fs::write(prov.radiko.join("sub").join("a.txt"), b"a").unwrap();

        let stato = deponejo.stato().unwrap();
        assert_eq!(stato.novaj, ["b.txt", "sub/a.txt"]);
        assert!(stato.modifitaj.is_empty() && stato.forigitaj.is_empty());
    }

    #[test]
    fn testo_stato_detektas_sxangxojn() {
        let prov = ProvDeponejo::nova("stato-sxangxoj");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::create_dir(prov.radiko.join("sub")).unwrap();
        fs::write(prov.radiko.join("restanta.txt"), b"sama").unwrap();
        fs::write(prov.radiko.join("sub").join("modifota.txt"), b"unua").unwrap();
        fs::write(prov.radiko.join("forigota.txt"), b"for").unwrap();
        komiti(&deponejo);
        assert!(deponejo.stato().unwrap().estas_pura());

        fs::write(prov.radiko.join("sub").join("modifota.txt"), b"dua!").unwrap();
        fs::remove_file(prov.radiko.join("forigota.txt")).unwrap();
        fs::write(prov.radiko.join("sub").join("nova.txt"), b"nova").unwrap();

        let stato = deponejo.stato().unwrap();
        assert_eq!(
            stato,
            Stato {
                modifitaj: vec!["sub/modifota.txt".into()],
                novaj: vec!["sub/nova.txt".into()],
                forigitaj: vec!["forigota.txt".into()],
            }
        );
    }

    #[test]
    fn testo_stato_uzas_kasxmemoron() {
        let prov = ProvDeponejo::nova("stato-kasxmemoro");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a.txt");
        fs::write(&dosiero, b"unua").unwrap();
        let tempo = malnovigi(&dosiero);
        komiti(&deponejo);

        assert!(deponejo.stato().unwrap().estas_pura());
        let indekso = deponejo.legi_indekson().unwrap();
        assert_eq!(indekso.enskriboj().len(), 1);
        assert_eq!(indekso.enskriboj()[0].vojo, "a.txt");

        // Sama grandeco kaj restarigita mtime: la kaŝmemoro respondas sen relegado.
        fs::write(&dosiero, b"dua!").unwrap();
        fs::File::options()
            .write(true)
            .open(&dosiero)
            .unwrap()
            .set_modified(tempo)
            .unwrap();
        assert!(deponejo.stato().unwrap().estas_pura());

        // Nova mtime nuligas la kaŝmemoron.
        malnovigi(&dosiero);
        assert_eq!(deponejo.stato().unwrap().modifitaj, ["a.txt"]);
    }

    #[test]
    fn testo_stato_ne_kasxmemoras_fresxajn_dosierojn() {
        let prov = ProvDeponejo::nova("stato-fresxa");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a.txt");
        fs::write(&dosiero, b"unua").unwrap();
        komiti(&deponejo);

        assert!(deponejo.stato().unwrap().estas_pura());
        assert_eq!(deponejo.legi_indekson().unwrap().enskriboj(), []);

        // Sama grandeco ene de la sama tempero: devas esti detektita.
        fs::write(&dosiero, b"dua!").unwrap();
        assert_eq!(deponejo.stato().unwrap().modifitaj, ["a.txt"]);
    }

    #[test]
    fn testo_stato_rekonstruas_difektitan_indekson() {
        let prov = ProvDeponejo::nova("stato-difekto");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a.txt");
        fs::write(&dosiero, b"enhavo").unwrap();
        malnovigi(&dosiero);
        komiti(&deponejo);

        let indeksa_vojo = prov.radiko.join(VS_DOSIERUJO).join(INDEKSO_DOSIERO);
        fs::write(&indeksa_vojo, b"rubo").unwrap();
        assert!(deponejo.stato().unwrap().estas_pura());
        assert_eq!(deponejo.legi_indekson().unwrap().enskriboj().len(), 1);
    }

    #[test]
    fn testo_komito_reuzas_indekson() {
        let prov = ProvDeponejo::nova("komito-kasxmemoro");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a.txt");
        fs::write(&dosiero, b"unua").unwrap();
        let tempo = malnovigi(&dosiero);
        komiti(&deponejo);

        let unua = deponejo.legi_head().unwrap().unwrap();
        let arbo1 = deponejo.legi_komiton(&unua).unwrap().arbo;

        // Sama grandeco kaj restarigita mtime: la komito reuzas la indeksan
        // haketon anstataŭ relegi la dosieron, do la arbo restas la sama.
        fs::write(&dosiero, b"alia").unwrap();
        fs::File::options()
            .write(true)
            .open(&dosiero)
            .unwrap()
            .set_modified(tempo)
            .unwrap();
        komiti(&deponejo);

        let dua = deponejo.legi_head().unwrap().unwrap();
        assert_eq!(deponejo.legi_komiton(&dua).unwrap().arbo, arbo1);
    }

    #[test]
    fn testo_komito_restarigas_mankantan_objekton() {
        let prov = ProvDeponejo::nova("komito-restarigo");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a.txt");
        fs::write(&dosiero, b"enhavo").unwrap();
        malnovigi(&dosiero);
        komiti(&deponejo);

        let head = deponejo.legi_head().unwrap().unwrap();
        let arbo = deponejo
            .legi_arbon(&deponejo.legi_komiton(&head).unwrap().arbo)
            .unwrap();
        let ArbaEnskribo::Dosiero { haketo, .. } = &arbo.enskriboj()[0] else {
            panic!("atendita dosiero");
        };
        let objekto = deponejo.stokejo().dosierujo().join(format!("{haketo}.lz4"));
        fs::remove_file(&objekto).unwrap();

        // La indekso ankoraŭ memoras la haketon, sed la objekto mankas:
        // la komito devas relegi kaj restarigi ĝin.
        komiti(&deponejo);
        assert!(
            objekto.is_file(),
            "la mankanta objekto devas esti restarigita"
        );
        assert_eq!(deponejo.stokejo().ricevi(haketo).unwrap(), b"enhavo");
    }

    #[test]
    fn testo_komito_konstruas_nestitan_arbon() {
        let prov = ProvDeponejo::nova("komito-nesto");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::create_dir_all(prov.radiko.join("a").join("b").join("c")).unwrap();
        fs::write(prov.radiko.join("a").join("1.txt"), b"unua").unwrap();
        fs::write(prov.radiko.join("a").join("b").join("2.txt"), b"dua").unwrap();
        fs::write(
            prov.radiko.join("a").join("b").join("c").join("3.txt"),
            b"tria",
        )
        .unwrap();
        komiti(&deponejo);

        let head = deponejo.legi_head().unwrap().unwrap();
        let arbo = deponejo.legi_komiton(&head).unwrap().arbo;
        let mut vojoj: Vec<String> = deponejo
            .platigi_arbon(&arbo, "")
            .unwrap()
            .into_iter()
            .map(|(vojo, _)| vojo)
            .collect();
        vojoj.sort_unstable();
        assert_eq!(vojoj, ["a/1.txt", "a/b/2.txt", "a/b/c/3.txt"]);
        assert!(deponejo.stato().unwrap().estas_pura());
    }
}
