//! Integraj testoj por `vs eltiri`.

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
            "versio-eltiro-{etikedo}-{}-{nombro}",
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

fn legi_head(dosierujo: &Path) -> String {
    std::fs::read_to_string(dosierujo.join(".vs").join("HEAD"))
        .unwrap()
        .trim()
        .to_owned()
}

fn prefikso(haketo: &str) -> String {
    haketo.chars().take(12).collect()
}

/// Kreas du komitojn kaj redonas iliajn plenajn haketojn.
fn du_komitoj(prov: &ProvDosierujo) -> (String, String) {
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    komiti(&prov.vojo, "Unua");
    let unua = legi_head(&prov.vojo);

    std::fs::write(prov.vojo.join("a.txt"), b"dua").unwrap();
    std::fs::create_dir(prov.vojo.join("sub")).unwrap();
    std::fs::write(prov.vojo.join("sub").join("nova.txt"), b"nova").unwrap();
    komiti(&prov.vojo, "Dua");
    let dua = legi_head(&prov.vojo);

    (unua, dua)
}

#[test]
fn testo_eltiri_restarigas_komiton_per_mallonga_haketo() {
    let prov = ProvDosierujo::nova("restarigo");
    let (unua, _) = du_komitoj(&prov);

    let eligo = ruli(&prov.vojo, &["eltiri", &prefikso(&unua)]);
    certigi_sukceson(&eligo);

    assert_eq!(legi_head(&prov.vojo), unua);
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"unua");
    assert!(!prov.vojo.join("sub").join("nova.txt").exists());
    assert!(
        !prov.vojo.join("sub").exists(),
        "la malplena dosierujo restis"
    );

    let stato = ruli(&prov.vojo, &["stato"]);
    certigi_sukceson(&stato);
    assert_eq!(String::from_utf8_lossy(&stato.stdout), "Neniu ŝanĝo\n");
}

#[test]
fn testo_eltiri_rifuzas_malpurigitan_laborspacon_kaj_perfortas() {
    let prov = ProvDosierujo::nova("perforto");
    let (unua, dua) = du_komitoj(&prov);

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", &prefikso(&unua)]));
    std::fs::write(prov.vojo.join("a.txt"), b"malpura").unwrap();

    // Reen al la pli nova komito sen perforto: rifuzata sen ŝanĝi ion.
    let eligo = ruli(&prov.vojo, &["eltiri", &prefikso(&dua)]);
    assert!(!eligo.status.success());
    assert!(
        String::from_utf8_lossy(&eligo.stderr).contains("nekomititajn ŝanĝojn"),
        "mankas la averto pri malpura laborspaco: {}",
        String::from_utf8_lossy(&eligo.stderr)
    );
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"malpura");
    assert_eq!(legi_head(&prov.vojo), unua);

    let eligo = ruli(&prov.vojo, &["eltiri", &prefikso(&dua), "--perforta"]);
    certigi_sukceson(&eligo);
    assert_eq!(legi_head(&prov.vojo), dua);
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"dua");
    assert_eq!(
        std::fs::read(prov.vojo.join("sub").join("nova.txt")).unwrap(),
        b"nova"
    );
}

#[test]
fn testo_eltiri_rifuzas_nekonatan_haketon() {
    let prov = ProvDosierujo::nova("nekonata");
    let _ = du_komitoj(&prov);

    let eligo = ruli(&prov.vojo, &["eltiri", "0000000000000000"]);
    assert!(!eligo.status.success());

    let eligo = ruli(&prov.vojo, &["eltiri", "xyz"]);
    assert!(!eligo.status.success());
}

#[test]
fn testo_eltiri_sen_deponejo_malsukcesas() {
    let prov = ProvDosierujo::nova("sen-deponejo");
    let eligo = ruli(&prov.vojo, &["eltiri", "00000000"]);
    assert!(!eligo.status.success());
}
