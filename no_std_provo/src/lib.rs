//! Kompila sentinelo por la `no_std`-kongrueco de `libversio`.
//!
//! Se iu ajn dependaĵo de `libversio` likas `std`, ĉi tiu crate nepre
//! malsukcesos kompili por nuda cela triopo.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::vec::Vec;

use libversio::{
    Arbo, DeponejoMotor, FiksaHorlogo, Komito, Laborspaco, MemoraKonservilo, Memspaco, haketi,
};

/// Vokas la purajn interfacojn de `libversio`: arbon, la deponejan motoron
/// sur enmemora laborspaco, komiton kaj staton.
///
/// La funkcio ekzistas nur por devigi la kompilon tra la tuta interfaca ĉeno
/// sen `std`.
#[must_use]
pub fn sentineleto() -> usize {
    let mut sukcesoj: usize = 0;
    if let Ok(arbo) = Arbo::el_enskriboj(Vec::new())
        && let Ok(bajtoj) = arbo.seriigi()
    {
        sukcesoj = sukcesoj.saturating_add(1);
        if haketi(&bajtoj).kiel_bajtoj().len() == 32 {
            sukcesoj = sukcesoj.saturating_add(1);
        }
    }

    let spaco = Memspaco::nova();
    if spaco.krei_dosierujon(".vs/objektoj").is_ok() && spaco.skribi_atome(".vs/KAPO", b"").is_ok()
    {
        let deponejo = DeponejoMotor::nova(
            spaco,
            MemoraKonservilo::nova(),
            FiksaHorlogo::nova(10_000_000_000),
        );
        if deponejo.laborspaco().skribi_atome("a.txt", b"a").is_ok()
            && let Ok(arbo) = deponejo.kapti_arbon()
            && let Ok(haketo) = deponejo.konservi_komiton(&Komito {
                arbo,
                gepatro: None,
                mesaĝo: alloc::string::String::from("sentinelo"),
                kreita_je: 1,
                dua_gepatro: None,
            })
            && deponejo.skribi_kapon(&haketo).is_ok()
            && deponejo.stato().is_ok_and(|stato| stato.estas_pura())
        {
            sukcesoj = sukcesoj.saturating_add(1);
        }
    }
    sukcesoj
}
