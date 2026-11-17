//! Integraj testoj por `vs stato`.

// PERMESO-JUSTIFIKO: nur testa kodo; panikigaj konstruaĵoj estas akcepteblaj.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "testa kodo rajtas uzi panikigajn konstrukciojn"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

static PROVOJ: AtomicU64 = AtomicU64::new(0);

struct ProvDosierujo {
    vojo: PathBuf,
}

impl ProvDosierujo {
    fn nova(etikedo: &str) -> Self {
        let nombro = PROVOJ.fetch_add(1, Ordering::Relaxed);
        let vojo = std::env::temp_dir().join(format!(
            "versio-cli-stato-{etikedo}-{}-{nombro}",
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

fn ruli(dosierujo: &Path, argumentoj: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vs"))
        .args(argumentoj)
        .current_dir(dosierujo)
        .output()
        .expect("ruli la binaran dosieron vs")
}

fn sukcesa_eligo(dosierujo: &Path, argumentoj: &[&str]) -> String {
    let eligo = ruli(dosierujo, argumentoj);
    assert!(
        eligo.status.success(),
        "vs malsukcesis: {}",
        String::from_utf8_lossy(&eligo.stderr)
    );
    String::from_utf8(eligo.stdout).unwrap()
}

#[test]
fn testo_stato_eligo() {
    let prov = ProvDosierujo::nova("eligo");
    sukcesa_eligo(&prov.vojo, &["iniciati"]);
    std::fs::write(prov.vojo.join("modifota.txt"), b"unua").unwrap();
    std::fs::write(prov.vojo.join("forigota.txt"), b"for").unwrap();
    assert_eq!(
        sukcesa_eligo(&prov.vojo, &["stato"]),
        "Novaj:\n  forigota.txt\n  modifota.txt\n"
    );

    sukcesa_eligo(&prov.vojo, &["komitu", "--priskribo", "Unua"]);
    assert_eq!(sukcesa_eligo(&prov.vojo, &["stato"]), "Neniu ŝanĝo\n");

    std::fs::write(prov.vojo.join("modifota.txt"), b"dua").unwrap();
    std::fs::remove_file(prov.vojo.join("forigota.txt")).unwrap();
    std::fs::create_dir(prov.vojo.join("sub")).unwrap();
    std::fs::write(prov.vojo.join("sub").join("nova.txt"), b"nova").unwrap();
    assert_eq!(
        sukcesa_eligo(&prov.vojo, &["stato"]),
        "Modifitaj:\n  modifota.txt\nNovaj:\n  sub/nova.txt\nForigitaj:\n  forigota.txt\n"
    );
}

#[test]
fn testo_stato_sen_deponejo_malsukcesas() {
    let prov = ProvDosierujo::nova("sen-deponejo");
    assert!(!ruli(&prov.vojo, &["stato"]).status.success());
}

/// Akceptokriterio de Fazo 3: 10 000 dosieroj, unu ŝanĝita, rezulto < 100 ms.
///
/// La tempolimo koncernas la eldonan binaron; neoptimumigita testa binaro
/// mezurus alian artefakton. Rulu per `cargo test --release -p versio-cli --test stato`.
#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "la tempolimo validas nur por la eldona binaro (--release)"
)]
fn testo_stato_10000_dosieroj_sub_100_ms() {
    let prov = ProvDosierujo::nova("rapideco");
    sukcesa_eligo(&prov.vojo, &["iniciati"]);

    // Realisma deponejo: la dosieroj estis skribitaj antaŭ longe.
    let malnova = SystemTime::now() - Duration::from_secs(3600);
    for dosierujo in 0..100 {
        let sub = prov.vojo.join(format!("d{dosierujo:03}"));
        std::fs::create_dir(&sub).unwrap();
        for dosiero in 0..100 {
            let vojo = sub.join(format!("f{dosiero:03}.txt"));
            std::fs::write(&vojo, format!("enhavo {dosierujo} {dosiero}\n")).unwrap();
            std::fs::File::options()
                .write(true)
                .open(&vojo)
                .unwrap()
                .set_modified(malnova)
                .unwrap();
        }
    }
    sukcesa_eligo(&prov.vojo, &["komitu", "--priskribo", "Granda"]);
    // La unua `stato` plenigas la indekson.
    assert_eq!(sukcesa_eligo(&prov.vojo, &["stato"]), "Neniu ŝanĝo\n");

    std::fs::write(prov.vojo.join("d042").join("f042.txt"), b"sxangxita\n").unwrap();

    let komenco = Instant::now();
    let eligo = sukcesa_eligo(&prov.vojo, &["stato"]);
    let dauxro = komenco.elapsed();

    assert_eq!(eligo, "Modifitaj:\n  d042/f042.txt\n");
    assert!(
        dauxro < Duration::from_millis(100),
        "vs stato daŭris {dauxro:?}"
    );
}
