//! Unueca erartipo de `libversio`.

use alloc::string::String;

use crate::stokado::Haketo;

/// Ĉiuj eraroj, kiujn povas produkti la kernbiblioteko.
#[derive(Debug, thiserror::Error)]
pub enum Eraro {
    /// La teksto ne estas valida deksesuma reprezento de `BLAKE3`-haketo.
    #[error("nevalida haketo")]
    NevalidaHaketo,

    /// La petita objekto ne ekzistas en la stokejo.
    #[error("objekto {haketo} ne ekzistas")]
    NeekzistantaObjekto {
        /// La haketo, kiun oni provis legi.
        haketo: Haketo,
    },

    /// La legitaj datumoj ne kongruas kun la haketo, sub kiu ili estis petita.
    #[error("datumoj difektitaj: atendita {atendita}, akirita {akirita}")]
    DatumoDifektita {
        /// La haketo uzita por la legado.
        atendita: Haketo,
        /// La haketo kalkulita el la legitaj datumoj.
        akirita: Haketo,
    },

    /// La dosierujo ne enhavas `.vs`-metadatumojn.
    #[error("la dosierujo ne estas Versio-deponejo")]
    Nedeponejo,

    /// Dosiernomo ne estas valida UTF-8.
    #[error("nevalida dosiernomo (ne estas UTF-8)")]
    NevalidaDosiernomo,

    /// Voja komponanto estas malplena, tro longa aŭ enhavas malpermesitan signon.
    #[error("nevalida voja komponanto")]
    NevalidaVojaKomponanto,

    /// Du enskriboj en la sama arbo havas la saman nomon.
    #[error("duplikata vojo: {nomo}")]
    DuplikataVojo {
        /// La ripetita nomo.
        nomo: String,
    },

    /// Longoprefikso aŭ kvanto superas la difinitan supran limon.
    #[error("tro longa enigo: akirita {akirita}, maksimumo {maksimumo}")]
    TrolongaEnigo {
        /// La permesita maksimumo.
        maksimumo: usize,
        /// La valoro trovita en la enigo.
        akirita: usize,
    },

    /// La bajtoj ne sekvas la atenditan formato-strukturon.
    #[error("nevalida seriigaĵo: {kialo}")]
    NevalidaDatumo {
        /// Mallonga klarigo de la problemo.
        kialo: &'static str,
    },

    /// La laborspaco enhavas dosiertipon, kiun la komando ankoraŭ ne subtenas.
    #[error("nesubtenita dosiertipo: {vojo}")]
    NesubtenitaDosiertipo {
        /// La vojo de la koncerna ero.
        vojo: String,
    },

    /// Seriiga eraro de `postcard`.
    #[error("seriiga eraro: {0}")]
    Seriigo(postcard::Error),

    /// Malseriiga eraro de `postcard`.
    #[error("malseriiga eraro: {0}")]
    Malseriigo(postcard::Error),

    /// Dosiersistema eraro (nur kun la trajto `std`).
    #[cfg(feature = "std")]
    #[error("dosiera eraro: {0}")]
    Io(#[from] std::io::Error),
}
