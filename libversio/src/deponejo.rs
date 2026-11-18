//! Deponeja metadatumaro kaj DAG-administrado (nur kun `std`).

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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

const REFS_DOSIERUJO: &str = "refs";
const BRANCHOJ_DOSIERUJO: &str = "heads";
const NUNA_BRANCHO_DOSIERO: &str = "brancxo";

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

    fn brancxo_vojo(&self, nomo: &str) -> PathBuf {
        self.metadatumoj()
            .join(REFS_DOSIERUJO)
            .join(BRANCHOJ_DOSIERUJO)
            .join(nomo)
    }

    /// Legas la celan komiton de la donita branĉo; `None` signifas, ke ĝi
    /// ankoraŭ ne ekzistas.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaVojaKomponanto`], se la nomo estas malvalida,
    /// aŭ dosiersisteman eraron.
    pub fn legi_brancxon(&self, nomo: &str) -> Result<Option<Haketo>, Eraro> {
        validi_brancxan_nomon(nomo)?;
        match fs::read_to_string(self.brancxo_vojo(nomo)) {
            Ok(teksto) => Ok(Some(teksto.trim().parse()?)),
            Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(eraro) => Err(eraro.into()),
        }
    }

    /// Skribas la celan komiton de la donita branĉo.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaVojaKomponanto`], se la nomo estas malvalida,
    /// aŭ dosiersisteman eraron.
    pub fn skribi_brancxon(&self, nomo: &str, haketo: &Haketo) -> Result<(), Eraro> {
        validi_brancxan_nomon(nomo)?;
        fs::create_dir_all(
            self.metadatumoj()
                .join(REFS_DOSIERUJO)
                .join(BRANCHOJ_DOSIERUJO),
        )?;
        skribi_atome(&self.brancxo_vojo(nomo), haketo.to_string().as_bytes())
    }

    /// Listigas la branĉojn kaj iliajn celajn komitojn, ordigitajn laŭ nomo.
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman aŭ haketan eraron por difektita referenco.
    pub fn brancxoj(&self) -> Result<Vec<(String, Haketo)>, Eraro> {
        let dosierujo = self
            .metadatumoj()
            .join(REFS_DOSIERUJO)
            .join(BRANCHOJ_DOSIERUJO);
        let eniroj = match fs::read_dir(&dosierujo) {
            Ok(eniroj) => eniroj,
            Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(eraro) => return Err(eraro.into()),
        };

        let mut brancxoj = Vec::new();
        for eniro in eniroj {
            let eniro = eniro?;
            if !eniro.file_type()?.is_file() {
                continue;
            }
            let Some(nomo) = eniro.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            // Provizoraj dosieroj de la atoma skribo komenciĝas per punkto.
            if nomo.starts_with('.') {
                continue;
            }
            let haketo = fs::read_to_string(eniro.path())?.trim().parse()?;
            brancxoj.push((nomo, haketo));
        }
        brancxoj.sort_unstable_by(|unu, du| unu.0.cmp(&du.0));
        Ok(brancxoj)
    }

    /// La nomo de la nuna branĉo; `None` signifas detaĉitan `HEAD`.
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman eraron.
    pub fn nuna_brancxo(&self) -> Result<Option<String>, Eraro> {
        match fs::read_to_string(self.metadatumoj().join(NUNA_BRANCHO_DOSIERO)) {
            Ok(teksto) => {
                let nomo = teksto.trim();
                if nomo.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(nomo.to_owned()))
                }
            }
            Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(eraro) => Err(eraro.into()),
        }
    }

    /// Memorigas, ke `HEAD` sekvas la donitan branĉon.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::NevalidaVojaKomponanto`], se la nomo estas malvalida,
    /// aŭ dosiersisteman eraron.
    pub fn agordi_nunan_brancxon(&self, nomo: &str) -> Result<(), Eraro> {
        validi_brancxan_nomon(nomo)?;
        skribi_atome(
            &self.metadatumoj().join(NUNA_BRANCHO_DOSIERO),
            nomo.as_bytes(),
        )
    }

    /// Forigas la memoron pri la nuna branĉo (detaĉita `HEAD`).
    ///
    /// # Errors
    ///
    /// Redonas dosiersisteman eraron, escepte de mankanta dosiero.
    pub fn forigi_nunan_brancxon(&self) -> Result<(), Eraro> {
        match fs::remove_file(self.metadatumoj().join(NUNA_BRANCHO_DOSIERO)) {
            Ok(()) => Ok(()),
            Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(eraro) => Err(eraro.into()),
        }
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
    /// `HEAD` ne estas ĝisdatigata; tion faras la alvokanto.
    ///
    /// # Errors
    ///
    /// Redonas stokejan, malseriigan aŭ dosiersisteman eraron.
    pub fn eltiri(&self, komito: &Haketo) -> Result<(), Eraro> {
        let arbo = self.legi_komiton(komito)?.arbo;
        let celo: HashMap<String, Haketo> = self
            .platigi_arbon(&arbo, "")?
            .into_iter()
            .map(|(vojo, haketo, _)| (vojo, haketo))
            .collect();

        let limo = SystemTime::now()
            .checked_sub(FRESXA_MARGENO)
            .and_then(nanosekundoj);
        let indekso = self.legi_indekson()?;
        let laboraj = skani(&self.radiko, "")?
            .into_par_iter()
            .map(|dosiero| haketi_laboran(dosiero, &indekso, limo))
            .collect::<Result<Vec<_>, Eraro>>()?;
        let nunaj: HashMap<&str, Haketo> = laboraj
            .iter()
            .map(|dosiero| (dosiero.vojo.as_str(), dosiero.haketo))
            .collect();

        // Unue forigu la superfluajn dosierojn, por ke dosiero kaj dosierujo
        // kun la sama nomo ne koliziu dum la skribado.
        for dosiero in &laboraj {
            if celo.contains_key(&dosiero.vojo) {
                continue;
            }
            match fs::remove_file(self.radiko.join(&dosiero.vojo)) {
                Ok(()) => {}
                Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => {}
                Err(eraro) => return Err(eraro.into()),
            }
        }
        forigi_malplenajn_dosierujojn(&self.radiko)?;

        celo.par_iter()
            .map(|(vojo, haketo)| {
                if nunaj.get(vojo.as_str()) == Some(haketo) {
                    return Ok(());
                }
                let datumoj = self.stokejo.ricevi(haketo)?;
                let absoluta = self.radiko.join(vojo);
                if let Some(parent) = absoluta.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&absoluta, datumoj)?;
                Ok(())
            })
            .collect::<Result<Vec<()>, Eraro>>()?;

        match fs::remove_file(self.metadatumoj().join(INDEKSO_DOSIERO)) {
            Ok(()) => Ok(()),
            Err(eraro) if eraro.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(eraro) => Err(eraro.into()),
        }
    }

    /// Kunfandas la celan komiton en la nunan.
    ///
    /// Se la celo jam estas en la historio de la nuna komito, nenio ŝanĝiĝas.
    /// Se la nuna komito estas antaŭulo de la celo, la operacio antaŭeniras
    /// sen nova komito. Alie ĝi trovas komunan antaŭulon kaj tri-vojon
    /// kunfandas la platajn arbojn. Post sukceso la laborspaco spegulas la
    /// rezulton; la referencoj kaj `HEAD` restas al la alvokanto.
    ///
    /// # Errors
    ///
    /// Redonas [`Eraro::KunfandaKonflikto`], se ambaŭ flankoj malsame ŝanĝis
    /// la saman vojon, [`Eraro::NesamajHistorioj`], se mankas komuna antaŭulo,
    /// aŭ stokejan, malseriigan aŭ dosiersisteman eraron.
    pub fn kunfandi(
        &self,
        nuna: &Haketo,
        celo: &Haketo,
        mesagxo: &str,
        kreita_je: u64,
    ) -> Result<Kunfando, Eraro> {
        if nuna == celo || self.estas_antauxulo(celo, nuna)? {
            return Ok(Kunfando::Nenio);
        }
        if self.estas_antauxulo(nuna, celo)? {
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
            mesagxo: mesagxo.to_owned(),
            kreita_je,
            dua_gepatro: Some(*celo),
        };
        let haketo = self.konservi_komiton(&komito)?;
        self.eltiri(&haketo)?;
        Ok(Kunfando::Kunfandita(haketo))
    }

    /// Ĉu `antauxulo` estas atingebla ekde `posteulo` sekvante gepatrojn.
    fn estas_antauxulo(&self, antauxulo: &Haketo, posteulo: &Haketo) -> Result<bool, Eraro> {
        let mut stako = vec![*posteulo];
        let mut vizititaj = HashSet::new();
        while let Some(haketo) = stako.pop() {
            if &haketo == antauxulo {
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
        let mut antauxuloj = HashSet::new();
        let mut stako = vec![*unua];
        while let Some(haketo) = stako.pop() {
            if !antauxuloj.insert(haketo) {
                continue;
            }
            let komito = self.legi_komiton(&haketo)?;
            stako.extend(komito.gepatro);
            stako.extend(komito.dua_gepatro);
        }

        let mut vizititaj = HashSet::new();
        let mut stako = vec![*dua];
        while let Some(haketo) = stako.pop() {
            if antauxuloj.contains(&haketo) {
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

        let mut antauxa: Option<&String> = None;
        for vojo in rezulto.keys() {
            if let Some(antauxa) = antauxa
                && vojo.starts_with(antauxa.as_str())
                && vojo.as_bytes().get(antauxa.len()) == Some(&b'/')
            {
                konfliktoj.push(antauxa.clone());
                konfliktoj.push(vojo.clone());
            }
            antauxa = Some(vojo);
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
    fn plata_arbaro(&self, haketo: &Haketo) -> Result<HashMap<String, (Haketo, u64)>, Eraro> {
        Ok(self
            .platigi_arbon(haketo, "")?
            .into_iter()
            .map(|(vojo, haketo, grandeco)| (vojo, (haketo, grandeco)))
            .collect())
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
                        Ok(self
                            .platigi_arbon(&arbo, "")?
                            .into_iter()
                            .map(|(vojo, haketo, _)| (vojo, haketo))
                            .collect())
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
    ) -> Result<Vec<(String, Haketo, u64)>, Eraro> {
        let arbo = self.legi_arbon(haketo)?;
        let mut dosieroj = Vec::new();
        let mut subarboj = Vec::new();
        for enskribo in arbo.enskriboj() {
            let vojo = kunigi(prefikso, enskribo.nomo())?;
            match enskribo {
                ArbaEnskribo::Dosiero {
                    haketo, grandeco, ..
                } => dosieroj.push((vojo, *haketo, *grandeco)),
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

/// Validas branĉan nomon: ĝi estas unu sekura voja komponanto, kiu ne
/// komenciĝas per punkto (tiel ĝi ne kolizias kun provizoraj dosieroj).
fn validi_brancxan_nomon(nomo: &str) -> Result<(), Eraro> {
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

fn nanosekundoj(tempo: SystemTime) -> Option<u64> {
    let dauxro = tempo.duration_since(UNIX_EPOCH).ok()?;
    u64::try_from(dauxro.as_nanos()).ok()
}

/// Rekursie forprenas malplenajn dosierujojn, preterlasante `.vs`.
fn forigi_malplenajn_dosierujojn(dosierujo: &Path) -> Result<(), Eraro> {
    for eniro in fs::read_dir(dosierujo)? {
        let eniro = eniro?;
        if !eniro.file_type()?.is_dir() || eniro.file_name().to_string_lossy() == VS_DOSIERUJO {
            continue;
        }
        let vojo = eniro.path();
        forigi_malplenajn_dosierujojn(&vojo)?;
        if fs::read_dir(&vojo)?.next().is_none() {
            fs::remove_dir(&vojo)?;
        }
    }
    Ok(())
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

    use super::{
        Deponejo, Eraro, HEAD_DOSIERO, INDEKSO_DOSIERO, Kunfando, OBJEKTUJO, Stato, VS_DOSIERUJO,
    };
    use crate::objektoj::{ArbaEnskribo, Komito};
    use crate::stokado::{Haketo, Konservilo};
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
            dua_gepatro: None,
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
            dua_gepatro: None,
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
    fn testo_kunfandi_antauxenigas() {
        let prov = ProvDeponejo::nova("kunfandi-antauxen");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"unua").unwrap();
        komiti(&deponejo);
        let unua = deponejo.legi_head().unwrap().unwrap();

        fs::write(prov.radiko.join("a.txt"), b"dua").unwrap();
        komiti(&deponejo);
        let dua = deponejo.legi_head().unwrap().unwrap();

        deponejo.eltiri(&unua).unwrap();
        deponejo.skribi_head(&unua).unwrap();

        assert_eq!(
            deponejo.kunfandi(&unua, &dua, "Kunfandis", 3).unwrap(),
            Kunfando::Antauen(dua)
        );
        assert_eq!(fs::read(prov.radiko.join("a.txt")).unwrap(), b"dua");
        deponejo.skribi_head(&dua).unwrap();
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
        fs::write(prov.radiko.join("sangxota.txt"), b"bazo").unwrap();
        komiti(&deponejo);
        let bazo = deponejo.legi_head().unwrap().unwrap();

        fs::write(prov.radiko.join("nur-majstro.txt"), b"m").unwrap();
        komiti(&deponejo);
        let majstro = deponejo.legi_head().unwrap().unwrap();

        deponejo.eltiri(&bazo).unwrap();
        deponejo.skribi_head(&bazo).unwrap();
        fs::write(prov.radiko.join("sangxota.txt"), b"fonto").unwrap();
        fs::write(prov.radiko.join("nur-fonto.txt"), b"f").unwrap();
        komiti(&deponejo);
        let fonto = deponejo.legi_head().unwrap().unwrap();

        deponejo.eltiri(&majstro).unwrap();
        deponejo.skribi_head(&majstro).unwrap();
        let rezulto = deponejo.kunfandi(&majstro, &fonto, "Kunfandis", 3).unwrap();
        let kunfando = match rezulto {
            Kunfando::Kunfandita(haketo) => haketo,
            alia => panic!("atendita kunfanda komito, sed ricevis: {alia:?}"),
        };

        let komito = deponejo.legi_komiton(&kunfando).unwrap();
        assert_eq!(komito.gepatro, Some(majstro));
        assert_eq!(komito.dua_gepatro, Some(fonto));
        assert_eq!(komito.mesagxo, "Kunfandis");
        assert_eq!(
            fs::read(prov.radiko.join("sangxota.txt")).unwrap(),
            b"fonto"
        );
        assert_eq!(fs::read(prov.radiko.join("komuna.txt")).unwrap(), b"bazo");
        assert!(prov.radiko.join("nur-majstro.txt").is_file());
        assert!(prov.radiko.join("nur-fonto.txt").is_file());

        deponejo.skribi_head(&kunfando).unwrap();
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
        let bazo = deponejo.legi_head().unwrap().unwrap();

        fs::write(prov.radiko.join("a.txt"), b"majstro").unwrap();
        komiti(&deponejo);
        let majstro = deponejo.legi_head().unwrap().unwrap();

        deponejo.eltiri(&bazo).unwrap();
        deponejo.skribi_head(&bazo).unwrap();
        fs::write(prov.radiko.join("a.txt"), b"fonto").unwrap();
        komiti(&deponejo);
        let fonto = deponejo.legi_head().unwrap().unwrap();

        deponejo.eltiri(&majstro).unwrap();
        deponejo.skribi_head(&majstro).unwrap();
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
        let bazo = deponejo.legi_head().unwrap().unwrap();

        fs::write(prov.radiko.join("a"), b"dosiero").unwrap();
        komiti(&deponejo);
        let majstro = deponejo.legi_head().unwrap().unwrap();

        deponejo.eltiri(&bazo).unwrap();
        deponejo.skribi_head(&bazo).unwrap();
        fs::create_dir_all(prov.radiko.join("a").join("b")).unwrap();
        fs::write(prov.radiko.join("a").join("b").join("c.txt"), b"c").unwrap();
        komiti(&deponejo);
        let fonto = deponejo.legi_head().unwrap().unwrap();

        deponejo.eltiri(&majstro).unwrap();
        deponejo.skribi_head(&majstro).unwrap();
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
                mesagxo: "Unua".into(),
                kreita_je: 1,
                dua_gepatro: None,
            })
            .unwrap();
        let dua = deponejo
            .konservi_komiton(&Komito {
                arbo,
                gepatro: None,
                mesagxo: "Dua".into(),
                kreita_je: 2,
                dua_gepatro: None,
            })
            .unwrap();
        deponejo.skribi_head(&unua).unwrap();

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
            gepatro: deponejo.legi_head().unwrap(),
            mesagxo: "testo".into(),
            kreita_je: 1,
            dua_gepatro: None,
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
        let unua = deponejo.legi_head().unwrap().unwrap();

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
        let unua = deponejo.legi_head().unwrap().unwrap();

        fs::remove_file(&dosiero).unwrap();
        fs::create_dir(&dosiero).unwrap();
        fs::write(dosiero.join("b.txt"), b"nesto").unwrap();
        komiti(&deponejo);
        let dua = deponejo.legi_head().unwrap().unwrap();

        deponejo.eltiri(&unua).unwrap();
        assert!(dosiero.is_file());
        assert_eq!(fs::read(&dosiero).unwrap(), b"dosiero");

        deponejo.eltiri(&dua).unwrap();
        assert!(dosiero.is_dir());
        assert_eq!(fs::read(dosiero.join("b.txt")).unwrap(), b"nesto");
    }

    #[test]
    fn testo_brancxoj_kaj_nuna_brancxo() {
        let prov = ProvDeponejo::nova("brancxoj");
        let deponejo = Deponejo::iniciati(prov.radiko.clone()).unwrap();
        assert_eq!(deponejo.brancxoj().unwrap(), Vec::new());
        assert_eq!(deponejo.nuna_brancxo().unwrap(), None);

        let cefa = Haketo::el_bajtoj([9; 32]);
        deponejo.skribi_brancxon("ĉefa", &cefa).unwrap();
        deponejo
            .skribi_brancxon("eksperimento", &Haketo::el_bajtoj([8; 32]))
            .unwrap();
        assert_eq!(deponejo.legi_brancxon("ĉefa").unwrap(), Some(cefa));
        let brancxoj = deponejo.brancxoj().unwrap();
        assert_eq!(
            brancxoj,
            [
                ("eksperimento".to_owned(), Haketo::el_bajtoj([8; 32])),
                ("ĉefa".to_owned(), cefa)
            ]
        );

        deponejo.agordi_nunan_brancxon("ĉefa").unwrap();
        assert_eq!(deponejo.nuna_brancxo().unwrap().as_deref(), Some("ĉefa"));
        deponejo.forigi_nunan_brancxon().unwrap();
        assert_eq!(deponejo.nuna_brancxo().unwrap(), None);

        for malvalida in ["", "../fuĝo", ".kaŝita", "a/b"] {
            assert!(
                matches!(
                    deponejo.skribi_brancxon(malvalida, &cefa),
                    Err(Eraro::NevalidaVojaKomponanto)
                ),
                "la nomo «{malvalida}» devus esti rifuzita"
            );
        }
    }
}
