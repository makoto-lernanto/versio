//! Kernbiblioteko de la versiokontrola sistemo Versio.
//!
//! La biblioteko estas `no_std`-kongrueca: la datumstrukturoj kaj la
//! enhavadresita stokado funkcias per `core` kaj `alloc`, dum la
//! dosiersistema realigo estas ŝaltata per la trajto `std`.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
pub mod deponejo;
pub mod eraroj;
pub mod objektoj;
pub mod stokado;

pub use eraroj::Eraro;
pub use objektoj::{ArbaEnskribo, Arbo, Dosiero, Komito};
pub use stokado::{HAKETA_LONGO, Haketo, Konservilo, haketi, kontroli};

#[cfg(feature = "std")]
pub use deponejo::Deponejo;
#[cfg(feature = "std")]
pub use stokado::DosieraKonservilo;
