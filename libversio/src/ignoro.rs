//! Reguloj por preterlasi dosierojn kaj dosierujojn en la laborspaco.
//!
//! La motoro ĉiam preterlasas la metadatuman dosierujon `.vs`, kaj defaŭlte
//! ankaŭ `.git` kaj `target`. Pliaj reguloj venas el `.vsignore`-dosieroj en
//! la radiko kaj en ĉiu subdosierujo. La formato estas simpligita versio de
//! `.gitignore`: komentoj komenciĝas per `#`, `!` nuligas antaŭan regulon,
//! fina `/` limigas regulon al dosierujoj, kaj `*`, `?` kaj `**` estas
//! ŝablonsignoj. La lasta kongrua regulo venkas.
//!
//! Preterlasitaj dosierujoj estas tute ne skanataj. Tial `!`-regulo ene de
//! preterlasita dosierujo ne povas remalfermi ĝin; tio estas intenca limigo,
//! kiu tenas la skanadon simpla kaj rapidan.

use alloc::string::String;
use alloc::vec::Vec;

/// Unu regulo el `.vsignore`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Regulo {
    /// La dosierujo, en kiu la regulo estis difinita (relativa, sen fina `/`).
    bazo: String,
    /// La ŝablono, sen komenca `/` kaj sen fina `/`.
    padrono: String,
    /// Ĉu la regulo remalfermas anstataŭ preterlasi.
    malŝaltas: bool,
    /// Ĉu la regulo validas nur por dosierujoj.
    nur_dosierujo: bool,
    /// Ĉu la ŝablono estas ligita al la bazo anstataŭ al iu ajn komponanto.
    ligita: bool,
}

/// Aro da preterlasaj reguloj, ordigitaj de la plej malnova al la plej nova.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ignoro {
    reguloj: Vec<Regulo>,
}

impl Ignoro {
    /// La defaŭltaj reguloj: `.git`, `target` kaj `.vs`.
    ///
    /// `.vs` estas ankaŭ aparte preterlasata de la skanado, do `.vsignore` ne
    /// povas remalfermi la metadatumojn.
    #[must_use]
    pub fn defaŭlta() -> Self {
        let mut ignoro = Self::default();
        for padrono in [".git", "target", ".vs"] {
            ignoro.reguloj.push(Regulo {
                bazo: String::new(),
                padrono: String::from(padrono),
                malŝaltas: false,
                nur_dosierujo: false,
                ligita: false,
            });
        }
        ignoro
    }

    /// Aldonas la regulojn el unu `.vsignore`-dosiero.
    ///
    /// `bazo` estas la relativa vojo de la dosierujo, kiu enhavas la dosieron;
    /// la radika dosierujo estas la malplena teksto. Malplenaj linioj,
    /// komentoj kaj evidente nevalidaj ŝablonoj estas silentigitaj.
    pub fn aldoni(&mut self, bazo: &str, enhavo: &str) {
        for linio in enhavo.lines() {
            let linio = linio.trim();
            if linio.is_empty() || linio.starts_with('#') {
                continue;
            }
            let (malŝaltas, linio) = match linio.strip_prefix('!') {
                Some(resto) => (true, resto),
                None => (false, linio),
            };
            let linio = linio.trim();
            if linio.is_empty() {
                continue;
            }
            let (nur_dosierujo, linio) = match linio.strip_suffix('/') {
                Some(resto) => (true, resto),
                None => (false, linio),
            };
            let (ligita, linio) = match linio.strip_prefix('/') {
                Some(resto) => (true, resto),
                None => (false, linio),
            };
            if linio.is_empty() || linio.split('/').any(|parto| parto == "..") {
                continue;
            }
            self.reguloj.push(Regulo {
                bazo: String::from(bazo),
                padrono: String::from(linio),
                malŝaltas,
                nur_dosierujo,
                ligita: ligita || linio.contains('/'),
            });
        }
    }

    /// Ĉu la relativa `vojo` estas preterlasata.
    ///
    /// `dosierujo` devas esti `true`, kiam la vojo mem estas dosierujo.
    #[must_use]
    pub fn ignorata(&self, vojo: &str, dosierujo: bool) -> bool {
        let mut ignorata = false;
        for regulo in &self.reguloj {
            let Some(relativa) = relativa_al_bazo(vojo, &regulo.bazo) else {
                continue;
            };
            let kongruas = if regulo.ligita {
                ligita_kongruas(&regulo.padrono, relativa, dosierujo, regulo.nur_dosierujo)
            } else {
                komponanto_kongruas(&regulo.padrono, relativa, dosierujo, regulo.nur_dosierujo)
            };
            if kongruas {
                ignorata = !regulo.malŝaltas;
            }
        }
        ignorata
    }
}

/// Redonas la relativan vojon de `vojo` rilate al `bazo`, aŭ `None` se la
/// vojo ne situas sub la bazo.
fn relativa_al_bazo<'a>(vojo: &'a str, bazo: &str) -> Option<&'a str> {
    if bazo.is_empty() {
        return Some(vojo);
    }
    let resto = vojo.strip_prefix(bazo)?;
    resto.strip_prefix('/')
}

/// Ĉu la ŝablono kongruas kun iu ajn komponanto aŭ kun la plena vojo.
///
/// Regulo sen `/` validas por ĉiu komponanto de la vojo, do ĝi ankaŭ kovras
/// la enhavon de kongrua dosierujo. La lasta komponanto obeas `nur_dosierujo`.
fn komponanto_kongruas(padrono: &str, vojo: &str, dosierujo: bool, nur_dosierujo: bool) -> bool {
    let mut partoj = vojo.split('/').peekable();
    while let Some(parto) = partoj.next() {
        let lasta = partoj.peek().is_none();
        if (!lasta || dosierujo || !nur_dosierujo) && ŝablono_kongruas(padrono, parto) {
            return true;
        }
    }
    false
}

/// Ĉu ligita ŝablono kongruas kun la vojo aŭ kun iu ajn el ĝiaj prauloj.
fn ligita_kongruas(padrono: &str, vojo: &str, dosierujo: bool, nur_dosierujo: bool) -> bool {
    if (dosierujo || !nur_dosierujo) && ŝablono_kongruas(padrono, vojo) {
        return true;
    }
    for (indekso, signo) in vojo.char_indices() {
        if signo == '/' && ŝablono_kongruas(padrono, vojo.get(..indekso).unwrap_or("")) {
            return true;
        }
    }
    false
}

/// Ĉu `padrono` kongruas kun la tuta `teksto`.
fn ŝablono_kongruas(padrono: &str, teksto: &str) -> bool {
    kongruas_bajtojn(padrono.as_bytes(), teksto.as_bytes())
}

fn kongruas_bajtojn(padrono: &[u8], teksto: &[u8]) -> bool {
    let Some((&unua, resto)) = padrono.split_first() else {
        return teksto.is_empty();
    };
    match unua {
        b'*' => {
            if resto.first() == Some(&b'*') {
                let post = resto.get(1..).unwrap_or(&[]);
                if post.first() == Some(&b'/') {
                    let post = post.get(1..).unwrap_or(&[]);
                    if kongruas_bajtojn(post, teksto) {
                        return true;
                    }
                }
                for indekso in 0..=teksto.len() {
                    if kongruas_bajtojn(post, teksto.get(indekso..).unwrap_or(&[])) {
                        return true;
                    }
                }
                return false;
            }
            for indekso in 0..=teksto.len() {
                if kongruas_bajtojn(resto, teksto.get(indekso..).unwrap_or(&[])) {
                    return true;
                }
                if teksto.get(indekso) == Some(&b'/') {
                    break;
                }
            }
            false
        }
        b'?' => {
            teksto.first().is_some_and(|signo| *signo != b'/')
                && kongruas_bajtojn(resto, teksto.get(1..).unwrap_or(&[]))
        }
        _ => {
            teksto.first() == Some(&unua) && kongruas_bajtojn(resto, teksto.get(1..).unwrap_or(&[]))
        }
    }
}

#[cfg(test)]
mod testoj {
    // PERMESO-JUSTIFIKO: nur testa kodo; panikigaj konstruaĵoj estas akcepteblaj.
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "testa kodo rajtas uzi panikigajn konstrukciojn"
    )]

    use super::Ignoro;

    #[test]
    fn defaŭltaj_reguloj() {
        let ignoro = Ignoro::defaŭlta();
        assert!(ignoro.ignorata(".git", true));
        assert!(ignoro.ignorata(".git/config", false));
        assert!(ignoro.ignorata("target/debug/x", false));
        assert!(ignoro.ignorata(".vs/KAPO", false));
        assert!(!ignoro.ignorata("src/lib.rs", false));
        assert!(!ignoro.ignorata("targeto", true));
    }

    #[test]
    fn ŝablonoj_kaj_komentoj() {
        let mut ignoro = Ignoro::defaŭlta();
        ignoro.aldoni("", "# komento\n*.log\nbuild/\n/eĉa.txt\n!grava.log\n");
        assert!(ignoro.ignorata("a.log", false));
        assert!(ignoro.ignorata("sub/a.log", false));
        assert!(!ignoro.ignorata("grava.log", false));
        assert!(ignoro.ignorata("build", true));
        assert!(!ignoro.ignorata("build", false));
        assert!(ignoro.ignorata("sub/build", true));
        assert!(ignoro.ignorata("eĉa.txt", false));
        assert!(!ignoro.ignorata("sub/eĉa.txt", false));
    }

    #[test]
    fn solidoj_kaj_duobla_stelo() {
        let mut ignoro = Ignoro::default();
        ignoro.aldoni("", "a/**/b\n?.tmp\n");
        assert!(ignoro.ignorata("a/b", false));
        assert!(ignoro.ignorata("a/x/b", false));
        assert!(ignoro.ignorata("a/x/y/b", false));
        assert!(!ignoro.ignorata("a/xb", false));
        assert!(ignoro.ignorata("x.tmp", false));
        assert!(!ignoro.ignorata("xy.tmp", false));
    }

    #[test]
    fn nestitaj_reguloj_estas_relativaj() {
        let mut ignoro = Ignoro::default();
        ignoro.aldoni("sub", "*.tmp\n!tenu.tmp\n");
        assert!(ignoro.ignorata("sub/a.tmp", false));
        assert!(!ignoro.ignorata("sub/tenu.tmp", false));
        assert!(!ignoro.ignorata("a.tmp", false));
    }
}
