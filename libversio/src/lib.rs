//! Kernbiblioteko de la versiokontrola sistemo Versio.
//!
//! La biblioteko estas `no_std`-kongrueca: la datumstrukturoj kaj la
//! enmemora stokado funkcias per `core` kaj `alloc`, dum la dosiersistema
//! realigo estas ŝaltata per la trajto `std`.

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
pub mod deponejo;
pub mod eraroj;
mod kodigo;
pub mod objektoj;
pub mod stokado;
pub mod vojo;

#[cfg(kani)]
mod pruvoj;

pub use eraroj::Eraro;
pub use objektoj::{ArbaEnskribo, Arbo, Dosiero, Komito};
pub use stokado::{HAKETA_LONGO, Haketo, Konservilo, MemoraKonservilo, haketi, kontroli};
pub use vojo::{MAKSIMA_KOMPONANTO, validi_bajtojn, validi_komponanton};

#[cfg(feature = "std")]
pub use deponejo::Deponejo;
#[cfg(feature = "std")]
pub use stokado::DosieraKonservilo;
