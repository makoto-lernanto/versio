//! Integraj testoj por `vs linio` kaj `vs linioj`.

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
            "versio-linio-{etikedo}-{}-{nombro}",
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

fn legi_kapon(dosierujo: &Path) -> String {
    std::fs::read_to_string(dosierujo.join(".vs").join("KAPO"))
        .unwrap()
        .trim()
        .to_owned()
}

fn legi_ref(dosierujo: &Path, nomo: &str) -> String {
    std::fs::read_to_string(
        dosierujo
            .join(".vs")
            .join("referencoj")
            .join("branĉoj")
            .join(nomo),
    )
    .unwrap()
    .trim()
    .to_owned()
}

fn prefikso(haketo: &str) -> String {
    haketo.chars().take(12).collect()
}

#[test]
fn testo_linio_eltiri_kaj_avanci() {
    let prov = ProvDosierujo::nova("avanci");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    komiti(&prov.vojo, "Unua");
    let unua = legi_kapon(&prov.vojo);
    std::fs::write(prov.vojo.join("a.txt"), b"dua").unwrap();
    komiti(&prov.vojo, "Dua");
    let dua = legi_kapon(&prov.vojo);

    certigi_sukceson(&ruli(&prov.vojo, &["linio", "ĉefa"]));
    assert_eq!(legi_ref(&prov.vojo, "ĉefa"), dua);

    let eligo = ruli(&prov.vojo, &["linioj"]);
    certigi_sukceson(&eligo);
    assert_eq!(
        String::from_utf8_lossy(&eligo.stdout),
        format!("  ĉefa  {}\n", prefikso(&dua))
    );

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "ĉefa"]));
    assert_eq!(legi_kapon(&prov.vojo), dua);
    let eligo = ruli(&prov.vojo, &["linioj"]);
    assert_eq!(
        String::from_utf8_lossy(&eligo.stdout),
        format!("* ĉefa  {}\n", prefikso(&dua))
    );

    // Komito sur branĉo movas la referencon kaj KAPO.
    std::fs::write(prov.vojo.join("a.txt"), b"tria").unwrap();
    komiti(&prov.vojo, "Tria");
    let tria = legi_kapon(&prov.vojo);
    assert_eq!(legi_ref(&prov.vojo, "ĉefa"), tria);
    assert_ne!(tria, dua);

    // Detaĉita eltiro ĉe malnova komito ne movas la branĉon.
    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", &prefikso(&unua)]));
    assert_eq!(legi_kapon(&prov.vojo), unua);
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"unua");
    let eligo = ruli(&prov.vojo, &["linioj"]);
    assert_eq!(
        String::from_utf8_lossy(&eligo.stdout),
        format!("  ĉefa  {}\n", prefikso(&tria))
    );

    std::fs::write(prov.vojo.join("a.txt"), b"detachita").unwrap();
    komiti(&prov.vojo, "Detaĉita");
    assert_eq!(legi_ref(&prov.vojo, "ĉefa"), tria);

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "ĉefa"]));
    assert_eq!(legi_kapon(&prov.vojo), tria);
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"tria");
}

#[test]
fn testo_linio_ce_malnova_komito_kaj_rifuzo() {
    let prov = ProvDosierujo::nova("malnova");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    komiti(&prov.vojo, "Unua");
    let unua = legi_kapon(&prov.vojo);
    std::fs::write(prov.vojo.join("a.txt"), b"dua").unwrap();
    komiti(&prov.vojo, "Dua");

    certigi_sukceson(&ruli(&prov.vojo, &["linio", "malnova", &prefikso(&unua)]));
    assert_eq!(legi_ref(&prov.vojo, "malnova"), unua);

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "malnova"]));
    assert_eq!(legi_kapon(&prov.vojo), unua);
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"unua");

    let eligo = ruli(&prov.vojo, &["linio", "malnova"]);
    assert!(!eligo.status.success());
    assert!(
        String::from_utf8_lossy(&eligo.stderr).contains("jam ekzistas"),
        "mankas la rifuzo de ekzistanta branĉo: {}",
        String::from_utf8_lossy(&eligo.stderr)
    );
}

#[test]
fn testo_linio_rifuzas_malvalidajn_nomojn() {
    let prov = ProvDosierujo::nova("nomoj");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    komiti(&prov.vojo, "Unua");

    for nomo in ["a/b", ".kaŝita", "kun spaco"] {
        let eligo = ruli(&prov.vojo, &["linio", nomo]);
        assert!(!eligo.status.success(), "la nomo «{nomo}» estis akceptita");
    }
}

#[test]
fn testo_linio_sen_komito_malsukcesas() {
    let prov = ProvDosierujo::nova("sen-komito");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    let eligo = ruli(&prov.vojo, &["linio", "ĉefa"]);
    assert!(!eligo.status.success());
}

#[test]
fn testo_linioj_sen_deponejo_malsukcesas() {
    let prov = ProvDosierujo::nova("sen-deponejo");
    let eligo = ruli(&prov.vojo, &["linioj"]);
    assert!(!eligo.status.success());
}
