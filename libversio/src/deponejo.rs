//! Deponeja metadatumaro kaj grafa administrado.
//!
//! La motoro estas `no_std`-kongrueca: ĝi aliras la laborspacon nur per la
//! trajtoj [`Laborspaco`] kaj [`Horlogo`], kaj la objektojn per [`Stokejo`].
//! La normbibliotekaj adaptiloj [`Dosierspaco`], [`DosieraKonservilo`] kaj
//! [`StdaHorlogo`] estas aldonataj nur per la trajto `std`.

use alloc::borrow::ToOwned;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Write as _;

use crate::eraroj::Eraro;
use crate::ignoro::Ignoro;
use crate::indekso::{IndeksEnskribo, Indekso};
use crate::laborspaco::{Horlogo, Laborspaco, Speco};
use crate::objektoj::{ArbaEnskribo, Arbo, Komito};
use crate::stokado::{Haketo, Stokejo};
use crate::vojo::{MAKSIMA_VOJO, validi_komponanton};

#[cfg(feature = "std")]
use crate::laborspaco::{Dosierspaco, StdaHorlogo};
#[cfg(feature = "std")]
use crate::stokado::DosieraKonservilo;
#[cfg(feature = "std")]
use std::path::PathBuf;

/// Nomo de la metadatumuja dosierujo.
pub const VS_DOSIERUJO: &str = ".vs";

/// Nomo de la objekta dosierujo.
pub const OBJEKTUJO: &str = "objektoj";

/// Nomo de la KAPO-dosiero.
pub const KAPO_DOSIERO: &str = "KAPO";

/// Nomo de la indeksa dosiero (la metadatuma kaŝmemoro).
pub const INDEKSO_DOSIERO: &str = "indekso";

const REFERENCOJ_DOSIERUJO: &str = "referencoj";
const BRANĈOJ_DOSIERUJO: &str = "branĉoj";
const ETIKEDOJ_DOSIERUJO: &str = "etikedoj";
const NUNA_BRANĈO_DOSIERO: &str = "branĉo";
const MOVHISTORIO_DOSIERO: &str = "movhistorio";

/// Dosieroj modifitaj malpli ol tiom antaŭ la skanado ne estas kaŝmemorataj.
///
/// Konkurenca Git-problemo: dosiersistemaj tempstampoj havas krudan precizecon, do
/// dua skribo en la sama tempero povus konservi la saman mtempo kaj grandecon.
/// Nur sufiĉe malnovaj tempstampoj garantias, ke posta ŝanĝo videblas.
const FREŜA_MARGENO_NS: u64 = 2_000_000_000;

/// La ŝanĝoj de la laborspaco rilate al la arbo de `KAPO`.
///
/// Ĉiu listo enhavas relativajn vojojn kun `/`, ordigitajn laŭ bajtoj.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stato {
    /// Dosieroj, kies enhavo diferencas de `KAPO`.
    pub modifitaj: Vec<String>,
    /// Dosieroj, kiuj ne ekzistas en `KAPO`.
    pub novaj: Vec<String>,
    /// Dosieroj de `KAPO`, kiuj ne plu ekzistas en la laborspaco.
    pub forigitaj: Vec<String>,
}

impl Stato {
    /// Ĉu la laborspaco kongruas kun `KAPO`.
    #[must_use]
    pub fn estas_pura(&self) -> bool {
        self.modifitaj.is_empty() && self.novaj.is_empty() && self.forigitaj.is_empty()
    }
}

struct LaborDosiero {
    vojo: String,
    grandeco: u64,
    mtempo: Option<u64>,
}

/// Dosieruja nodo por konstrui la arbaron el plata listo de dosieroj.
#[derive(Default)]
struct Nodo {
    dosieroj: Vec<(String, Haketo, u64)>,
    subarboj: BTreeMap<String, Nodo>,
}

struct HaketitaDosiero {
    vojo: String,
    haketo: Haketo,
    grandeco: u64,
    /// `(mtempo, grandeco)`, se la dosiero estas sufiĉe malnova por la kaŝmemoro.
    kaŝebla: Option<(u64, u64)>,
}

/// La rezulto de kunfando.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kunfando {
    /// La celo jam estas en la historio de la nuna komito; nenio ŝanĝiĝis.
    Nenio,
    /// La nuna komito estas antaŭulo de la celo; la branĉo simple avancis.
    Antauen(Haketo),
    /// Estis kreita nova kunfanda komito kun du gepatroj.
    Kunfandita(Haketo),
}

/// La celo de etikedo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EtikedaCelo {
    /// La etikedo montras rekte al komito.
    Komito(Haketo),
    /// La etikedo montras al alia etikedo.
    Alias(String),
}

/// Etikedo: nomo kaj la celo, al kiun ĝi montras.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Etikedo {
    /// La nomo de la etikedo.
    pub nomo: String,
    /// La celo de la etikedo.
    pub celo: EtikedaCelo,
}

/// Referenco, kies movoj estas registrataj en la movhistorio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Referenco {
    /// La pinta referenco `KAPO`.
    Kapo,
    /// Branĉo kun la donita nomo.
    Branĉo(String),
}

impl Referenco {
    /// La homa nomo de la referenco.
    #[must_use]
    pub fn nomo(&self) -> &str {
        match self {
            Self::Kapo => "KAPO",
            Self::Branĉo(nomo) => nomo,
        }
    }

    fn kodi(&self) -> String {
        match self {
            Self::Kapo => String::from("kapo"),
            Self::Branĉo(nomo) => format!("branĉo:{nomo}"),
        }
    }

    fn malkodi(teksto: &str) -> Result<Self, Eraro> {
        if teksto == "kapo" {
            return Ok(Self::Kapo);
        }
        let nomo = teksto
            .strip_prefix("branĉo:")
            .ok_or(Eraro::NevalidaDatumo {
                kialo: "nekonata referenco en la movhistorio",
            })?;
        Ok(Self::Branĉo(nomo.to_owned()))
    }
}

/// La speco de movo registrita en la movhistorio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ago {
    /// Nova komito sur la nuna branĉo.
    Komiti,
    /// Eltiro de komito.
    Eltiri,
    /// Kunfando.
    Kunfandi,
    /// Kreado de branĉo.
    Krei,
    /// Renomo de branĉo.
    Renomi,
    /// Forigo de branĉo.
    Forigi,
}

impl Ago {
    /// La homa priskribo de la movo.
    #[must_use]
    pub fn priskribo(self) -> &'static str {
        match self {
            Self::Komiti => "komito",
            Self::Eltiri => "eltiro",
            Self::Kunfandi => "kunfando",
            Self::Krei => "kreado",
            Self::Renomi => "renomo",
            Self::Forigi => "forigo",
        }
    }

    fn kodi(self) -> &'static str {
        match self {
            Self::Komiti => "komiti",
            Self::Eltiri => "eltiri",
            Self::Kunfandi => "kunfandi",
            Self::Krei => "krei",
            Self::Renomi => "renomi",
            Self::Forigi => "forigi",
        }
    }

    fn malkodi(teksto: &str) -> Result<Self, Eraro> {
        match teksto {
            "komiti" => Ok(Self::Komiti),
            "eltiri" => Ok(Self::Eltiri),
            "kunfandi" => Ok(Self::Kunfandi),
            "krei" => Ok(Self::Krei),
            "renomi" => Ok(Self::Renomi),
            "forigi" => Ok(Self::Forigi),
            _ => Err(Eraro::NevalidaDatumo {
                kialo: "nekonata movospeco en la movhistorio",
            }),
        }
    }
}

/// Unu movo de referenco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movo {
    /// Uniksa tempstampo de la movo, en sekundoj.
    pub tempo: u64,
    /// La movita referenco.
    pub celo: Referenco,
    /// La speco de la movo.
    pub ago: Ago,
    /// La antaŭa valoro, se ekzistis.
    pub malnova: Option<Haketo>,
    /// La nova valoro, se ĝi restas.
    pub nova: Option<Haketo>,
}

/// La movhistorio: apenda listo de referencaj movoj.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Movhistorio {
    movoj: Vec<Movo>,
}

impl Movhistorio {
    /// La registritaj movoj, de la plej malnova al la plej nova.
    #[must_use]
    pub fn movoj(&self) -> &[Movo] {
        &self.movoj
    }

    fn seriigi(&self) -> String {
        let mut teksto = String::new();
        for movo in &self.movoj {
            let _ = writeln!(
                teksto,
                "{} {} {} {} {}",
                movo.tempo,
                movo.celo.kodi(),
                movo.ago.kodi(),
                kodi_haketon(movo.malnova),
                kodi_haketon(movo.nova),
            );
        }
        teksto
    }

    fn malseriigi(teksto: &str) -> Result<Self, Eraro> {
        let mut movoj = Vec::new();
        for linio in teksto.lines() {
            let linio = linio.trim();
            if linio.is_empty() {
                continue;
            }
            let kampoj: Vec<&str> = linio.split_whitespace().collect();
            let [tempo, celo, ago, malnova, nova] = kampoj.as_slice() else {
                return Err(Eraro::NevalidaDatumo {
                    kialo: "movhistoria linio ne havas kvin kampojn",
                });
            };
            movoj.push(Movo {
                tempo: tempo.parse().map_err(|_| Eraro::NevalidaDatumo {
                    kialo: "nevalida tempstampo en la movhistorio",
                })?,
                celo: Referenco::malkodi(celo)?,
                ago: Ago::malkodi(ago)?,
                malnova: malkodi_haketon(malnova)?,
                nova: malkodi_haketon(nova)?,
            });
        }
        Ok(Self { movoj })
    }
}

fn kodi_haketon(haketo: Option<Haketo>) -> String {
    haketo.map_or_else(|| String::from("-"), |haketo| haketo.to_string())
}

fn malkodi_haketon(teksto: &str) -> Result<Option<Haketo>, Eraro> {
    if teksto == "-" {
        Ok(None)
    } else {
        Ok(Some(teksto.parse()?))
    }
}

/// La rezulto de rubokolekto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rubokolekto {
    /// La nombro da forigitaj objektoj.
    pub forigitaj: usize,
    /// La nombro da konservitaj objektoj.
    pub konservitaj: usize,
}

/// Deponeja motoro super ajna [`Laborspaco`], [`Stokejo`] kaj [`Horlogo`].
pub struct DeponejoMotor<L, K, H> {
    laborspaco: L,
    stokejo: K,
    horlogo: H,
}

/// La norma dosiersistema deponejo.
#[cfg(feature = "std")]
pub type Deponejo = DeponejoMotor<Dosierspaco, DosieraKonservilo, StdaHorlogo>;

impl<L, K, H> DeponejoMotor<L, K, H>
where
    L: Laborspaco,
    K: Stokejo,
    H: Horlogo,
{
    /// Kreas motoron el jam pretaj laborspaco, stokejo kaj horloĝo.
    ///
    /// La konstruanto ne kreas `.vs`-strukturon; por tio uzu
    /// [`Deponejo::iniciati`] aŭ kreu la metadatumojn mem.
    #[must_use]
    pub fn nova(laborspaco: L, stokejo: K, horlogo: H) -> Self {
        Self {
            laborspaco,
            stokejo,
            horlogo,
        }
    }

    /// La radika dosierujo de la deponejo.
    #[must_use]
    pub fn radiko(&self) -> &str {
        self.laborspaco.radiko()
    }

    /// La objektstokejo de la deponejo.
    #[must_use]
    pub fn stokejo(&self) -> &K {
        &self.stokejo
    }

    /// La laborspaco de la deponejo.
    #[must_use]
    pub fn laborspaco(&self) -> &L {
        &self.laborspaco
    }

    /// La horloĝo de la deponejo.
    #[must_use]
    pub fn horlogo(&self) -> &H {
        &self.horlogo
    }

    fn meta(relativa: &str) -> String {
        format!("{VS_DOSIERUJO}/{relativa}")
    }

    fn skribi_atome(&self, vojo: &str, datumoj: &[u8]) -> Result<(), Eraro> {
        self.laborspaco.skribi_atome(vojo, datumoj)
    }

    fn limo(&self) -> Option<u64> {
        self.horlogo
            .nunaj_nanosekundoj()
            .and_then(|nunaj| nunaj.checked_sub(FREŜA_MARGENO_NS))
    }

    /// Legas la haketon en `KAPO`; `None` signifas, ke ankoraŭ ne ekzistas komito.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron aŭ [`Eraro::NevalidaHaketo`], se `KAPO`
    /// enhavas nevalidan tekston.
    pub fn legi_kapon(&self) -> Result<Option<Haketo>, Eraro> {
        let vojo = Self::meta(KAPO_DOSIERO);
        let Some(teksto) = self.laborspaco.legi_tekston(&vojo)? else {
            return Ok(None);
        };
        let teksto = teksto.trim();
        if teksto.is_empty() {
            Ok(None)
        } else {
            Ok(Some(teksto.parse()?))
        }
    }

    /// Skribas la donitan komiton kiel la novan `KAPO`.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron.
    pub fn skribi_kapon(&self, haketo: &Haketo) -> Result<(), Eraro> {
        self.skribi_atome(&Self::meta(KAPO_DOSIERO), haketo.to_string().as_bytes())
    }

    fn branĉo_vojo(nomo: &str) -> String {
        Self::meta(&format!(
            "{REFERENCOJ_DOSIERUJO}/{BRANĈOJ_DOSIERUJO}/{nomo}"
        ))
    }

    /// Legas la celan komiton de la donita branĉo; `None` signifas, ke ĝi
    /// ankoraŭ ne ekzistas.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaVojaKomponanto`], se la nomo estas malvalida,
    /// aŭ laborspacan eraron.
    pub fn legi_branĉon(&self, nomo: &str) -> Result<Option<Haketo>, Eraro> {
        validi_referencan_nomon(nomo)?;
        match self.laborspaco.legi_tekston(&Self::branĉo_vojo(nomo))? {
            Some(teksto) => Ok(Some(teksto.trim().parse()?)),
            None => Ok(None),
        }
    }

    /// Skribas la celan komiton de la donita branĉo.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaVojaKomponanto`], se la nomo estas malvalida,
    /// aŭ laborspacan eraron.
    pub fn skribi_branĉon(&self, nomo: &str, haketo: &Haketo) -> Result<(), Eraro> {
        validi_referencan_nomon(nomo)?;
        self.laborspaco.krei_dosierujon(&Self::meta(&format!(
            "{REFERENCOJ_DOSIERUJO}/{BRANĈOJ_DOSIERUJO}"
        )))?;
        self.skribi_atome(&Self::branĉo_vojo(nomo), haketo.to_string().as_bytes())
    }

    /// Listigas la branĉojn kaj iliajn celajn komitojn, ordigitajn laŭ nomo.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan aŭ haketan eraron por difektita referenco.
    pub fn branĉoj(&self) -> Result<Vec<(String, Haketo)>, Eraro> {
        let dosierujo = Self::meta(&format!("{REFERENCOJ_DOSIERUJO}/{BRANĈOJ_DOSIERUJO}"));
        let mut branĉoj = Vec::new();
        for ero in self.laborspaco.listi(&dosierujo)? {
            if ero.speco != Speco::Dosiero || ero.nomo.starts_with('.') {
                continue;
            }
            let vojo = kunigi(&dosierujo, &ero.nomo)?;
            let Some(teksto) = self.laborspaco.legi_tekston(&vojo)? else {
                continue;
            };
            branĉoj.push((ero.nomo, teksto.trim().parse()?));
        }
        branĉoj.sort_unstable_by(|unu, du| unu.0.cmp(&du.0));
        Ok(branĉoj)
    }

    /// La nomo de la nuna branĉo; `None` signifas detaĉitan `KAPO`.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron.
    pub fn nuna_branĉo(&self) -> Result<Option<String>, Eraro> {
        match self
            .laborspaco
            .legi_tekston(&Self::meta(NUNA_BRANĈO_DOSIERO))?
        {
            Some(teksto) => {
                let nomo = teksto.trim();
                if nomo.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(nomo.to_owned()))
                }
            }
            None => Ok(None),
        }
    }

    /// Memorigas, ke `KAPO` sekvas la donitan branĉon.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaVojaKomponanto`], se la nomo estas malvalida,
    /// aŭ laborspacan eraron.
    pub fn agordi_nunan_branĉon(&self, nomo: &str) -> Result<(), Eraro> {
        validi_referencan_nomon(nomo)?;
        self.skribi_atome(&Self::meta(NUNA_BRANĈO_DOSIERO), nomo.as_bytes())
    }

    /// Forigas la memoron pri la nuna branĉo (detaĉita `KAPO`).
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, escepte de mankanta dosiero.
    pub fn forigi_nunan_branĉon(&self) -> Result<(), Eraro> {
        self.laborspaco.forigi(&Self::meta(NUNA_BRANĈO_DOSIERO))
    }

    /// Skribas novan `KAPO`-valoron kaj registras la movon.
    ///
    /// Ripeta movo al la sama valoro ne registriĝas.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan aŭ haketan eraron.
    pub fn movi_kapon(&self, nova: &Haketo, ago: Ago) -> Result<(), Eraro> {
        let malnova = self.legi_kapon()?;
        if malnova.as_ref() == Some(nova) {
            return Ok(());
        }
        self.skribi_kapon(nova)?;
        self.registri_movon(Movo {
            tempo: self.horlogo.nunaj_sekundoj(),
            celo: Referenco::Kapo,
            ago,
            malnova,
            nova: Some(*nova),
        })
    }

    /// Skribas novan branĉan valoron kaj registras la movon.
    ///
    /// Ripeta movo al la sama valoro ne registriĝas.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan aŭ haketan eraron.
    pub fn movi_branĉon(&self, nomo: &str, nova: &Haketo, ago: Ago) -> Result<(), Eraro> {
        let malnova = self.legi_branĉon(nomo)?;
        if malnova.as_ref() == Some(nova) {
            return Ok(());
        }
        self.skribi_branĉon(nomo, nova)?;
        self.registri_movon(Movo {
            tempo: self.horlogo.nunaj_sekundoj(),
            celo: Referenco::Branĉo(nomo.to_owned()),
            ago,
            malnova,
            nova: Some(*nova),
        })
    }

    /// Renomas branĉon, konservante ĝian celon.
    ///
    /// Se la renomata branĉo estas la nuna, la nova nomo iĝas la nuna.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NeekzistantaBranĉo`] aŭ [`Eraro::BranĉoEkzistas`],
    /// se la nomoj ne kongruas kun la stato, aŭ laborspacan eraron.
    pub fn renomi_branĉon(&self, malnova: &str, nova: &str) -> Result<(), Eraro> {
        validi_referencan_nomon(malnova)?;
        validi_referencan_nomon(nova)?;
        if malnova == nova {
            return Ok(());
        }
        let celo = self
            .legi_branĉon(malnova)?
            .ok_or_else(|| Eraro::NeekzistantaBranĉo {
                nomo: malnova.to_owned(),
            })?;
        if self.legi_branĉon(nova)?.is_some() {
            return Err(Eraro::BranĉoEkzistas {
                nomo: nova.to_owned(),
            });
        }
        self.laborspaco
            .renomi(&Self::branĉo_vojo(malnova), &Self::branĉo_vojo(nova))?;
        if self.nuna_branĉo()?.as_deref() == Some(malnova) {
            self.agordi_nunan_branĉon(nova)?;
        }
        self.registri_movon(Movo {
            tempo: self.horlogo.nunaj_sekundoj(),
            celo: Referenco::Branĉo(nova.to_owned()),
            ago: Ago::Renomi,
            malnova: Some(celo),
            nova: Some(celo),
        })
    }

    /// Forigas branĉon.
    ///
    /// Se la forigata branĉo estis la nuna, `KAPO` restas detaĉita.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NeekzistantaBranĉo`], se la branĉo mankas, aŭ
    /// laborspacan eraron.
    pub fn forigi_branĉon(&self, nomo: &str) -> Result<(), Eraro> {
        validi_referencan_nomon(nomo)?;
        let malnova = self
            .legi_branĉon(nomo)?
            .ok_or_else(|| Eraro::NeekzistantaBranĉo {
                nomo: nomo.to_owned(),
            })?;
        self.laborspaco.forigi(&Self::branĉo_vojo(nomo))?;
        if self.nuna_branĉo()?.as_deref() == Some(nomo) {
            self.forigi_nunan_branĉon()?;
        }
        self.registri_movon(Movo {
            tempo: self.horlogo.nunaj_sekundoj(),
            celo: Referenco::Branĉo(nomo.to_owned()),
            ago: Ago::Forigi,
            malnova: Some(malnova),
            nova: None,
        })
    }

    fn etikedo_vojo(nomo: &str) -> String {
        Self::meta(&format!(
            "{REFERENCOJ_DOSIERUJO}/{ETIKEDOJ_DOSIERUJO}/{nomo}"
        ))
    }

    /// Legas la celon de etikedo; `None` signifas, ke ĝi ne ekzistas.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaVojaKomponanto`] por malvalida nomo, aŭ
    /// laborspacan aŭ malseriigan eraron.
    pub fn legi_etikedon(&self, nomo: &str) -> Result<Option<EtikedaCelo>, Eraro> {
        validi_referencan_nomon(nomo)?;
        let Some(teksto) = self.laborspaco.legi_tekston(&Self::etikedo_vojo(nomo))? else {
            return Ok(None);
        };
        let mut kampoj = teksto.split_whitespace();
        let speco = kampoj.next().ok_or(Eraro::NevalidaDatumo {
            kialo: "malplena etikeda dosiero",
        })?;
        let valoro = kampoj.next().ok_or(Eraro::NevalidaDatumo {
            kialo: "etikeda dosiero sen celo",
        })?;
        if kampoj.next().is_some() {
            return Err(Eraro::NevalidaDatumo {
                kialo: "etikeda dosiero kun superfluaj kampoj",
            });
        }
        match speco {
            "komito" => Ok(Some(EtikedaCelo::Komito(valoro.parse()?))),
            "etikedo" => {
                validi_referencan_nomon(valoro)?;
                Ok(Some(EtikedaCelo::Alias(valoro.to_owned())))
            }
            _ => Err(Eraro::NevalidaDatumo {
                kialo: "nekonata etikedspeco",
            }),
        }
    }

    /// Skribas etikedon; ekzistanta samnoma etikedo estas anstataŭigita.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaVojaKomponanto`] por malvalida nomo, aŭ
    /// laborspacan eraron.
    pub fn skribi_etikedon(&self, nomo: &str, celo: &EtikedaCelo) -> Result<(), Eraro> {
        validi_referencan_nomon(nomo)?;
        let teksto = match celo {
            EtikedaCelo::Komito(haketo) => format!("komito {haketo}"),
            EtikedaCelo::Alias(aliaso) => {
                validi_referencan_nomon(aliaso)?;
                format!("etikedo {aliaso}")
            }
        };
        self.laborspaco.krei_dosierujon(&Self::meta(&format!(
            "{REFERENCOJ_DOSIERUJO}/{ETIKEDOJ_DOSIERUJO}"
        )))?;
        self.skribi_atome(&Self::etikedo_vojo(nomo), teksto.as_bytes())
    }

    /// Listigas la etikedojn, ordigitajn laŭ nomo.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan aŭ malseriigan eraron por difektita etikedo.
    pub fn etikedoj(&self) -> Result<Vec<Etikedo>, Eraro> {
        let dosierujo = Self::meta(&format!("{REFERENCOJ_DOSIERUJO}/{ETIKEDOJ_DOSIERUJO}"));
        let mut etikedoj = Vec::new();
        for ero in self.laborspaco.listi(&dosierujo)? {
            if ero.speco != Speco::Dosiero || ero.nomo.starts_with('.') {
                continue;
            }
            if let Some(celo) = self.legi_etikedon(&ero.nomo)? {
                etikedoj.push(Etikedo {
                    nomo: ero.nomo,
                    celo,
                });
            }
        }
        etikedoj.sort_unstable_by(|unu, du| unu.nomo.cmp(&du.nomo));
        Ok(etikedoj)
    }

    /// Solvas etikedon al komito, sekvante aliaskĉenon.
    ///
    /// `None` signifas, ke la etikedo ne ekzistas.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NeekzistantaEtikedo`], se la ĉeno montras al mankanta
    /// etikedo, [`Eraro::CiklaEtikedo`] por ciklo, aŭ laborspacan eraron.
    pub fn solvi_etikedon(&self, nomo: &str) -> Result<Option<Haketo>, Eraro> {
        let mut vizititaj = BTreeSet::new();
        let mut nuna = nomo.to_owned();
        loop {
            if !vizititaj.insert(nuna.clone()) {
                return Err(Eraro::CiklaEtikedo { nomo: nuna });
            }
            let Some(celo) = self.legi_etikedon(&nuna)? else {
                return if nuna == nomo {
                    Ok(None)
                } else {
                    Err(Eraro::NeekzistantaEtikedo { nomo: nuna })
                };
            };
            match celo {
                EtikedaCelo::Komito(haketo) => return Ok(Some(haketo)),
                EtikedaCelo::Alias(sekva) => nuna = sekva,
            }
        }
    }

    /// Legas la movhistorion; mankanta dosiero estas malplena historio.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan aŭ malseriigan eraron.
    pub fn movhistorio(&self) -> Result<Movhistorio, Eraro> {
        match self
            .laborspaco
            .legi_tekston(&Self::meta(MOVHISTORIO_DOSIERO))?
        {
            Some(teksto) => Movhistorio::malseriigi(&teksto),
            None => Ok(Movhistorio::default()),
        }
    }

    fn registri_movon(&self, movo: Movo) -> Result<(), Eraro> {
        let mut historio = self.movhistorio()?;
        historio.movoj.push(movo);
        self.skribi_atome(
            &Self::meta(MOVHISTORIO_DOSIERO),
            historio.seriigi().as_bytes(),
        )
    }

    /// Forigas ĉiujn objektojn, kiujn neniu referenco atingas.
    ///
    /// La radikoj estas `KAPO`, la branĉoj, la etikedoj kaj ambaŭ valoroj de
    /// ĉiu movhistoria enskribo. La movhistorio do retenas la antaŭajn
    /// valorojn ĝis la uzanto forigos ĝin permane.
    ///
    /// # Errors
    ///
    /// Redonas stokejan aŭ laborspacan eraron.
    pub fn rubokolekti(&self) -> Result<Rubokolekto, Eraro> {
        let mut stako = Vec::new();
        stako.extend(self.legi_kapon()?);
        stako.extend(self.branĉoj()?.into_iter().map(|(_, haketo)| haketo));
        for etikedo in self.etikedoj()? {
            stako.extend(self.solvi_etikedon(&etikedo.nomo)?);
        }
        for movo in self.movhistorio()?.movoj() {
            stako.extend(movo.malnova);
            stako.extend(movo.nova);
        }

        let mut atengaj = BTreeSet::new();
        while let Some(haketo) = stako.pop() {
            if !atengaj.insert(haketo) {
                continue;
            }
            let datumoj = self.stokejo.ricevi(&haketo)?;
            if let Ok(komito) = Komito::malseriigi(&datumoj) {
                stako.extend(komito.gepatro);
                stako.extend(komito.dua_gepatro);
                stako.push(komito.arbo);
            } else if let Ok(arbo) = Arbo::malseriigi(&datumoj) {
                for enskribo in arbo.enskriboj() {
                    match enskribo {
                        ArbaEnskribo::Dosiero { haketo, .. }
                        | ArbaEnskribo::SubArbo { haketo, .. } => stako.push(*haketo),
                    }
                }
            }
        }

        let mut raporto = Rubokolekto {
            forigitaj: 0,
            konservitaj: 0,
        };
        for haketo in self.stokejo.haketoj()? {
            if atengaj.contains(&haketo) {
                raporto.konservitaj = raporto.konservitaj.saturating_add(1);
            } else {
                self.stokejo.forigi(&haketo)?;
                raporto.forigitaj = raporto.forigitaj.saturating_add(1);
            }
        }
        Ok(raporto)
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

    /// Rekonstruas la laborspacon el la arbo de la donita komito.
    ///
    /// Dosieroj, kies enhavo jam kongruas, restas netuŝitaj; la ceteraj estas
    /// skribataj el la stokejo. Dosieroj, kiuj ne apartenas al la celata arbo,
    /// estas forigitaj, kaj malplenaj dosierujoj estas forprenitaj. La indeksa
    /// kaŝmemoro estas forviŝita, ĉar la metadatumoj de la laborspaco ŝanĝiĝis.
    /// `KAPO` ne estas ĝisdatigata; tion faras la alvokanto.
    ///
    /// # Errors
    ///
    /// Redonas stokejan, malseriigan aŭ laborspacan eraron.
    pub fn eltiri(&self, komito: &Haketo) -> Result<(), Eraro> {
        let arbo = self.legi_komiton(komito)?.arbo;
        let celo: BTreeMap<String, Haketo> = self
            .platigi_arbon(&arbo, "")?
            .into_iter()
            .map(|(vojo, haketo, _)| (vojo, haketo))
            .collect();

        let limo = self.limo();
        let indekso = self.legi_indekson()?;
        let laboraj = self
            .skani("")?
            .into_iter()
            .map(|dosiero| self.haketi_laboran(dosiero, &indekso, limo))
            .collect::<Result<Vec<_>, Eraro>>()?;
        let nunaj: BTreeMap<&str, Haketo> = laboraj
            .iter()
            .map(|dosiero| (dosiero.vojo.as_str(), dosiero.haketo))
            .collect();

        // Unue forigu la superfluajn dosierojn, por ke dosiero kaj dosierujo
        // kun la sama nomo ne koliziu dum la skribado.
        for dosiero in &laboraj {
            if celo.contains_key(&dosiero.vojo) {
                continue;
            }
            self.laborspaco.forigi(&dosiero.vojo)?;
        }
        self.forigi_malplenajn_dosierujojn("", &Ignoro::defaŭlta())?;

        for (vojo, haketo) in &celo {
            if nunaj.get(vojo.as_str()) == Some(haketo) {
                continue;
            }
            let datumoj = self.stokejo.ricevi(haketo)?;
            if let Some(parent) = parento(vojo) {
                self.laborspaco.krei_dosierujon(parent)?;
            }
            self.laborspaco.skribi_atome(vojo, &datumoj)?;
        }

        self.laborspaco.forigi(&Self::meta(INDEKSO_DOSIERO))
    }

    /// Kunfandas la celan komiton en la nunan.
    ///
    /// Se la celo jam estas en la historio de la nuna komito, nenio ŝanĝiĝas.
    /// Se la nuna komito estas antaŭulo de la celo, la operacio antaŭeniras
    /// sen nova komito. Alie ĝi trovas komunan antaŭulon kaj tri-vojon
    /// kunfandas la platajn arbojn. Post sukceso la laborspaco spegulas la
    /// rezulton; la referencoj kaj `KAPO` restas al la alvokanto.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::KunfandaKonflikto`], se ambaŭ flankoj malsame ŝanĝis
    /// la saman vojon, [`Eraro::NesamajHistorioj`], se mankas komuna antaŭulo,
    /// aŭ stokejan, malseriigan aŭ laborspacan eraron.
    pub fn kunfandi(
        &self,
        nuna: &Haketo,
        celo: &Haketo,
        mesaĝo: &str,
        kreita_je: u64,
    ) -> Result<Kunfando, Eraro> {
        if nuna == celo || self.estas_antaŭulo(celo, nuna)? {
            return Ok(Kunfando::Nenio);
        }
        if self.estas_antaŭulo(nuna, celo)? {
            self.eltiri(celo)?;
            return Ok(Kunfando::Antauen(*celo));
        }

        let bazo = self
            .komuna_gepatro(nuna, celo)?
            .ok_or(Eraro::NesamajHistorioj)?;
        let arbo = self.kunfandi_arbojn(&bazo, nuna, celo)?;
        let komito = Komito {
            arbo,
            gepatro: Some(*nuna),
            mesaĝo: mesaĝo.to_owned(),
            kreita_je,
            dua_gepatro: Some(*celo),
        };
        let haketo = self.konservi_komiton(&komito)?;
        self.eltiri(&haketo)?;
        Ok(Kunfando::Kunfandita(haketo))
    }

    /// Ĉu `antaŭulo` estas atingebla ekde `posteulo` sekvante gepatrojn.
    fn estas_antaŭulo(&self, antaŭulo: &Haketo, posteulo: &Haketo) -> Result<bool, Eraro> {
        let mut stako = vec![*posteulo];
        let mut vizititaj = BTreeSet::new();
        while let Some(haketo) = stako.pop() {
            if &haketo == antaŭulo {
                return Ok(true);
            }
            if !vizititaj.insert(haketo) {
                continue;
            }
            let komito = self.legi_komiton(&haketo)?;
            stako.extend(komito.gepatro);
            stako.extend(komito.dua_gepatro);
        }
        Ok(false)
    }

    /// Trovas komunan antaŭulon de la du komitoj, se ĝi ekzistas.
    fn komuna_gepatro(&self, unua: &Haketo, dua: &Haketo) -> Result<Option<Haketo>, Eraro> {
        let mut antaŭuloj = BTreeSet::new();
        let mut stako = vec![*unua];
        while let Some(haketo) = stako.pop() {
            if !antaŭuloj.insert(haketo) {
                continue;
            }
            let komito = self.legi_komiton(&haketo)?;
            stako.extend(komito.gepatro);
            stako.extend(komito.dua_gepatro);
        }

        let mut vizititaj = BTreeSet::new();
        let mut stako = vec![*dua];
        while let Some(haketo) = stako.pop() {
            if antaŭuloj.contains(&haketo) {
                return Ok(Some(haketo));
            }
            if !vizititaj.insert(haketo) {
                continue;
            }
            let komito = self.legi_komiton(&haketo)?;
            stako.extend(komito.gepatro);
            stako.extend(komito.dua_gepatro);
        }
        Ok(None)
    }

    /// Tri-voja kunfando de la plataj arboj; redonas la novan radikan arbon.
    fn kunfandi_arbojn(
        &self,
        bazo: &Haketo,
        nuna: &Haketo,
        celo: &Haketo,
    ) -> Result<Haketo, Eraro> {
        let bazo = self.plata_arbaro(&self.legi_komiton(bazo)?.arbo)?;
        let nuna = self.plata_arbaro(&self.legi_komiton(nuna)?.arbo)?;
        let celo = self.plata_arbaro(&self.legi_komiton(celo)?.arbo)?;

        let mut vojoj = BTreeSet::new();
        vojoj.extend(bazo.keys());
        vojoj.extend(nuna.keys());
        vojoj.extend(celo.keys());

        let mut rezulto = BTreeMap::new();
        let mut konfliktoj = Vec::new();
        for vojo in vojoj {
            let n = nuna.get(vojo);
            let t = celo.get(vojo);
            let b = bazo.get(vojo);
            let elektita = if sama_enhavo(n, t) {
                n.or(t).copied()
            } else if sama_enhavo(n, b) {
                t.copied()
            } else if sama_enhavo(t, b) {
                n.copied()
            } else {
                konfliktoj.push(vojo.clone());
                continue;
            };
            if let Some(enskribo) = elektita {
                rezulto.insert(vojo.clone(), enskribo);
            }
        }

        let mut antaŭa: Option<&String> = None;
        for vojo in rezulto.keys() {
            if let Some(antaŭa) = antaŭa
                && vojo.starts_with(antaŭa.as_str())
                && vojo.as_bytes().get(antaŭa.len()) == Some(&b'/')
            {
                konfliktoj.push(antaŭa.clone());
                konfliktoj.push(vojo.clone());
            }
            antaŭa = Some(vojo);
        }

        if !konfliktoj.is_empty() {
            konfliktoj.sort_unstable();
            konfliktoj.dedup();
            return Err(Eraro::KunfandaKonflikto { vojoj: konfliktoj });
        }

        let nodo = nodo_el_dosieroj(
            rezulto
                .iter()
                .map(|(vojo, (haketo, grandeco))| (vojo.as_str(), *haketo, *grandeco)),
        );
        self.konservi_nodon(&nodo)
    }

    /// Legas arbon kiel mapon de vojoj al dosieraj haketoj kaj grandecoj.
    fn plata_arbaro(&self, haketo: &Haketo) -> Result<BTreeMap<String, (Haketo, u64)>, Eraro> {
        Ok(self
            .platigi_arbon(haketo, "")?
            .into_iter()
            .map(|(vojo, haketo, grandeco)| (vojo, (haketo, grandeco)))
            .collect())
    }

    /// Kaptas la nunan laborspacon kaj redonas la haketon de la radika arbo.
    ///
    /// Dosieroj, kies mtempo kaj grandeco kongruas kun la indekso kaj kies
    /// objekto jam ekzistas en la stokejo, estas reuzataj sen relegado; nur
    /// ŝanĝitaj aŭ novaj dosieroj estas relegataj kaj kunpremataj. Post la
    /// kaptado la indekso estas ĝisdatigita, same kiel ĉe
    /// [`DeponejoMotor::stato`]. `.git`, `target`, `.vs` kaj la reguloj de
    /// `.vsignore` estas preterlasataj.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, [`Eraro::NevalidaDosiernomo`],
    /// [`Eraro::NevalidaVojaKomponanto`] aŭ [`Eraro::NesubtenitaDosiertipo`]
    /// por la laborspaco, aŭ stokejan eraron.
    pub fn kapti_arbon(&self) -> Result<Haketo, Eraro> {
        let limo = self.limo();
        let malnova = self.legi_indekson()?;
        let haketitaj = self
            .skani("")?
            .into_iter()
            .map(|dosiero| self.konservi_laboran(dosiero, &malnova, limo))
            .collect::<Result<Vec<_>, Eraro>>()?;
        self.ĝisdatigi_indekson(&malnova, &haketitaj)?;

        self.konservi_nodon(&nodo_el_dosieroj(
            haketitaj
                .iter()
                .map(|dosiero| (dosiero.vojo.as_str(), dosiero.haketo, dosiero.grandeco)),
        ))
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
            && let Some(haketo) = indekso.kaŝita_haketo(&dosiero.vojo, mtempo, dosiero.grandeco)
            && self.stokejo.havas(&haketo)
        {
            return Ok(HaketitaDosiero {
                vojo: dosiero.vojo,
                haketo,
                grandeco: dosiero.grandeco,
                kaŝebla: Some((mtempo, dosiero.grandeco)),
            });
        }

        let datumoj = self.laborspaco.legi(&dosiero.vojo)?;
        let haketo = self.stokejo.konservi(&datumoj)?;
        let grandeco = u64::try_from(datumoj.len()).map_err(|_| Eraro::TrolongaEnigo {
            maksimumo: usize::MAX,
            akirita: datumoj.len(),
        })?;
        let kaŝebla = mtempo
            .filter(|_| grandeco == dosiero.grandeco)
            .map(|mtempo| (mtempo, grandeco));
        Ok(HaketitaDosiero {
            vojo: dosiero.vojo,
            haketo,
            grandeco,
            kaŝebla,
        })
    }

    /// Konservas la arbon de unu dosieruja nodo kaj ĝiajn subarbojn.
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
        for (nomo, sub) in &nodo.subarboj {
            enskriboj.push(ArbaEnskribo::SubArbo {
                nomo: nomo.clone(),
                haketo: self.konservi_nodon(sub)?,
            });
        }
        self.konservi_arbon(&Arbo::el_enskriboj(enskriboj)?)
    }

    /// Komparas la laborspacon kun la arbo de `KAPO`.
    ///
    /// Haketo estas rekalkulata nur por dosieroj, kies mtempo aŭ grandeco
    /// diferencas de la indekso; la ceteraj reuzas la memoritan haketon. Se la
    /// kaŝmemoro ŝanĝiĝis, la indekso estas atome reskribita.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, [`Eraro::NevalidaDosiernomo`],
    /// [`Eraro::NevalidaVojaKomponanto`] aŭ [`Eraro::NesubtenitaDosiertipo`]
    /// por la laborspaco, aŭ stokejan eraron por la arbo de `KAPO`.
    pub fn stato(&self) -> Result<Stato, Eraro> {
        let limo = self.limo();
        let malnova = self.legi_indekson()?;
        let haketitaj = self
            .skani("")?
            .into_iter()
            .map(|dosiero| self.haketi_laboran(dosiero, &malnova, limo))
            .collect::<Result<Vec<_>, Eraro>>()?;
        self.ĝisdatigi_indekson(&malnova, &haketitaj)?;

        let kapo = match self.legi_kapon()? {
            Some(komito) => {
                let arbo = self.legi_komiton(&komito)?.arbo;
                self.platigi_arbon(&arbo, "")?
                    .into_iter()
                    .map(|(vojo, haketo, _)| (vojo, haketo))
                    .collect()
            }
            None => BTreeMap::new(),
        };
        Ok(kompari(&kapo, &haketitaj))
    }

    /// Reskribas la indekson, se la kaŝeblaj enskriboj ŝanĝiĝis.
    fn ĝisdatigi_indekson(
        &self,
        malnova: &Indekso,
        haketitaj: &[HaketitaDosiero],
    ) -> Result<(), Eraro> {
        // La vojoj estas unikaj, do egala nombro kaj kongruo de ĉiu kaŝebla
        // dosiero signifas egalan indekson; tiel la kutima kazo evitas rekonstruon.
        let kaŝeblaj = haketitaj
            .iter()
            .filter(|dosiero| dosiero.kaŝebla.is_some())
            .count();
        let sama = kaŝeblaj == malnova.enskriboj().len()
            && haketitaj.iter().all(|dosiero| {
                dosiero.kaŝebla.is_none_or(|(mtempo, grandeco)| {
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
                    dosiero.kaŝebla.map(|(mtempo, grandeco)| IndeksEnskribo {
                        vojo: dosiero.vojo.clone(),
                        mtempo,
                        grandeco,
                        haketo: dosiero.haketo,
                    })
                })
                .collect(),
        )?;
        self.skribi_atome(&Self::meta(INDEKSO_DOSIERO), &nova.seriigi()?)
    }

    /// Legas la indekson; manka aŭ difektita indekso estas malplena kaŝmemoro.
    fn legi_indekson(&self) -> Result<Indekso, Eraro> {
        match self.laborspaco.legi(&Self::meta(INDEKSO_DOSIERO)) {
            // La indekso estas nur kaŝmemoro: difektita indekso estas rekonstruata
            // per plena haketado, do ĝia eraro ne devas haltigi la komandon.
            Ok(bajtoj) => Ok(Indekso::malseriigi(&bajtoj).unwrap_or_default()),
            Err(Eraro::NeekzistantaVojo { .. }) => Ok(Indekso::default()),
            Err(eraro) => Err(eraro),
        }
    }

    /// Legas la arbon kaj ĝiajn subarbojn en platan liston de dosieroj.
    fn platigi_arbon(
        &self,
        haketo: &Haketo,
        prefikso: &str,
    ) -> Result<Vec<(String, Haketo, u64)>, Eraro> {
        let arbo = self.legi_arbon(haketo)?;
        let mut dosieroj = Vec::new();
        for enskribo in arbo.enskriboj() {
            let vojo = kunigi(prefikso, enskribo.nomo())?;
            match enskribo {
                ArbaEnskribo::Dosiero {
                    haketo, grandeco, ..
                } => dosieroj.push((vojo, *haketo, *grandeco)),
                ArbaEnskribo::SubArbo { haketo, .. } => {
                    dosieroj.extend(self.platigi_arbon(haketo, &vojo)?);
                }
            }
        }
        Ok(dosieroj)
    }

    /// Prenas la haketon el la indekso, se la metadatumoj kongruas, alie relegas.
    fn haketi_laboran(
        &self,
        dosiero: LaborDosiero,
        indekso: &Indekso,
        limo: Option<u64>,
    ) -> Result<HaketitaDosiero, Eraro> {
        let mtempo = dosiero
            .mtempo
            .filter(|mtempo| limo.is_some_and(|limo| *mtempo < limo));

        if let Some(mtempo) = mtempo
            && let Some(haketo) = indekso.kaŝita_haketo(&dosiero.vojo, mtempo, dosiero.grandeco)
        {
            return Ok(HaketitaDosiero {
                vojo: dosiero.vojo,
                haketo,
                grandeco: dosiero.grandeco,
                kaŝebla: Some((mtempo, dosiero.grandeco)),
            });
        }

        let datumoj = self.laborspaco.legi(&dosiero.vojo)?;
        let haketo = crate::stokado::haketi(&datumoj);
        let grandeco = u64::try_from(datumoj.len()).map_err(|_| Eraro::TrolongaEnigo {
            maksimumo: usize::MAX,
            akirita: datumoj.len(),
        })?;
        // Se la grandeco ŝanĝiĝis inter `stat` kaj legado, la dosiero estas
        // ankoraŭ skribata; ĝi ne estas kaŝmemorata.
        let kaŝebla = mtempo
            .filter(|_| grandeco == dosiero.grandeco)
            .map(|mtempo| (mtempo, grandeco));
        Ok(HaketitaDosiero {
            vojo: dosiero.vojo,
            haketo,
            grandeco,
            kaŝebla,
        })
    }

    /// Listigas la regulajn dosierojn sub la radiko, preterlasante ignoritajn.
    fn skani(&self, prefikso: &str) -> Result<Vec<LaborDosiero>, Eraro> {
        let mut ignoro = Ignoro::defaŭlta();
        self.skani_en(prefikso, &mut ignoro)
    }

    fn skani_en(&self, dosierujo: &str, ignoro: &mut Ignoro) -> Result<Vec<LaborDosiero>, Eraro> {
        let signo = kunigi(dosierujo, ".vsignore")?;
        if let Some(enhavo) = self.laborspaco.legi_tekston(&signo)? {
            ignoro.aldoni(dosierujo, &enhavo);
        }

        let mut rezulto = Vec::new();
        let mut eroj = self.laborspaco.listi(dosierujo)?;
        eroj.sort_unstable_by(|unu, du| unu.nomo.cmp(&du.nomo));
        for ero in eroj {
            if ero.nomo == VS_DOSIERUJO {
                continue;
            }
            validi_komponanton(&ero.nomo)?;
            let vojo = kunigi(dosierujo, &ero.nomo)?;
            match ero.speco {
                Speco::Dosierujo => {
                    if ignoro.ignorata(&vojo, true) {
                        continue;
                    }
                    let mut suba = ignoro.clone();
                    rezulto.extend(self.skani_en(&vojo, &mut suba)?);
                }
                Speco::Dosiero => {
                    if ignoro.ignorata(&vojo, false) {
                        continue;
                    }
                    let metadatumoj = self.laborspaco.metadatumoj(&vojo)?;
                    if metadatumoj.speco != Speco::Dosiero {
                        return Err(Eraro::NesubtenitaDosiertipo {
                            vojo: format!("{}/{vojo}", self.laborspaco.radiko()),
                        });
                    }
                    rezulto.push(LaborDosiero {
                        vojo,
                        grandeco: metadatumoj.grandeco,
                        mtempo: metadatumoj.mtempo,
                    });
                }
                Speco::Alia => {
                    return Err(Eraro::NesubtenitaDosiertipo {
                        vojo: format!("{}/{vojo}", self.laborspaco.radiko()),
                    });
                }
            }
        }
        Ok(rezulto)
    }

    /// Rekursie forprenas malplenajn dosierujojn, preterlasante ignoritajn.
    fn forigi_malplenajn_dosierujojn(&self, dosierujo: &str, ignoro: &Ignoro) -> Result<(), Eraro> {
        let mut loka = ignoro.clone();
        let signo = kunigi(dosierujo, ".vsignore")?;
        if let Some(enhavo) = self.laborspaco.legi_tekston(&signo)? {
            loka.aldoni(dosierujo, &enhavo);
        }
        for ero in self.laborspaco.listi(dosierujo)? {
            if ero.speco != Speco::Dosierujo {
                continue;
            }
            let sub = kunigi(dosierujo, &ero.nomo)?;
            if sub == VS_DOSIERUJO || loka.ignorata(&sub, true) {
                continue;
            }
            self.forigi_malplenajn_dosierujojn(&sub, &loka)?;
            if self.laborspaco.listi(&sub)?.is_empty() {
                self.laborspaco.forigi_dosierujon(&sub)?;
            }
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl DeponejoMotor<Dosierspaco, DosieraKonservilo, StdaHorlogo> {
    /// Iniciatas deponejon en la donita radika dosierujo.
    ///
    /// La operacio estas idempotenta: ekzistanta `.vs`-strukturo ne estas
    /// forviŝita, kaj nur mankantaj partoj estas kreataj.
    ///
    /// # Errors
    ///
    /// Redonas laborspacan eraron, se la strukturo ne kreeblas.
    pub fn iniciati(radiko: impl Into<PathBuf>) -> Result<Self, Eraro> {
        let radiko = radiko.into();
        let spaco = Dosierspaco::nova(radiko.clone());
        spaco.krei_dosierujon(&format!("{VS_DOSIERUJO}/{OBJEKTUJO}"))?;

        let stokejo = DosieraKonservilo::nova(radiko.join(VS_DOSIERUJO).join(OBJEKTUJO));
        let deponejo = Self::nova(spaco, stokejo, StdaHorlogo);
        let kapo = Self::meta(KAPO_DOSIERO);
        if !deponejo.laborspaco.ekzistas(&kapo)? {
            deponejo.skribi_atome(&kapo, b"")?;
        }
        Ok(deponejo)
    }

    /// Malfermas ekzistantan deponejon.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::Nedeponejo`], se la dosierujo ne enhavas `.vs`, aŭ
    /// laborspacan eraron.
    pub fn malfermi(radiko: impl Into<PathBuf>) -> Result<Self, Eraro> {
        let radiko = radiko.into();
        let spaco = Dosierspaco::nova(radiko.clone());
        let meta = String::from(VS_DOSIERUJO);
        let kapo = format!("{meta}/{KAPO_DOSIERO}");
        if !spaco.ekzistas(&meta)? || !spaco.ekzistas(&kapo)? {
            return Err(Eraro::Nedeponejo);
        }
        spaco.krei_dosierujon(&format!("{meta}/{OBJEKTUJO}"))?;

        let stokejo = DosieraKonservilo::nova(radiko.join(VS_DOSIERUJO).join(OBJEKTUJO));
        Ok(Self::nova(spaco, stokejo, StdaHorlogo))
    }
}

/// Validas referencan nomon (branĉon aŭ etikedon): ĝi estas unu sekura voja
/// komponanto, kiu ne komenciĝas per punkto (tiel ĝi ne kolizias kun
/// provizoraj dosieroj).
fn validi_referencan_nomon(nomo: &str) -> Result<(), Eraro> {
    validi_komponanton(nomo)?;
    if nomo.starts_with('.') || nomo.chars().any(char::is_whitespace) {
        return Err(Eraro::NevalidaVojaKomponanto);
    }
    Ok(())
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

fn parento(vojo: &str) -> Option<&str> {
    vojo.rsplit_once('/').map(|(parento, _)| parento)
}

/// Ĉu du arbaroj havas la saman dosieran enhavon ĉe la sama vojo.
fn sama_enhavo(unua: Option<&(Haketo, u64)>, dua: Option<&(Haketo, u64)>) -> bool {
    match (unua, dua) {
        (None, None) => true,
        (Some(unua), Some(dua)) => unua.0 == dua.0,
        _ => false,
    }
}

/// Konstruas la dosierujan hierarkion el plata listo de relativaj vojoj.
fn nodo_el_dosieroj<'a>(dosieroj: impl IntoIterator<Item = (&'a str, Haketo, u64)>) -> Nodo {
    let mut radiko = Nodo::default();
    for (vojo, haketo, grandeco) in dosieroj {
        let mut nodo = &mut radiko;
        let mut partoj = vojo.split('/').peekable();
        while let Some(parto) = partoj.next() {
            if partoj.peek().is_some() {
                nodo = nodo.subarboj.entry(parto.to_owned()).or_default();
            } else {
                nodo.dosieroj.push((parto.to_owned(), haketo, grandeco));
            }
        }
    }
    radiko
}

/// Klasas la laborspacajn dosierojn rilate al la plata `KAPO`-arbo.
fn kompari(kapo: &BTreeMap<String, Haketo>, laboraj: &[HaketitaDosiero]) -> Stato {
    let mut modifitaj: Vec<String> = laboraj
        .iter()
        .filter(|dosiero| {
            kapo.get(&dosiero.vojo)
                .is_some_and(|haketo| *haketo != dosiero.haketo)
        })
        .map(|dosiero| dosiero.vojo.clone())
        .collect();
    let mut novaj: Vec<String> = laboraj
        .iter()
        .filter(|dosiero| !kapo.contains_key(&dosiero.vojo))
        .map(|dosiero| dosiero.vojo.clone())
        .collect();

    // La vojoj estas unikaj ambaŭflanke: se ĉiu dosiero de `KAPO` havas
    // laborspacan paron, nenio estis forigita kaj la aro ne necesas.
    let kongruaj = laboraj.len().saturating_sub(novaj.len());
    let mut forigitaj: Vec<String> = if kongruaj == kapo.len() {
        Vec::new()
    } else {
        let ekzistantaj: BTreeSet<&str> = laboraj
            .iter()
            .map(|dosiero| dosiero.vojo.as_str())
            .collect();
        kapo.iter()
            .filter(|(vojo, _)| !ekzistantaj.contains(vojo.as_str()))
            .map(|(vojo, _)| vojo.clone())
            .collect()
    };

    modifitaj.sort_unstable();
    novaj.sort_unstable();
    forigitaj.sort_unstable();
    Stato {
        modifitaj,
        novaj,
        forigitaj,
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

    use super::{
        Ago, Deponejo, DeponejoMotor, Eraro, EtikedaCelo, INDEKSO_DOSIERO, KAPO_DOSIERO, Kunfando,
        OBJEKTUJO, Referenco, Rubokolekto, Stato, VS_DOSIERUJO,
    };
    use crate::FiksaHorlogo;
    use crate::laborspaco::{Laborspaco, Memspaco};
    use crate::objektoj::{ArbaEnskribo, Komito};
    use crate::stokado::{Haketo, Konservilo, MemoraKonservilo};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, SystemTime};

    type MemDeponejo = DeponejoMotor<Memspaco, MemoraKonservilo, FiksaHorlogo>;

    static PROVOJ: AtomicU64 = AtomicU64::new(0);

    fn memdeponejo() -> MemDeponejo {
        let spaco = Memspaco::nova();
        spaco
            .krei_dosierujon(&format!("{VS_DOSIERUJO}/{OBJEKTUJO}"))
            .unwrap();
        spaco
            .skribi_atome(&format!("{VS_DOSIERUJO}/{KAPO_DOSIERO}"), b"")
            .unwrap();
        DeponejoMotor::nova(
            spaco,
            MemoraKonservilo::nova(),
            FiksaHorlogo::nova(10_000_000_000),
        )
    }

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
        assert!(prov.radiko.join(VS_DOSIERUJO).join(KAPO_DOSIERO).is_file());
        assert_eq!(deponejo.legi_kapon().unwrap(), None);

        // Ripeta iniciato konservas la ekzistantan strukturon.
        let denove = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        assert_eq!(denove.legi_kapon().unwrap(), None);
    }

    #[test]
    fn testo_kapti_arbon_kaj_legi_dosierajn_objektojn() {
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
            haketo: dosiera_haketo,
            grandeco,
            ..
        } = &subarbo.enskriboj()[0]
        else {
            panic!("atendita dosiero");
        };
        assert_eq!(*grandeco, 6);
        assert_eq!(
            deponejo.stokejo().ricevi(dosiera_haketo).unwrap(),
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
            gepatro: deponejo.legi_kapon().unwrap(),
            mesaĝo: "Unua".into(),
            kreita_je: 1,
            dua_gepatro: None,
        };
        let h1 = deponejo.konservi_komiton(&komito1).unwrap();
        deponejo.skribi_kapon(&h1).unwrap();
        assert_eq!(deponejo.legi_kapon().unwrap(), Some(h1));

        fs::write(prov.radiko.join("testo.txt"), b"dua").unwrap();
        let arbo2 = deponejo.kapti_arbon().unwrap();
        let komito2 = Komito {
            arbo: arbo2,
            gepatro: deponejo.legi_kapon().unwrap(),
            mesaĝo: "Dua".into(),
            kreita_je: 2,
            dua_gepatro: None,
        };
        let h2 = deponejo.konservi_komiton(&komito2).unwrap();
        deponejo.skribi_kapon(&h2).unwrap();

        let legita = deponejo.legi_komiton(&h2).unwrap();
        assert_eq!(legita.gepatro, Some(h1));
        assert_eq!(legita.mesaĝo, "Dua");

        let denove = Deponejo::malfermi(prov.radiko.clone()).unwrap();
        assert_eq!(denove.legi_kapon().unwrap(), Some(h2));
    }

    #[test]
    fn testo_kunfandi_antaŭenigas() {
        let prov = ProvDeponejo::nova("kunfandi-antaŭen");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"unua").unwrap();
        komiti(&deponejo);
        let unua = deponejo.legi_kapon().unwrap().unwrap();

        fs::write(prov.radiko.join("a.txt"), b"dua").unwrap();
        komiti(&deponejo);
        let dua = deponejo.legi_kapon().unwrap().unwrap();

        deponejo.eltiri(&unua).unwrap();
        deponejo.skribi_kapon(&unua).unwrap();

        assert_eq!(
            deponejo.kunfandi(&unua, &dua, "Kunfandis", 3).unwrap(),
            Kunfando::Antauen(dua)
        );
        assert_eq!(fs::read(prov.radiko.join("a.txt")).unwrap(), b"dua");
        deponejo.skribi_kapon(&dua).unwrap();
        assert!(deponejo.stato().unwrap().estas_pura());
        assert_eq!(
            deponejo.kunfandi(&dua, &dua, "Kunfandis", 4).unwrap(),
            Kunfando::Nenio
        );
    }

    #[test]
    fn testo_kunfandi_tri_voja() {
        let prov = ProvDeponejo::nova("kunfandi-tri");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("komuna.txt"), b"bazo").unwrap();
        fs::write(prov.radiko.join("sanĝota.txt"), b"bazo").unwrap();
        komiti(&deponejo);
        let bazo = deponejo.legi_kapon().unwrap().unwrap();

        fs::write(prov.radiko.join("nur-majstro.txt"), b"m").unwrap();
        komiti(&deponejo);
        let majstro = deponejo.legi_kapon().unwrap().unwrap();

        deponejo.eltiri(&bazo).unwrap();
        deponejo.skribi_kapon(&bazo).unwrap();
        fs::write(prov.radiko.join("sanĝota.txt"), b"fonto").unwrap();
        fs::write(prov.radiko.join("nur-fonto.txt"), b"f").unwrap();
        komiti(&deponejo);
        let fonto = deponejo.legi_kapon().unwrap().unwrap();

        deponejo.eltiri(&majstro).unwrap();
        deponejo.skribi_kapon(&majstro).unwrap();
        let rezulto = deponejo.kunfandi(&majstro, &fonto, "Kunfandis", 3).unwrap();
        let kunfando = match rezulto {
            Kunfando::Kunfandita(haketo) => haketo,
            alia => panic!("atendita kunfanda komito, sed ricevis: {alia:?}"),
        };

        let komito = deponejo.legi_komiton(&kunfando).unwrap();
        assert_eq!(komito.gepatro, Some(majstro));
        assert_eq!(komito.dua_gepatro, Some(fonto));
        assert_eq!(komito.mesaĝo, "Kunfandis");
        assert_eq!(fs::read(prov.radiko.join("sanĝota.txt")).unwrap(), b"fonto");
        assert_eq!(fs::read(prov.radiko.join("komuna.txt")).unwrap(), b"bazo");
        assert!(prov.radiko.join("nur-majstro.txt").is_file());
        assert!(prov.radiko.join("nur-fonto.txt").is_file());

        deponejo.skribi_kapon(&kunfando).unwrap();
        assert!(deponejo.stato().unwrap().estas_pura());
        assert_eq!(
            deponejo.kunfandi(&kunfando, &fonto, "Denove", 4).unwrap(),
            Kunfando::Nenio
        );
    }

    #[test]
    fn testo_kunfandi_raportas_konflikton() {
        let prov = ProvDeponejo::nova("kunfandi-konflikto");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"bazo").unwrap();
        komiti(&deponejo);
        let bazo = deponejo.legi_kapon().unwrap().unwrap();

        fs::write(prov.radiko.join("a.txt"), b"majstro").unwrap();
        komiti(&deponejo);
        let majstro = deponejo.legi_kapon().unwrap().unwrap();

        deponejo.eltiri(&bazo).unwrap();
        deponejo.skribi_kapon(&bazo).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"fonto").unwrap();
        komiti(&deponejo);
        let fonto = deponejo.legi_kapon().unwrap().unwrap();

        deponejo.eltiri(&majstro).unwrap();
        deponejo.skribi_kapon(&majstro).unwrap();
        let rezulto = deponejo.kunfandi(&majstro, &fonto, "Kunfandis", 3);
        assert!(matches!(
            &rezulto,
            Err(Eraro::KunfandaKonflikto { vojoj }) if vojoj == &["a.txt"]
        ));
        assert_eq!(
            fs::read(prov.radiko.join("a.txt")).unwrap(),
            b"majstro",
            "fiaskinta kunfando ne rajtas tuŝi la laborspacon"
        );
    }

    #[test]
    fn testo_kunfandi_raportas_dosierujan_kolizion() {
        let prov = ProvDeponejo::nova("kunfandi-kolizio");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("komuna.txt"), b"bazo").unwrap();
        komiti(&deponejo);
        let bazo = deponejo.legi_kapon().unwrap().unwrap();

        fs::write(prov.radiko.join("a"), b"dosiero").unwrap();
        komiti(&deponejo);
        let majstro = deponejo.legi_kapon().unwrap().unwrap();

        deponejo.eltiri(&bazo).unwrap();
        deponejo.skribi_kapon(&bazo).unwrap();
        fs::create_dir_all(prov.radiko.join("a").join("b")).unwrap();
        fs::write(prov.radiko.join("a").join("b").join("c.txt"), b"c").unwrap();
        komiti(&deponejo);
        let fonto = deponejo.legi_kapon().unwrap().unwrap();

        deponejo.eltiri(&majstro).unwrap();
        deponejo.skribi_kapon(&majstro).unwrap();
        let rezulto = deponejo.kunfandi(&majstro, &fonto, "Kunfandis", 3);
        assert!(matches!(
            &rezulto,
            Err(Eraro::KunfandaKonflikto { vojoj }) if vojoj == &["a", "a/b/c.txt"]
        ));
    }

    #[test]
    fn testo_kunfandi_rifuzas_nesamajn_historiojn() {
        let prov = ProvDeponejo::nova("kunfandi-samaj");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"a").unwrap();
        let arbo = deponejo.kapti_arbon().unwrap();
        let unua = deponejo
            .konservi_komiton(&Komito {
                arbo,
                gepatro: None,
                mesaĝo: "Unua".into(),
                kreita_je: 1,
                dua_gepatro: None,
            })
            .unwrap();
        let dua = deponejo
            .konservi_komiton(&Komito {
                arbo,
                gepatro: None,
                mesaĝo: "Dua".into(),
                kreita_je: 2,
                dua_gepatro: None,
            })
            .unwrap();
        deponejo.skribi_kapon(&unua).unwrap();

        assert!(matches!(
            deponejo.kunfandi(&unua, &dua, "Kunfandis", 3),
            Err(Eraro::NesamajHistorioj)
        ));
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
            gepatro: deponejo.legi_kapon().unwrap(),
            mesaĝo: "testo".into(),
            kreita_je: 1,
            dua_gepatro: None,
        };
        let haketo = deponejo.konservi_komiton(&komito).unwrap();
        deponejo.skribi_kapon(&haketo).unwrap();
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
    fn testo_stato_detektas_ŝanĝojn() {
        let prov = ProvDeponejo::nova("stato-ŝanĝoj");
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
    fn testo_stato_uzas_kaŝmemoron() {
        let prov = ProvDeponejo::nova("stato-kaŝmemoro");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a.txt");
        fs::write(&dosiero, b"unua").unwrap();
        let tempo = malnovigi(&dosiero);
        komiti(&deponejo);

        assert!(deponejo.stato().unwrap().estas_pura());
        let indekso = deponejo.legi_indekson().unwrap();
        assert_eq!(indekso.enskriboj().len(), 1);
        assert_eq!(indekso.enskriboj()[0].vojo, "a.txt");

        // Sama grandeco kaj restarigita mtempo: la kaŝmemoro respondas sen relegado.
        fs::write(&dosiero, b"dua!").unwrap();
        fs::File::options()
            .write(true)
            .open(&dosiero)
            .unwrap()
            .set_modified(tempo)
            .unwrap();
        assert!(deponejo.stato().unwrap().estas_pura());

        // Nova mtempo nuligas la kaŝmemoron.
        malnovigi(&dosiero);
        assert_eq!(deponejo.stato().unwrap().modifitaj, ["a.txt"]);
    }

    #[test]
    fn testo_stato_ne_kaŝmemoras_freŝajn_dosierojn() {
        let prov = ProvDeponejo::nova("stato-freŝa");
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
        let prov = ProvDeponejo::nova("komito-kaŝmemoro");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a.txt");
        fs::write(&dosiero, b"unua").unwrap();
        let tempo = malnovigi(&dosiero);
        komiti(&deponejo);

        let unua = deponejo.legi_kapon().unwrap().unwrap();
        let arbo1 = deponejo.legi_komiton(&unua).unwrap().arbo;

        // Sama grandeco kaj restarigita mtempo: la komito reuzas la indeksan
        // haketon anstataŭ relegi la dosieron, do la arbo restas la sama.
        fs::write(&dosiero, b"alia").unwrap();
        fs::File::options()
            .write(true)
            .open(&dosiero)
            .unwrap()
            .set_modified(tempo)
            .unwrap();
        komiti(&deponejo);

        let dua = deponejo.legi_kapon().unwrap().unwrap();
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

        let kapo = deponejo.legi_kapon().unwrap().unwrap();
        let arbo = deponejo
            .legi_arbon(&deponejo.legi_komiton(&kapo).unwrap().arbo)
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

        let kapo = deponejo.legi_kapon().unwrap().unwrap();
        let arbo = deponejo.legi_komiton(&kapo).unwrap().arbo;
        let mut vojoj: Vec<String> = deponejo
            .platigi_arbon(&arbo, "")
            .unwrap()
            .into_iter()
            .map(|(vojo, _, _)| vojo)
            .collect();
        vojoj.sort_unstable();
        assert_eq!(vojoj, ["a/1.txt", "a/b/2.txt", "a/b/c/3.txt"]);
        assert!(deponejo.stato().unwrap().estas_pura());
    }

    #[test]
    fn testo_eltiri_restarigas_arbon_kaj_forigas_la_indekson() {
        let prov = ProvDeponejo::nova("eltiri");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a.txt");
        fs::write(&dosiero, b"unua").unwrap();
        malnovigi(&dosiero);
        fs::create_dir(prov.radiko.join("sub")).unwrap();
        let suba = prov.radiko.join("sub").join("b.txt");
        fs::write(&suba, b"b").unwrap();
        malnovigi(&suba);
        komiti(&deponejo);
        let unua = deponejo.legi_kapon().unwrap().unwrap();

        fs::write(&dosiero, b"dua").unwrap();
        fs::create_dir(prov.radiko.join("aldona")).unwrap();
        fs::write(prov.radiko.join("aldona").join("nova.txt"), b"nova").unwrap();
        komiti(&deponejo);
        assert!(
            prov.radiko
                .join(VS_DOSIERUJO)
                .join(INDEKSO_DOSIERO)
                .is_file(),
            "la indekso devas ekzisti antaŭ la eltiro"
        );

        deponejo.eltiri(&unua).unwrap();

        assert_eq!(fs::read(&dosiero).unwrap(), b"unua");
        assert_eq!(fs::read(&suba).unwrap(), b"b");
        assert!(
            !prov.radiko.join("aldona").exists(),
            "la malplena dosierujo devas esti forprenita"
        );
        assert!(
            !prov
                .radiko
                .join(VS_DOSIERUJO)
                .join(INDEKSO_DOSIERO)
                .exists(),
            "la malnova indekso devas esti forviŝita"
        );
    }

    #[test]
    fn testo_eltiri_traktas_dosiernoman_kolizion() {
        let prov = ProvDeponejo::nova("eltiri-kolizio");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let dosiero = prov.radiko.join("a");
        fs::write(&dosiero, b"dosiero").unwrap();
        malnovigi(&dosiero);
        komiti(&deponejo);
        let unua = deponejo.legi_kapon().unwrap().unwrap();

        fs::remove_file(&dosiero).unwrap();
        fs::create_dir(&dosiero).unwrap();
        fs::write(dosiero.join("b.txt"), b"nesto").unwrap();
        komiti(&deponejo);
        let dua = deponejo.legi_kapon().unwrap().unwrap();

        deponejo.eltiri(&unua).unwrap();
        assert!(dosiero.is_file());
        assert_eq!(fs::read(&dosiero).unwrap(), b"dosiero");

        deponejo.eltiri(&dua).unwrap();
        assert!(dosiero.is_dir());
        assert_eq!(fs::read(dosiero.join("b.txt")).unwrap(), b"nesto");
    }

    #[test]
    fn testo_branĉoj_kaj_nuna_branĉo() {
        let prov = ProvDeponejo::nova("branĉoj");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        assert_eq!(deponejo.branĉoj().unwrap(), Vec::new());
        assert_eq!(deponejo.nuna_branĉo().unwrap(), None);

        let cefa = Haketo::el_bajtoj([9; 32]);
        deponejo.skribi_branĉon("ĉefa", &cefa).unwrap();
        deponejo
            .skribi_branĉon("eksperimento", &Haketo::el_bajtoj([8; 32]))
            .unwrap();
        assert_eq!(deponejo.legi_branĉon("ĉefa").unwrap(), Some(cefa));
        let branĉoj = deponejo.branĉoj().unwrap();
        assert_eq!(
            branĉoj,
            [
                ("eksperimento".to_owned(), Haketo::el_bajtoj([8; 32])),
                ("ĉefa".to_owned(), cefa)
            ]
        );

        deponejo.agordi_nunan_branĉon("ĉefa").unwrap();
        assert_eq!(deponejo.nuna_branĉo().unwrap().as_deref(), Some("ĉefa"));
        deponejo.forigi_nunan_branĉon().unwrap();
        assert_eq!(deponejo.nuna_branĉo().unwrap(), None);

        for malvalida in ["", "../fuĝo", ".kaŝita", "a/b"] {
            assert!(
                matches!(
                    deponejo.skribi_branĉon(malvalida, &cefa),
                    Err(Eraro::NevalidaVojaKomponanto)
                ),
                "la nomo «{malvalida}» devus esti rifuzita"
            );
        }
    }

    #[test]
    fn testo_etikedoj_kaj_aliasoj() {
        let prov = ProvDeponejo::nova("etikedoj");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"unua").unwrap();
        komiti(&deponejo);
        let unua = deponejo.legi_kapon().unwrap().unwrap();

        assert_eq!(deponejo.solvi_etikedon("v1.0").unwrap(), None);
        deponejo
            .skribi_etikedon("v1.0", &EtikedaCelo::Komito(unua))
            .unwrap();
        deponejo
            .skribi_etikedon("lasta", &EtikedaCelo::Alias("v1.0".into()))
            .unwrap();
        deponejo
            .skribi_etikedon("ĉena", &EtikedaCelo::Alias("lasta".into()))
            .unwrap();

        assert_eq!(deponejo.solvi_etikedon("v1.0").unwrap(), Some(unua));
        assert_eq!(deponejo.solvi_etikedon("lasta").unwrap(), Some(unua));
        assert_eq!(deponejo.solvi_etikedon("ĉena").unwrap(), Some(unua));

        let etikedoj = deponejo.etikedoj().unwrap();
        assert_eq!(etikedoj.len(), 3);
        assert_eq!(etikedoj[0].nomo, "lasta");
        assert_eq!(etikedoj[2].nomo, "ĉena");

        deponejo
            .skribi_etikedon("ciklo-a", &EtikedaCelo::Alias("ciklo-b".into()))
            .unwrap();
        deponejo
            .skribi_etikedon("ciklo-b", &EtikedaCelo::Alias("ciklo-a".into()))
            .unwrap();
        assert!(matches!(
            deponejo.solvi_etikedon("ciklo-a"),
            Err(Eraro::CiklaEtikedo { .. })
        ));

        deponejo
            .skribi_etikedon("mankanta", &EtikedaCelo::Alias("nenio".into()))
            .unwrap();
        assert!(matches!(
            deponejo.solvi_etikedon("mankanta"),
            Err(Eraro::NeekzistantaEtikedo { nomo }) if nomo == "nenio"
        ));

        assert!(matches!(
            deponejo.skribi_etikedon("a/b", &EtikedaCelo::Komito(unua)),
            Err(Eraro::NevalidaVojaKomponanto)
        ));
        assert!(matches!(
            deponejo.skribi_etikedon("fuĝo", &EtikedaCelo::Alias("a/b".into())),
            Err(Eraro::NevalidaVojaKomponanto)
        ));
    }

    #[test]
    fn testo_renomi_kaj_forigi_branĉon() {
        let prov = ProvDeponejo::nova("renomi");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        let celo = Haketo::el_bajtoj([5; 32]);
        deponejo.skribi_branĉon("malnova", &celo).unwrap();
        deponejo
            .skribi_branĉon("alia", &Haketo::el_bajtoj([6; 32]))
            .unwrap();
        deponejo.agordi_nunan_branĉon("malnova").unwrap();

        deponejo.renomi_branĉon("malnova", "nova").unwrap();
        assert_eq!(deponejo.legi_branĉon("malnova").unwrap(), None);
        assert_eq!(deponejo.legi_branĉon("nova").unwrap(), Some(celo));
        assert_eq!(deponejo.nuna_branĉo().unwrap().as_deref(), Some("nova"));
        deponejo.renomi_branĉon("nova", "nova").unwrap();

        assert!(matches!(
            deponejo.renomi_branĉon("nova", "alia"),
            Err(Eraro::BranĉoEkzistas { nomo }) if nomo == "alia"
        ));
        assert!(matches!(
            deponejo.renomi_branĉon("mankanta", "x"),
            Err(Eraro::NeekzistantaBranĉo { nomo }) if nomo == "mankanta"
        ));

        let movoj = deponejo.movhistorio().unwrap().movoj().to_vec();
        assert_eq!(movoj.len(), 1);
        assert_eq!(movoj[0].ago, Ago::Renomi);
        assert_eq!(movoj[0].celo.nomo(), "nova");
        assert_eq!(movoj[0].malnova, Some(celo));
        assert_eq!(movoj[0].nova, Some(celo));

        deponejo.forigi_branĉon("nova").unwrap();
        assert_eq!(deponejo.legi_branĉon("nova").unwrap(), None);
        assert_eq!(deponejo.nuna_branĉo().unwrap(), None);
        let movoj = deponejo.movhistorio().unwrap().movoj().to_vec();
        assert_eq!(movoj.len(), 2);
        assert_eq!(movoj[1].ago, Ago::Forigi);
        assert_eq!(movoj[1].malnova, Some(celo));
        assert_eq!(movoj[1].nova, None);
        assert!(matches!(
            deponejo.forigi_branĉon("nova"),
            Err(Eraro::NeekzistantaBranĉo { .. })
        ));
    }

    #[test]
    fn testo_movhistorio_konservas_antauajn_valorojn() {
        let prov = ProvDeponejo::nova("movhistorio");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"unua").unwrap();
        let unua = deponejo
            .konservi_komiton(&Komito {
                arbo: deponejo.kapti_arbon().unwrap(),
                gepatro: None,
                mesaĝo: "Unua".into(),
                kreita_je: 1,
                dua_gepatro: None,
            })
            .unwrap();
        deponejo.movi_kapon(&unua, Ago::Komiti).unwrap();
        deponejo.movi_branĉon("ĉefa", &unua, Ago::Krei).unwrap();

        fs::write(prov.radiko.join("a.txt"), b"dua").unwrap();
        let dua = deponejo
            .konservi_komiton(&Komito {
                arbo: deponejo.kapti_arbon().unwrap(),
                gepatro: Some(unua),
                mesaĝo: "Dua".into(),
                kreita_je: 2,
                dua_gepatro: None,
            })
            .unwrap();
        deponejo.movi_kapon(&dua, Ago::Komiti).unwrap();
        deponejo.movi_branĉon("ĉefa", &dua, Ago::Komiti).unwrap();

        let historio = deponejo.movhistorio().unwrap();
        let movoj = historio.movoj();
        assert_eq!(movoj.len(), 4);
        assert_eq!(movoj[0].celo, Referenco::Kapo);
        assert_eq!(movoj[0].malnova, None);
        assert_eq!(movoj[0].nova, Some(unua));
        assert_eq!(movoj[1].celo, Referenco::Branĉo("ĉefa".into()));
        assert_eq!(movoj[1].ago, Ago::Krei);
        assert_eq!(movoj[1].malnova, None);
        assert_eq!(movoj[3].malnova, Some(unua));
        assert_eq!(movoj[3].nova, Some(dua));

        deponejo.movi_kapon(&dua, Ago::Eltiri).unwrap();
        assert_eq!(deponejo.movhistorio().unwrap().movoj().len(), 4);
    }

    #[test]
    fn testo_rubokolekti_forigas_nur_neatingeblajn_objektojn() {
        let prov = ProvDeponejo::nova("rubokolekto");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"unua").unwrap();
        komiti(&deponejo);
        let unua = deponejo.legi_kapon().unwrap().unwrap();
        let arbo_unua = deponejo.legi_komiton(&unua).unwrap().arbo;
        let blobo_unua = match &deponejo.legi_arbon(&arbo_unua).unwrap().enskriboj()[0] {
            ArbaEnskribo::Dosiero { haketo, .. } => *haketo,
            ArbaEnskribo::SubArbo { .. } => panic!("atendita dosiero"),
        };

        fs::write(prov.radiko.join("a.txt"), b"dua").unwrap();
        komiti(&deponejo);
        let dua = deponejo.legi_kapon().unwrap().unwrap();

        deponejo
            .skribi_etikedon("unua", &EtikedaCelo::Komito(unua))
            .unwrap();
        deponejo.eltiri(&unua).unwrap();
        deponejo.skribi_kapon(&unua).unwrap();
        let rubo = deponejo.stokejo().konservi(b"rubo").unwrap();

        let raporto = deponejo.rubokolekti().unwrap();
        assert_eq!(raporto.forigitaj, 4);
        assert_eq!(raporto.konservitaj, 3);

        assert!(deponejo.stokejo().havas(&unua));
        assert!(deponejo.stokejo().havas(&arbo_unua));
        assert!(deponejo.stokejo().havas(&blobo_unua));
        assert!(!deponejo.stokejo().havas(&rubo));
        assert!(!deponejo.stokejo().havas(&dua));
        assert_eq!(deponejo.stokejo().ricevi(&blobo_unua).unwrap(), b"unua");
        assert!(deponejo.stato().unwrap().estas_pura());

        assert_eq!(
            deponejo.rubokolekti().unwrap(),
            Rubokolekto {
                forigitaj: 0,
                konservitaj: 3
            }
        );
    }

    #[test]
    fn testo_rubokolekti_konservas_movhistoriajn_valorojn() {
        let prov = ProvDeponejo::nova("rubokolekto-reflog");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"unua").unwrap();
        let unua = deponejo
            .konservi_komiton(&Komito {
                arbo: deponejo.kapti_arbon().unwrap(),
                gepatro: None,
                mesaĝo: "Unua".into(),
                kreita_je: 1,
                dua_gepatro: None,
            })
            .unwrap();
        deponejo.movi_kapon(&unua, Ago::Komiti).unwrap();

        fs::write(prov.radiko.join("a.txt"), b"dua").unwrap();
        let dua = deponejo
            .konservi_komiton(&Komito {
                arbo: deponejo.kapti_arbon().unwrap(),
                gepatro: Some(unua),
                mesaĝo: "Dua".into(),
                kreita_je: 2,
                dua_gepatro: None,
            })
            .unwrap();
        deponejo.movi_kapon(&dua, Ago::Komiti).unwrap();

        deponejo.skribi_kapon(&unua).unwrap();
        let raporto = deponejo.rubokolekti().unwrap();
        assert_eq!(raporto.forigitaj, 0);
        assert!(deponejo.stokejo().havas(&dua));
        assert_eq!(deponejo.legi_komiton(&dua).unwrap().gepatro, Some(unua));
    }

    fn arbo_kiel_vojoj(deponejo: &Deponejo, haketo: &Haketo) -> Vec<String> {
        let mut vojoj: Vec<String> = deponejo
            .platigi_arbon(haketo, "")
            .unwrap()
            .into_iter()
            .map(|(vojo, _, _)| vojo)
            .collect();
        vojoj.sort_unstable();
        vojoj
    }

    #[test]
    fn testo_defaule_ignoras_git_target_kaj_vs() {
        let prov = ProvDeponejo::nova("ignoro-defaule");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::create_dir_all(prov.radiko.join(".git").join("objektoj")).unwrap();
        fs::write(prov.radiko.join(".git").join("konfigo"), b"git").unwrap();
        fs::create_dir_all(prov.radiko.join("target").join("debug")).unwrap();
        fs::write(prov.radiko.join("target").join("debug").join("bin"), b"bin").unwrap();
        fs::create_dir(prov.radiko.join("src")).unwrap();
        fs::write(prov.radiko.join("src").join("lib.rs"), b"kodo").unwrap();

        let arbo = deponejo.kapti_arbon().unwrap();
        assert_eq!(arbo_kiel_vojoj(&deponejo, &arbo), ["src/lib.rs"]);
        assert_eq!(deponejo.stato().unwrap().novaj, ["src/lib.rs"]);
    }

    #[test]
    fn testo_vsignore_kaj_nestitaj_reguloj() {
        let prov = ProvDeponejo::nova("ignoro-vsignore");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join(".vsignore"), b"*.log\n!grava.log\nsub/\n").unwrap();
        fs::write(prov.radiko.join("a.log"), b"a").unwrap();
        fs::write(prov.radiko.join("grava.log"), b"g").unwrap();
        fs::write(prov.radiko.join("konservu.txt"), b"k").unwrap();
        fs::create_dir(prov.radiko.join("sub")).unwrap();
        fs::write(prov.radiko.join("sub").join("x.txt"), b"x").unwrap();
        fs::create_dir(prov.radiko.join("dok")).unwrap();
        fs::write(prov.radiko.join("dok").join(".vsignore"), b"*.tmp\n").unwrap();
        fs::write(prov.radiko.join("dok").join("a.tmp"), b"t").unwrap();
        fs::write(prov.radiko.join("dok").join("b.txt"), b"b").unwrap();

        let arbo = deponejo.kapti_arbon().unwrap();
        assert_eq!(
            arbo_kiel_vojoj(&deponejo, &arbo),
            [
                ".vsignore",
                "dok/.vsignore",
                "dok/b.txt",
                "grava.log",
                "konservu.txt"
            ]
        );
    }

    #[test]
    fn testo_eltiri_konservas_ignoritajn_dosierojn() {
        let prov = ProvDeponejo::nova("ignoro-eltiri");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"unua").unwrap();
        komiti(&deponejo);
        let unua = deponejo.legi_kapon().unwrap().unwrap();

        fs::create_dir(prov.radiko.join("target")).unwrap();
        fs::write(
            prov.radiko.join("target").join("kaŝu"),
            "ne tuŝu".as_bytes(),
        )
        .unwrap();
        fs::write(prov.radiko.join("a.txt"), b"dua").unwrap();
        komiti(&deponejo);

        deponejo.eltiri(&unua).unwrap();
        assert_eq!(fs::read(prov.radiko.join("a.txt")).unwrap(), b"unua");
        assert_eq!(
            fs::read(prov.radiko.join("target").join("kaŝu")).unwrap(),
            "ne tuŝu".as_bytes()
        );
    }

    #[test]
    fn testo_memora_motoro_funkcias_sen_std() {
        let deponejo = memdeponejo();
        deponejo
            .laborspaco()
            .skribi_atome("saluton.txt", b"saluton")
            .unwrap();

        let arbo = deponejo.kapti_arbon().unwrap();
        let komito = Komito {
            arbo,
            gepatro: None,
            mesaĝo: "Enmemora".into(),
            kreita_je: 1,
            dua_gepatro: None,
        };
        let haketo = deponejo.konservi_komiton(&komito).unwrap();
        deponejo.skribi_kapon(&haketo).unwrap();

        assert!(deponejo.stato().unwrap().estas_pura());
        assert_eq!(deponejo.legi_kapon().unwrap(), Some(haketo));
        assert_eq!(
            deponejo.laborspaco().legi("saluton.txt").unwrap(),
            b"saluton"
        );
    }
}
