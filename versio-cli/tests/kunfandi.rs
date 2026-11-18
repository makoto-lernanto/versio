//! Integraj testoj por `vs kunfandi`.

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

use libversio::Haketo;
use libversio::deponejo::Deponejo;

static PROVOJ: AtomicU64 = AtomicU64::new(0);

struct ProvDosierujo {
    vojo: PathBuf,
}

impl ProvDosierujo {
    fn nova(etikedo: &str) -> Self {
        let nombro = PROVOJ.fetch_add(1, Ordering::Relaxed);
        let vojo = std::env::temp_dir().join(format!(
            "versio-kunfandi-{etikedo}-{}-{nombro}",
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

fn legi_head(dosierujo: &Path) -> Haketo {
    std::fs::read_to_string(dosierujo.join(".vs").join("HEAD"))
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

/// Preparas du diverĝantajn liniojn el komuna bazo.
///
/// Rezulto: `majstro` kaj `fonto` ŝanĝis la saman dosieron malsame, kaj la
/// laborspaco staras sur `majstro`.
fn preparu_diverĝon(prov: &ProvDosierujo) -> (Haketo, Haketo) {
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("komuna.txt"), b"bazo").unwrap();
    std::fs::write(prov.vojo.join("konflikta.txt"), b"bazo").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Unua"]));

    certigi_sukceson(&ruli(&prov.vojo, &["linio", "majstro"]));
    certigi_sukceson(&ruli(&prov.vojo, &["linio", "fonto"]));

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "majstro"]));
    std::fs::write(prov.vojo.join("nur-majstro.txt"), b"m").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Majstro"]));

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "fonto"]));
    std::fs::write(prov.vojo.join("nur-fonto.txt"), b"f").unwrap();
    std::fs::write(prov.vojo.join("konflikta.txt"), b"fonto").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Fonto"]));

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "majstro"]));
    std::fs::write(prov.vojo.join("konflikta.txt"), b"majstro").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Majstro denove"]));

    let majstro = legi_head(&prov.vojo);
    let fonto = Deponejo::malfermi(prov.vojo.clone())
        .unwrap()
        .legi_branĉon("fonto")
        .unwrap()
        .unwrap();
    (majstro, fonto)
}

#[test]
fn testo_kunfandi_kreas_dugepatran_komiton() {
    let prov = ProvDosierujo::nova("du-gepatroj");
    let (_, fonto) = preparu_diverĝon(&prov);

    // Revenigu la konfliktan dosieron al la baza enhavo: nur `fonto` ŝanĝis ĝin.
    std::fs::write(prov.vojo.join("konflikta.txt"), b"bazo").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Majstro pura"]));
    let majstro = legi_head(&prov.vojo);

    let eligo = ruli(&prov.vojo, &["kunfandi", "fonto"]);
    certigi_sukceson(&eligo);
    assert!(
        String::from_utf8_lossy(&eligo.stdout).contains("Kunfanda komito"),
        "mankas konfirmo de la kunfanda komito"
    );

    let kunfando = legi_head(&prov.vojo);
    let deponejo = Deponejo::malfermi(prov.vojo.clone()).unwrap();
    let komito = deponejo.legi_komiton(&kunfando).unwrap();
    assert_eq!(komito.gepatro, Some(majstro));
    assert_eq!(komito.dua_gepatro, Some(fonto));
    assert_eq!(komito.mesaĝo, "Kunfandis linion fonto");
    assert_eq!(
        deponejo.legi_branĉon("majstro").unwrap(),
        Some(kunfando),
        "la nuna linio devas montri al la kunfanda komito"
    );
    assert!(deponejo.stato().unwrap().estas_pura());

    assert!(prov.vojo.join("nur-majstro.txt").is_file());
    assert!(prov.vojo.join("nur-fonto.txt").is_file());
    assert_eq!(
        std::fs::read(prov.vojo.join("konflikta.txt")).unwrap(),
        b"fonto"
    );
}

#[test]
fn testo_kunfandi_antaŭenigas() {
    let prov = ProvDosierujo::nova("antaŭen");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"unua").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Unua"]));
    certigi_sukceson(&ruli(&prov.vojo, &["linio", "majstro"]));
    certigi_sukceson(&ruli(&prov.vojo, &["linio", "fonto"]));

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "fonto"]));
    std::fs::write(prov.vojo.join("a.txt"), b"dua").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Dua"]));
    let dua = legi_head(&prov.vojo);

    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "majstro"]));
    let eligo = ruli(&prov.vojo, &["kunfandi", "fonto"]);
    certigi_sukceson(&eligo);
    assert!(
        String::from_utf8_lossy(&eligo.stdout).contains("Antaŭen al"),
        "mankas konfirmo pri antaŭenigo"
    );

    assert_eq!(legi_head(&prov.vojo), dua);
    let deponejo = Deponejo::malfermi(prov.vojo.clone()).unwrap();
    assert_eq!(deponejo.legi_branĉon("majstro").unwrap(), Some(dua));
    assert!(deponejo.stato().unwrap().estas_pura());
    assert_eq!(std::fs::read(prov.vojo.join("a.txt")).unwrap(), b"dua");
}

#[test]
fn testo_kunfandi_raportas_konflikton() {
    let prov = ProvDosierujo::nova("konflikto");
    let (majstro, _) = preparu_diverĝon(&prov);

    let eligo = ruli(&prov.vojo, &["kunfandi", "fonto"]);
    assert!(!eligo.status.success(), "kunfando devus malsukcesi");
    let eraro = String::from_utf8_lossy(&eligo.stderr);
    assert!(
        eraro.contains("kunfanda konflikto") && eraro.contains("konflikta.txt"),
        "la eraro ne raportas la konflikton:\n{eraro}"
    );
    assert_eq!(legi_head(&prov.vojo), majstro);
    assert_eq!(
        std::fs::read(prov.vojo.join("konflikta.txt")).unwrap(),
        b"majstro"
    );
}

#[test]
fn testo_kunfandi_rifuzas_malpuron_laborspacon() {
    let prov = ProvDosierujo::nova("malpura");
    preparu_diverĝon(&prov);
    std::fs::write(prov.vojo.join("nur-majstro.txt"), b"nekomitita").unwrap();

    let eligo = ruli(&prov.vojo, &["kunfandi", "fonto"]);
    assert!(!eligo.status.success());
    assert!(
        String::from_utf8_lossy(&eligo.stderr).contains("nekomititajn ŝanĝojn"),
        "mankas averto pri malpura laborspaco"
    );
}

#[test]
fn testo_kunfandi_raportas_mankantan_linion() {
    let prov = ProvDosierujo::nova("mankanta");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"a").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Unua"]));

    let eligo = ruli(&prov.vojo, &["kunfandi", "neekzista"]);
    assert!(!eligo.status.success());
    assert!(
        String::from_utf8_lossy(&eligo.stderr).contains("ne ekzistas"),
        "mankas klara eraro pri neekzistanta linio"
    );
}

#[test]
fn testo_kunfandi_jam_kunfandita() {
    let prov = ProvDosierujo::nova("nenio");
    certigi_sukceson(&ruli(&prov.vojo, &["ini"]));
    std::fs::write(prov.vojo.join("a.txt"), b"a").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Unua"]));
    certigi_sukceson(&ruli(&prov.vojo, &["linio", "majstro"]));
    certigi_sukceson(&ruli(&prov.vojo, &["eltiri", "majstro"]));

    let eligo = ruli(&prov.vojo, &["kunfandi", "majstro"]);
    certigi_sukceson(&eligo);
    assert!(
        String::from_utf8_lossy(&eligo.stdout).contains("Nenio farenda"),
        "mankas konfirmo pri nenio farenda"
    );
}

#[test]
fn testo_historio_montras_ambaŭ_gepatrojn() {
    let prov = ProvDosierujo::nova("historio");
    preparu_diverĝon(&prov);
    std::fs::write(prov.vojo.join("konflikta.txt"), b"bazo").unwrap();
    certigi_sukceson(&ruli(&prov.vojo, &["komitu", "--noto", "Pura"]));

    certigi_sukceson(&ruli(&prov.vojo, &["kunfandi", "fonto"]));
    let eligo = ruli(&prov.vojo, &["historio", "--nombro", "10"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);

    for deviga in ["Kunfandis linion fonto", "Fonto", "Pura", "Unua"] {
        assert!(
            teksto.contains(deviga),
            "mankas «{deviga}» en la historio:\n{teksto}"
        );
    }
}

#[test]
fn testo_helpo_listigas_kunfandi() {
    let prov = ProvDosierujo::nova("helpo");
    let eligo = ruli(&prov.vojo, &["--helpo"]);
    certigi_sukceson(&eligo);
    let teksto = String::from_utf8_lossy(&eligo.stdout);
    assert!(
        teksto.contains("kunfandi"),
        "mankas kunfandi en la helppano:\n{teksto}"
    );
    assert!(
        teksto.contains("Kunfandas linion"),
        "mankas la priskribo de kunfandi:\n{teksto}"
    );
}
