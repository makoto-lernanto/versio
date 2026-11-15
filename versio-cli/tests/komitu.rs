//! Integraj testoj por la komandlinia interfaco.

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

use libversio::deponejo::Deponejo;
use libversio::objektoj::ArbaEnskribo;
use libversio::{Haketo, Konservilo};

static PROVOJ: AtomicU64 = AtomicU64::new(0);

struct ProvDosierujo {
    vojo: PathBuf,
}

impl ProvDosierujo {
    fn nova(etikedo: &str) -> Self {
        let nombro = PROVOJ.fetch_add(1, Ordering::Relaxed);
        let vojo = std::env::temp_dir().join(format!(
            "versio-cli-{etikedo}-{}-{nombro}",
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

#[test]
fn testo_iniciati_kaj_komitu() {
    let prov = ProvDosierujo::nova("unua");
    let vs = prov.vojo.join(".vs");

    certigi_sukceson(&ruli(&prov.vojo, &["iniciati"]));
    assert!(vs.join("HEAD").is_file());
    assert!(vs.join("objektoj").is_dir());

    std::fs::write(prov.vojo.join("testo.txt"), b"saluton\n").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--mesagxo", "Unua komito"]));

    let head = std::fs::read_to_string(vs.join("HEAD")).unwrap();
    let haketo: Haketo = head.trim().parse().expect("HEAD enhavu haketon");

    let deponejo = Deponejo::malfermi(prov.vojo.clone()).unwrap();
    let komito = deponejo.legi_komiton(&haketo).unwrap();
    assert_eq!(komito.mesagxo, "Unua komito");
    assert_eq!(komito.gepatro, None);

    let arbo = deponejo.legi_arbon(&komito.arbo).unwrap();
    assert_eq!(arbo.enskriboj().len(), 1);
    match &arbo.enskriboj()[0] {
        ArbaEnskribo::Dosiero {
            nomo,
            haketo,
            grandeco,
        } => {
            assert_eq!(nomo, "testo.txt");
            assert_eq!(*grandeco, 8);
            assert_eq!(
                deponejo.stokejo().ricevi(haketo).unwrap(),
                b"saluton\n".to_vec()
            );
        }
        alia @ ArbaEnskribo::SubArbo { .. } => {
            panic!("atendita dosiero, sed trovis: {alia:?}")
        }
    }
}

#[test]
fn testo_dua_komito_havas_gepatron() {
    let prov = ProvDosierujo::nova("dua");
    let vs = prov.vojo.join(".vs");

    certigi_sukceson(&ruli(&prov.vojo, &["iniciati"]));
    std::fs::write(prov.vojo.join("testo.txt"), b"unua").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--mesagxo", "Unua"]));
    let unua: Haketo = std::fs::read_to_string(vs.join("HEAD"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    std::fs::write(prov.vojo.join("testo.txt"), b"dua").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--mesagxo", "Dua"]));
    let dua: Haketo = std::fs::read_to_string(vs.join("HEAD"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    assert_ne!(unua, dua);
    let deponejo = Deponejo::malfermi(prov.vojo.clone()).unwrap();
    assert_eq!(deponejo.legi_komiton(&dua).unwrap().gepatro, Some(unua));
}

#[test]
fn testo_komitu_sen_deponejo_malsukcesas() {
    let prov = ProvDosierujo::nova("sen-deponejo");
    let eligo = ruli(&prov.vojo, &["komitu", "--mesagxo", "Sen deponejo"]);

    assert!(!eligo.status.success());
}
