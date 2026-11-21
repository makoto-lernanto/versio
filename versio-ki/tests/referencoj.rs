//! Integraj testoj por la referencoj: etikedoj, movhistorio kaj rubokolekto.

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

use libversio::{DosieraKonservilo, Konservilo};

static PROVOJ: AtomicU64 = AtomicU64::new(0);

struct ProvDosierujo {
    vojo: PathBuf,
}

impl ProvDosierujo {
    fn nova(etikedo: &str) -> Self {
        let nombro = PROVOJ.fetch_add(1, Ordering::Relaxed);
        let vojo = std::env::temp_dir().join(format!(
            "versio-referencoj-{etikedo}-{}-{nombro}",
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

fn komiti(dosierujo: &Path, noto: &str) -> String {
    certigi_sukceson(&ruli(dosierujo, &["komitu", "--noto", noto]));
    legi_kapon(dosierujo)
}

fn legi_kapon(dosierujo: &Path) -> String {
    std::fs::read_to_string(dosierujo.join(".vs").join("KAPO"))
        .unwrap()
        .trim()
        .to_owned()
}

fn legi_etikedon(dosierujo: &Path, nomo: &str) -> String {
    std::fs::read_to_string(
        dosierujo
            .join(".vs")
            .join("referencoj")
            .join("etikedoj")
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
fn testo_etikedi_kaj_historio_el_etikedo() {
    let prov = ProvDosierujo::nova("etikedi");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    let unua = komiti(&prov.vojo, "Unua");
    std::fs::write(prov.vojo.join("a.txt"), b"dua").unwrap();
    let dua = komiti(&prov.vojo, "Dua");

    let eligo = ruli(&prov.vojo, &["etikedi", "v1.0"]);
    certigi_sukceson(&eligo);
    assert!(
        String::from_utf8_lossy(&eligo.stdout).contains("Etikedis v1.0"),
        "mankas konfirmo de la etikedo"
    );
    assert_eq!(legi_etikedon(&prov.vojo, "v1.0"), format!("komito {dua}"));

    certigi_sukceson(&ruli(&prov.vojo, &["etikedi", "v0.9", &prefikso(&unua)]));
    assert_eq!(legi_etikedon(&prov.vojo, "v0.9"), format!("komito {unua}"));

    let eligo = ruli(&prov.vojo, &["historio", "v0.9"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);
    assert!(teksto.contains("Unua"), "mankas la unua komito:\n{teksto}");
    assert!(
        !teksto.contains("Dua"),
        "la etikedo v0.9 ne rajtas montri pli novan komiton:\n{teksto}"
    );

    let eligo = ruli(&prov.vojo, &["historio", "v1.0"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);
    assert!(teksto.contains("Dua") && teksto.contains("Unua"));

    let eligo = ruli(&prov.vojo, &["etikedoj"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);
    assert!(teksto.contains("v0.9") && teksto.contains("v1.0"));

    let eligo = ruli(&prov.vojo, &["etikedi", "v1.0"]);
    assert!(!eligo.status.success());
    assert!(
        String::from_utf8_lossy(&eligo.stderr).contains("jam ekzistas"),
        "mankas la rifuzo de duobla etikedo"
    );

    let eligo = ruli(&prov.vojo, &["etikedi", "malbona", "nenia"]);
    assert!(!eligo.status.success());

    std::fs::write(prov.vojo.join("a.txt"), b"tria").unwrap();
    komiti(&prov.vojo, "Tria");
    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "v0.9"]));
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"unua");
    assert_eq!(legi_kapon(&prov.vojo), unua);

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "v1.0"]));
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"dua");
    assert_eq!(legi_kapon(&prov.vojo), dua);

    let eligo = ruli(&prov.vojo, &["historio", "nenia"]);
    assert!(!eligo.status.success());
    assert!(
        String::from_utf8_lossy(&eligo.stderr).contains("neniu etikedo nek branĉo"),
        "mankas klara eraro pri nekonata celo"
    );
}

#[test]
fn testo_etikedo_povas_montri_al_alia_etikedo() {
    let prov = ProvDosierujo::nova("aliaso");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    let unua = komiti(&prov.vojo, "Unua");

    certigi_sukceson(&ruli(&prov.vojo, &["etikedi", "bazo", &prefikso(&unua)]));
    let eligo = ruli(&prov.vojo, &["etikedi", "lasta", "bazo"]);
    certigi_sukceson(&eligo);
    assert_eq!(legi_etikedon(&prov.vojo, "lasta"), "etikedo bazo");

    let eligo = ruli(&prov.vojo, &["etikedoj"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);
    assert!(
        teksto.contains("lasta") && teksto.contains("-> bazo"),
        "la etikeda listo ne montras la aliason:\n{teksto}"
    );

    let eligo = ruli(&prov.vojo, &["historio", "lasta"]);
    certigi_sukceson(&eligo);
    assert!(String::from_utf8_lossy(&eligo.stdout).contains("Unua"));
}

#[test]
fn testo_renomi_kaj_forigi_branĉon() {
    let prov = ProvDosierujo::nova("renomi");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    let unua = komiti(&prov.vojo, "Unua");

    certigi_sukceson(&ruli(&prov.vojo, &["linio", "malnova"]));
    certigi_sukceson(&ruli(&prov.vojo, &["renomi", "malnova", "nova"]));
    let eligo = ruli(&prov.vojo, &["linioj"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);
    assert!(teksto.contains("nova") && !teksto.contains("malnova"));

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "nova"]));
    let eligo = ruli(&prov.vojo, &["forigi", "nova"]);
    certigi_sukceson(&eligo);
    assert!(
        String::from_utf8_lossy(&eligo.stdout).contains("detaĉita"),
        "la forigo de la nuna branĉo devas raporti la detaĉon"
    );
    assert_eq!(legi_kapon(&prov.vojo), unua);

    let eligo = ruli(&prov.vojo, &["linioj"]);
    certigi_sukceson(&eligo);
    assert_eq!(String::from_utf8_lossy(&eligo.stdout), "Neniu branĉo\n");

    let eligo = ruli(&prov.vojo, &["forigi", "nova"]);
    assert!(!eligo.status.success());
    assert!(String::from_utf8_lossy(&eligo.stderr).contains("ne ekzistas"));

    let eligo = ruli(&prov.vojo, &["renomi", "nenia", "alia"]);
    assert!(!eligo.status.success());
    assert!(String::from_utf8_lossy(&eligo.stderr).contains("ne ekzistas"));
}

#[test]
fn testo_movhistorio_registras_kaj_montras_movojn() {
    let prov = ProvDosierujo::nova("movhistorio");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    let unua = komiti(&prov.vojo, "Unua");
    certigi_sukceson(&ruli(&prov.vojo, &["linio", "ĉefa"]));
    std::fs::write(prov.vojo.join("a.txt"), b"dua").unwrap();
    let dua = komiti(&prov.vojo, "Dua");

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", &prefikso(&unua)]));
    let eligo = ruli(&prov.vojo, &["movhistorio"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);

    for deviga in [
        "KAPO",
        "ĉefa",
        "komito",
        "kreado",
        "eltiro",
        &prefikso(&unua),
        &prefikso(&dua),
    ] {
        assert!(
            teksto.contains(deviga),
            "mankas «{deviga}» en la movhistorio:\n{teksto}"
        );
    }
    let lasta = teksto.lines().next().unwrap_or_default();
    assert!(
        lasta.contains(&prefikso(&unua)) && lasta.ends_with("eltiro"),
        "la plej nova movo devas esti la eltiro al {unua}:\n{teksto}"
    );
}

#[test]
fn testo_rubokolekti_forigas_nur_neatingeblajn_objektojn() {
    let prov = ProvDosierujo::nova("rubokolekto");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    let unua = komiti(&prov.vojo, "Unua");
    std::fs::write(prov.vojo.join("a.txt"), b"dua").unwrap();
    let dua = komiti(&prov.vojo, "Dua");

    certigi_sukceson(&ruli(&prov.vojo, &["etikedi", "unua", &unua]));
    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", &unua]));

    let stokejo = DosieraKonservilo::nova(prov.vojo.join(".vs").join("objektoj"));
    let rubo = stokejo.konservi(b"rubo").unwrap();
    let ruba_vojo = stokejo.dosierujo().join(format!("{rubo}.lz4"));
    assert!(ruba_vojo.is_file());

    let eligo = ruli(&prov.vojo, &["rubokolekti"]);
    certigi_sukceson(&eligo);
    assert!(
        String::from_utf8_lossy(&eligo.stdout)
            .contains("Forigis 1 objekton, konservis 6 objektojn."),
        "la movhistorio devas konservi la antaŭan KAPO-valoron"
    );
    assert!(!ruba_vojo.exists(), "la ruba objekto devas malaperi");

    std::fs::remove_file(prov.vojo.join(".vs").join("movhistorio")).unwrap();
    let eligo = ruli(&prov.vojo, &["rubokolekti"]);
    certigi_sukceson(&eligo);
    assert!(
        String::from_utf8_lossy(&eligo.stdout)
            .contains("Forigis 3 objektojn, konservis 3 objektojn."),
        "restas nur la KAPO kaj la etikedo"
    );

    certigi_sukceson(&ruli(&prov.vojo, &["stato"]));
    let eligo = ruli(&prov.vojo, &["historio"]);
    certigi_sukceson(&eligo);
    assert!(String::from_utf8_lossy(&eligo.stdout).contains("Unua"));

    let eligo = ruli(&prov.vojo, &["eltiri", &prefikso(&dua)]);
    assert!(
        !eligo.status.success(),
        "la forigita komito {dua} ne plu eltireblas"
    );
}

#[test]
fn testo_helpo_listigas_la_novajn_komandojn() {
    let prov = ProvDosierujo::nova("helpo");
    let eligo = ruli(&prov.vojo, &["--helpo"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);

    for komando in [
        "etikedi",
        "etikedoj",
        "renomi",
        "forigi",
        "movhistorio",
        "rubokolekti",
    ] {
        assert!(
            teksto.contains(komando),
            "mankas «{komando}» en la helppano:\n{teksto}"
        );
    }
}
