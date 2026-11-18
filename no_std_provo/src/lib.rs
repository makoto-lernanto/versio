//! Kompila sentinelo por la `no_std`-kongrueco de `libversio`.
//!
//! Se iu ajn dependaĵo de `libversio` likas `std`, ĉi tiu crate nepre
//! malsukcesos kompili por nuda cela triopo.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::vec::Vec;

use libversio::{Arbo, haketi};

/// Vokas la purajn interfacojn de `libversio`: konstrui arbon, seriigi, haketi.
///
/// La funkcio ekzistas nur por devigi la kompilon tra la tuta interfaca ĉeno
/// sen `std`.
#[must_use]
pub fn sentineleto() -> usize {
    if let Ok(arbo) = Arbo::el_enskriboj(Vec::new())
        && let Ok(bajtoj) = arbo.seriigi()
    {
        return haketi(&bajtoj).kiel_bajtoj().len();
    }
    0
}
