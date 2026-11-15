//! Kani-pruvoj por la pura kerno de Versio.
//!
//! La pruvoj respektas la limojn de la konstitucio: malgrandaj enigoj,
//! `kani::cover!` por malhelpi malplenajn pruvojn, kaj stubo por
//! `blake3::hash`. La kolizio-rezisto de la reala haketilo estas komputa
//! supozo, ne parto de la pruvado.

use alloc::string::String;
use alloc::vec;

use crate::eraroj::Eraro;
use crate::kodigo::limigi_kvanton;
use crate::objektoj::{ArbaEnskribo, Arbo, Komito, MAKSIMAJ_ENSKRIBOJ};
use crate::stokado::{HAKETA_LONGO, Haketo, Konservilo, MemoraKonservilo, haketi};
use crate::vojo::validi_bajtojn;

/// Determinisma anstataŭo por `blake3::hash` en la pruvoj.
///
/// Por samlongaj enigoj ĝi estas injekcia, do unu-bita ŝanĝo ĉiam ŝanĝas la
/// haketon. La reala `BLAKE3` estas komputa supozo, ne pruvata ĉi tie.
fn prova_haketo(datumoj: &[u8]) -> blake3::Hash {
    let mut bajtoj = [0u8; HAKETA_LONGO];
    let longo = datumoj.len();
    if longo > 0 {
        for (indekso, celo) in bajtoj.iter_mut().enumerate() {
            if let Some(fonto) = datumoj.get(indekso % longo) {
                *celo = *fonto;
            }
        }
    }
    blake3::Hash::from_bytes(bajtoj)
}

fn simbolika_nomo() -> char {
    let bajto: u8 = kani::any();
    // JUSTIFIKO: nur minusklaj ASCII-literoj estas akceptataj kiel unu-signaj
    // nomoj en la pruvoj; aliaj bajtoj ne apartenas al la modelo.
    kani::assume(bajto.is_ascii_lowercase());
    char::from(bajto)
}

fn fiksa_haketo() -> Haketo {
    Haketo::el_bajtoj([7u8; HAKETA_LONGO])
}

/// 1. Ajna blob konservita kaj poste legita estas bajte identa.
#[kani::proof]
#[kani::unwind(10)]
#[kani::stub(blake3::hash, prova_haketo)]
fn pruvo_stokado_roundtrip() {
    let datumoj: [u8; 8] = kani::any();
    let stokejo = MemoraKonservilo::nova();

    let konservita = stokejo.konservi(&datumoj);
    kani::cover!(konservita.is_ok());
    assert!(konservita.is_ok(), "enmemora konservo devas sukcesi");

    if let Ok(haketo) = konservita {
        let legita = stokejo.ricevi(&haketo);
        kani::cover!(legita.is_ok());
        assert!(legita.is_ok(), "enmemora legado devas sukcesi");
        if let Ok(enhavo) = legita {
            assert!(
                enhavo.as_slice() == datumoj.as_slice(),
                "legitaj datumoj devas egali la konservitajn"
            );
        }
    }
}

/// 2. Ajna unu-bita difekto estas rifuzata per `DatumoDifektita`.
#[kani::proof]
#[kani::unwind(10)]
#[kani::stub(blake3::hash, prova_haketo)]
fn pruvo_difekto_rifuzita() {
    let originalaj: [u8; 8] = kani::any();
    let indekso: usize = kani::any();
    // JUSTIFIKO: la modelo havas ekzakte 8 bajtojn; eksteraj indeksoj ne ekzistas.
    kani::assume(indekso < 8);
    let bito: u8 = kani::any();
    // JUSTIFIKO: u8 havas bitojn 0..=7; aliaj valoroj ne estas bitoj.
    kani::assume(bito < 8);

    let haketo = haketi(&originalaj);
    let mut difektitaj = originalaj;
    if let Some(bajto) = difektitaj.get_mut(indekso) {
        *bajto ^= 1u8.wrapping_shl(u32::from(bito));
    }
    assert!(difektitaj != originalaj, "la bito devas ŝanĝiĝi");

    let stokejo = MemoraKonservilo::nova();
    stokejo.enigi_krudajn(haketo, difektitaj.to_vec());
    let rezulto = stokejo.ricevi(&haketo);

    kani::cover!(matches!(&rezulto, Err(Eraro::DatumoDifektita { .. })));
    assert!(
        matches!(&rezulto, Err(Eraro::DatumoDifektita { .. })),
        "difektitaj datumoj devas esti rifuzitaj"
    );
}

/// 3. Permuto de enskriboj ne ŝanĝas la seriigitajn bajtojn.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_arbo_kanonika() {
    let nomo_a = simbolika_nomo();
    let nomo_b = simbolika_nomo();
    // JUSTIFIKO: kanonika arbo malpermesas samajn nomojn; la pruvo temas pri
    // permuto de du malsamaj nomoj.
    kani::assume(nomo_a != nomo_b);

    let haketo = fiksa_haketo();
    let unua = Arbo::el_enskriboj(vec![
        ArbaEnskribo::Dosiero {
            nomo: String::from(nomo_a),
            haketo,
            grandeco: 0,
        },
        ArbaEnskribo::SubArbo {
            nomo: String::from(nomo_b),
            haketo,
        },
    ]);
    let dua = Arbo::el_enskriboj(vec![
        ArbaEnskribo::SubArbo {
            nomo: String::from(nomo_b),
            haketo,
        },
        ArbaEnskribo::Dosiero {
            nomo: String::from(nomo_a),
            haketo,
            grandeco: 0,
        },
    ]);

    kani::cover!(nomo_a < nomo_b);
    kani::cover!(nomo_a > nomo_b);
    assert!(
        unua.is_ok() && dua.is_ok(),
        "ambaŭ arboj devas esti validaj"
    );

    if let (Ok(unua), Ok(dua)) = (unua, dua) {
        let unua_bajtoj = unua.seriigi();
        let dua_bajtoj = dua.seriigi();
        assert!(
            unua_bajtoj.is_ok() && dua_bajtoj.is_ok(),
            "seriigado devas sukcesi"
        );
        if let (Ok(unua_bajtoj), Ok(dua_bajtoj)) = (unua_bajtoj, dua_bajtoj) {
            assert!(
                unua_bajtoj == dua_bajtoj,
                "permuto ne rajtas ŝanĝi la seriigitajn bajtojn"
            );
        }
    }
}

/// 4. Duplikataj vojoj estas rifuzitaj.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_arbo_duplikatoj() {
    let nomo = simbolika_nomo();
    let haketo = fiksa_haketo();

    let rezulto = Arbo::el_enskriboj(vec![
        ArbaEnskribo::Dosiero {
            nomo: String::from(nomo),
            haketo,
            grandeco: 0,
        },
        ArbaEnskribo::SubArbo {
            nomo: String::from(nomo),
            haketo,
        },
    ]);

    kani::cover!(matches!(&rezulto, Err(Eraro::DuplikataVojo { .. })));
    assert!(
        matches!(&rezulto, Err(Eraro::DuplikataVojo { .. })),
        "duplikata nomo devas esti rifuzita"
    );

    // Pozitiva kontrolo: malsamaj nomoj estas akceptataj.
    let valida = Arbo::el_enskriboj(vec![
        ArbaEnskribo::Dosiero {
            nomo: String::from("a"),
            haketo,
            grandeco: 0,
        },
        ArbaEnskribo::SubArbo {
            nomo: String::from("b"),
            haketo,
        },
    ]);
    assert!(valida.is_ok(), "malsamaj nomoj devas esti akceptitaj");
}

/// 5. Danĝeraj vojkonsisteroj estas rifuzitaj; ajna enigo redonas `Result`.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_vojo_sekura() {
    let bajtoj: [u8; 8] = kani::any();
    let rezulto = validi_bajtojn(&bajtoj);

    kani::cover!(rezulto.is_ok());
    kani::cover!(rezulto.is_err());
    if rezulto.is_ok() {
        for &bajto in &bajtoj {
            assert!(bajto != b'/', "oblikva streko devas esti rifuzita");
            assert!(bajto != b'\\', "kontraŭoblikva streko devas esti rifuzita");
            assert!(bajto != 0, "NUL-signo devas esti rifuzita");
        }
    }

    assert!(
        validi_bajtojn(b"").is_err(),
        "malplena nomo devas esti rifuzita"
    );
    assert!(validi_bajtojn(b"..").is_err(), "«..» devas esti rifuzita");
    assert!(validi_bajtojn(b"a/b").is_err(), "«/» devas esti rifuzita");
}

/// 6. Ajna 16-bajta enigo al la limigita malseriigado redonas `Result`.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_dekodo_neniam_panikas() {
    let bajtoj: [u8; 16] = kani::any();
    let rezulto = Arbo::malseriigi(&bajtoj);

    kani::cover!(matches!(&rezulto, Err(Eraro::Malseriigo(_))));
    kani::cover!(matches!(&rezulto, Err(Eraro::NevalidaDatumo { .. })));

    // La sukcesa vojo estas atingebla per la valida kodigo de malplena arbo.
    assert!(
        Arbo::malseriigi(&[0]).is_ok(),
        "malplena arbo devas dekodiĝi"
    );
    assert!(
        Arbo::malseriigi(&[0, 0]).is_err(),
        "postrestaj bajtoj devas esti rifuzitaj"
    );
    assert!(
        Komito::malseriigi(&[]).is_err(),
        "malplena komito devas esti rifuzita"
    );
}

/// 7. Deklarita kvanto estas limigita antaŭ ol rezervi memoraĵon.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_dekodo_limigita() {
    let kvanto: u32 = kani::any();
    let rezulto = limigi_kvanton(kvanto, 16, MAKSIMAJ_ENSKRIBOJ);

    kani::cover!(rezulto.is_ok());
    kani::cover!(rezulto.is_err());
    if let Ok(valoro) = rezulto {
        assert!(
            valoro <= MAKSIMAJ_ENSKRIBOJ,
            "la supra limo devas esti respektata"
        );
        assert!(
            valoro <= 16 / 35,
            "la kvanto devas kongrui kun la restantaj bajtoj"
        );
    }

    // Malica trolonga longoprefikso estas rifuzata sen grandega rezervado.
    let malica = [0xffu8; 16];
    assert!(
        Arbo::malseriigi(&malica).is_err(),
        "trolonga prefikso devas esti rifuzita"
    );
}
