//! Kernbiblioteko de la versiokontrola sistemo Versio.
//!
//! La biblioteko estas `no_std`-kongrueca: la datumstrukturoj, la deponeja
//! motoro kaj la enmemora stokado funkcias per `core` kaj `alloc`. La
//! dosiersistema laborspaco kaj la `lz4`-stokejo estas ŝaltataj per la trajto
//! `std`.

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod deponejo;
pub mod eraroj;
pub mod ignoro;
pub mod indekso;
mod kodigo;
pub mod laborspaco;
pub mod objektoj;
pub mod stokado;
pub mod vojo;

#[cfg(kani)]
mod pruvoj;

pub use eraroj::Eraro;
pub use ignoro::Ignoro;
pub use laborspaco::{Ero, FiksaHorlogo, Horlogo, Laborspaco, Memspaco, Metadatumoj, Speco};
pub use objektoj::{ArbaEnskribo, Arbo, Dosiero, Komito};
pub use stokado::{HAKETA_LONGO, Haketo, Konservilo, MemoraKonservilo, Stokejo, haketi, kontroli};
pub use vojo::{
    MAKSIMA_KOMPONANTO, MAKSIMA_VOJO, validi_bajtojn, validi_komponanton, validi_vojon,
};

pub use deponejo::{
    Ago, DeponejoMotor, EtikedaCelo, Etikedo, Kunfando, Movhistorio, Movo, Referenco, Rubokolekto,
    Stato,
};

#[cfg(feature = "std")]
pub use deponejo::Deponejo;
#[cfg(feature = "std")]
pub use laborspaco::{Dosierspaco, StdaHorlogo};
#[cfg(feature = "std")]
pub use stokado::DosieraKonservilo;
