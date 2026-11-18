//! Integraj testoj por `vs historio`.

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

static PROVOJ: AtomicU64 = AtomicU64::new(0);

struct ProvDosierujo {
    vojo: PathBuf,
}

impl ProvDosierujo {
    fn nova(etikedo: &str) -> Self {
        let nombro = PROVOJ.fetch_add(1, Ordering::Relaxed);
        let vojo = std::env::temp_dir().join(format!(
            "versio-historo-{etikedo}-{}-{nombro}",
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

fn certigi_sukceson(eligo: &Output) {
    assert!(
        eligo.status.success(),
        "vs malsukcesis: {}",
        String::from_utf8_lossy(&eligo.stderr)
    );
}

fn komiti(dosierujo: &Path, noto: &str) {
    certigi_sukceson(&ruli(dosierujo, &["komitu", "--noto", noto]));
}

#[test]
fn testo_historio_montras_komitojn_unue_la_plej_novan() {
    let prov = ProvDosierujo::nova("ordo");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("testo.txt"), b"unua").unwrap();
    komiti(&prov.vojo, "Unua");
    std::fs::write(prov.vojo.join("testo.txt"), b"dua").unwrap();
    komiti(&prov.vojo, "Dua");
    std::fs::write(prov.vojo.join("testo.txt"), b"tria").unwrap();
    komiti(&prov.vojo, "Tria");

    let eligo = ruli(&prov.vojo, &["historio"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);
    let linioj: Vec<&str> = teksto.lines().collect();

    assert_eq!(linioj.len(), 3, "neatendita eligo:\n{teksto}");
    assert!(linioj[0].ends_with("  Tria"), "malĝusta ordo:\n{teksto}");
    assert!(linioj[1].ends_with("  Dua"), "malĝusta ordo:\n{teksto}");
    assert!(linioj[2].ends_with("  Unua"), "malĝusta ordo:\n{teksto}");

    let kapo = std::fs::read_to_string(prov.vojo.join(".vs").join("KAPO")).unwrap();
    let unua_linio = linioj[0];
    let kampoj: Vec<&str> = unua_linio.split("  ").collect();
    assert_eq!(kampoj.len(), 3, "neatendita linia formato:\n{unua_linio}");
    assert_eq!(kampoj[0].len(), 12);
    assert_eq!(kampoj[1].len(), 23);
    assert!(kapo.trim().starts_with(kampoj[0]));
}

#[test]
fn testo_historio_limigas_la_nombron() {
    let prov = ProvDosierujo::nova("limo");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("testo.txt"), b"unua").unwrap();
    komiti(&prov.vojo, "Unua");
    std::fs::write(prov.vojo.join("testo.txt"), b"dua").unwrap();
    komiti(&prov.vojo, "Dua");

    let eligo = ruli(&prov.vojo, &["historio", "-n", "1"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);
    let linioj: Vec<&str> = teksto.lines().collect();
    assert_eq!(linioj.len(), 1, "neatendita eligo:\n{teksto}");
    assert!(linioj[0].ends_with("  Dua"));

    let eligo = ruli(&prov.vojo, &["historio", "-n", "0"]);
    certigi_sukceson(&eligo);
    assert_eq!(eligo.stdout, Vec::<u8>::new());
}

#[test]
fn testo_historio_sen_komitoj() {
    let prov = ProvDosierujo::nova("malplena");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));

    let eligo = ruli(&prov.vojo, &["historio"]);
    certigi_sukceson(&eligo);
    assert_eq!(
        String::from_utf8_lossy(&eligo.stdout),
        "Neniu komito ankoraŭ\n"
    );
}

#[test]
fn testo_historio_sen_deponejo_malsukcesas() {
    let prov = ProvDosierujo::nova("sen-deponejo");
    let eligo = ruli(&prov.vojo, &["historio"]);
    assert!(!eligo.status.success());
}
